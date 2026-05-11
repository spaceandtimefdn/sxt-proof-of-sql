use super::{DecimalProofExpr, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_multiply_column_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::multiply_columns,
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable numerical * expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiplyExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl MultiplyExpr {
    /// Create numerical `*` expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_multiply_column_types(left_datatype, right_datatype)
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

impl ProofExpr for MultiplyExpr {
    fn data_type(&self) -> ColumnType {
        try_multiply_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to multiply column types")
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let res = multiply_columns(&lhs_column, &rhs_column, alloc);
        Ok(Column::Decimal75(self.precision(), self.scale(), res))
    }

    #[tracing::instrument(name = "MultiplyExpr::final_round_evaluate", level = "info", skip_all)]
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

        // lhs_times_rhs
        let lhs_times_rhs: &'a [S] = multiply_columns(&lhs_column, &rhs_column, alloc);
        builder.produce_intermediate_mle(lhs_times_rhs);

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(lhs_times_rhs)]),
                (-S::one(), vec![Box::new(lhs_column), Box::new(rhs_column)]),
            ],
        );
        let res = Column::Decimal75(self.precision(), self.scale(), lhs_times_rhs);

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;

        // lhs_times_rhs
        let lhs_times_rhs = builder.try_consume_final_round_mle_evaluation()?;

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_times_rhs - lhs * rhs,
            2,
        )?;

        // selection
        Ok(lhs_times_rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

impl DecimalProofExpr for MultiplyExpr {}

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
    fn we_can_verify_multiply_expr_helper_paths() {
        let alloc = Bump::new();
        let lhs = [
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(5),
            TestScalar::from(7),
        ];
        let rhs = [
            TestScalar::from(11),
            TestScalar::from(13),
            TestScalar::from(17),
            TestScalar::from(19),
        ];
        let expected: Vec<_> = lhs
            .iter()
            .zip(rhs.iter())
            .map(|(lhs, rhs)| *lhs * *rhs)
            .collect();
        let data = table([
            borrowed_decimal75("a", 12, 0, lhs, &alloc),
            borrowed_decimal75("b", 12, 0, rhs, &alloc),
        ]);
        let t: TableRef = "sxt.t".parse().unwrap();
        let decimal_type = ColumnType::Decimal75(Precision::new(12).unwrap(), 0);
        let a = ColumnRef::new(t.clone(), Ident::from("a"), decimal_type);
        let b = ColumnRef::new(t, Ident::from("b"), decimal_type);
        let multiply_expr = MultiplyExpr::try_new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
            Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
        )
        .unwrap();

        assert_eq!(multiply_expr.lhs().data_type(), decimal_type);
        assert_eq!(multiply_expr.rhs().data_type(), decimal_type);

        let mut referenced_columns: IndexSet<ColumnRef> = IndexSet::default();
        multiply_expr.get_column_references(&mut referenced_columns);
        assert_eq!(
            referenced_columns.into_iter().collect::<Vec<_>>(),
            vec![a.clone(), b.clone()]
        );

        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(4, VecDeque::new());
        let result = multiply_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &data, &[])
            .unwrap();
        assert_eq!(
            result,
            Column::Decimal75(
                multiply_expr.precision(),
                multiply_expr.scale(),
                alloc.alloc_slice_copy(&expected)
            )
        );

        let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(4);
        let verification_builder = run_verify_for_each_row(
            4,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            3,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    a.clone().column_id() => lhs.as_slice().inner_product(evaluation_point),
                    b.clone().column_id() => rhs.as_slice().inner_product(evaluation_point)
                };
                let eval = multiply_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                    .unwrap();
                assert_eq!(eval, expected.as_slice().inner_product(evaluation_point));
            },
        );
        assert_eq!(
            verification_builder.get_identity_results(),
            vec![vec![true]; 4]
        );
    }
}
