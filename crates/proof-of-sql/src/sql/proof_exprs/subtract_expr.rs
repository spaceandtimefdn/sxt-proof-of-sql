use super::{add_subtract_columns, DecimalProofExpr, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{
            try_add_subtract_column_types, Column, ColumnRef, ColumnType, LiteralValue, Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable numerical `-` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubtractExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl SubtractExpr {
    /// Create numerical `-` expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_add_subtract_column_types(left_datatype, right_datatype)
            .map(|_| Self { lhs, rhs })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: left_datatype.to_string(),
                right_type: right_datatype.to_string(),
            })
    }

    /// Get the left-hand side expression
    pub fn lhs(&self) -> &DynProofExpr {
        &self.lhs
    }

    /// Get the right-hand side expression
    pub fn rhs(&self) -> &DynProofExpr {
        &self.rhs
    }
}

impl ProofExpr for SubtractExpr {
    fn data_type(&self) -> ColumnType {
        try_add_subtract_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to add/subtract column types")
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, true);
        Ok(Column::Decimal75(self.precision(), self.scale(), res))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.subtract_expr.final_round_evaluate",
        level = "info",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, true);

        log::log_memory_usage("End");

        Ok(Column::Decimal75(self.precision(), self.scale(), res))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs_eval = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs_eval = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        Ok(lhs_eval - rhs_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

impl DecimalProofExpr for SubtractExpr {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{
                table_utility::{borrowed_decimal75, table},
                Column, ColumnRef, ColumnType, TableRef,
            },
            map::{indexmap, IndexSet},
            math::decimal::Precision,
            polynomial::MultilinearExtension,
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::{
                mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder,
                FirstRoundBuilder,
            },
            proof_exprs::{ColumnExpr, DecimalProofExpr},
        },
    };
    use alloc::vec::Vec;
    use bumpalo::Bump;
    use sqlparser::ast::Ident;
    use std::collections::VecDeque;

    #[test]
    fn we_can_verify_subtract_expr_helper_paths() {
        let alloc = Bump::new();
        let lhs = [
            TestScalar::from(23),
            TestScalar::from(29),
            TestScalar::from(31),
            TestScalar::from(37),
        ];
        let rhs = [
            TestScalar::from(3),
            TestScalar::from(5),
            TestScalar::from(7),
            TestScalar::from(11),
        ];
        let expected: Vec<_> = lhs
            .iter()
            .zip(rhs.iter())
            .map(|(lhs, rhs)| *lhs - *rhs)
            .collect();
        let data = table([
            borrowed_decimal75("a", 12, 0, lhs, &alloc),
            borrowed_decimal75("b", 12, 0, rhs, &alloc),
        ]);
        let t: TableRef = "sxt.t".parse().unwrap();
        let decimal_type = ColumnType::Decimal75(Precision::new(12).unwrap(), 0);
        let a = ColumnRef::new(t.clone(), Ident::from("a"), decimal_type);
        let b = ColumnRef::new(t, Ident::from("b"), decimal_type);
        let subtract_expr = SubtractExpr::try_new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
            Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
        )
        .unwrap();

        assert_eq!(subtract_expr.lhs().data_type(), decimal_type);
        assert_eq!(subtract_expr.rhs().data_type(), decimal_type);

        let mut referenced_columns: IndexSet<ColumnRef> = IndexSet::default();
        subtract_expr.get_column_references(&mut referenced_columns);
        assert_eq!(
            referenced_columns.into_iter().collect::<Vec<_>>(),
            vec![a.clone(), b.clone()]
        );

        let mut final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
        let result = subtract_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &data, &[])
            .unwrap();
        assert_eq!(
            result,
            Column::Decimal75(
                subtract_expr.precision(),
                subtract_expr.scale(),
                alloc.alloc_slice_copy(&expected)
            )
        );

        let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(4);
        let verification_builder = run_verify_for_each_row(
            4,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            1,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    a.clone().column_id() => lhs.as_slice().inner_product(evaluation_point),
                    b.clone().column_id() => rhs.as_slice().inner_product(evaluation_point)
                };
                let eval = subtract_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                    .unwrap();
                assert_eq!(eval, expected.as_slice().inner_product(evaluation_point));
            },
        );
        assert!(verification_builder.get_identity_results().is_empty());
    }
}
