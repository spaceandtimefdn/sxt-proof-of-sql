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

/// Provable numerical `+` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl AddExpr {
    /// Create numerical `+` expression
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

impl ProofExpr for AddExpr {
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
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, false);
        Ok(Column::Decimal75(self.precision(), self.scale(), res))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.add_expr.final_round_evaluate",
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
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, false);
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
        Ok(lhs_eval + rhs_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

impl DecimalProofExpr for AddExpr {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::table_utility::{borrowed_int, table},
            math::decimal::Precision,
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::collections::VecDeque;

    fn literal(value: LiteralValue) -> Box<DynProofExpr> {
        Box::new(DynProofExpr::new_literal(value))
    }

    #[test]
    fn add_expr_rejects_non_numeric_operands() {
        let err = AddExpr::try_new(
            literal(LiteralValue::Boolean(true)),
            literal(LiteralValue::Int(7)),
        )
        .unwrap_err();

        assert!(matches!(
            err,
            AnalyzeError::DataTypeMismatch {
                left_type,
                right_type
            } if left_type == "BOOLEAN" && right_type == "INT"
        ));
    }

    #[test]
    fn add_expr_exposes_operands_and_decimal_result_type() {
        let expr = AddExpr::try_new(
            literal(LiteralValue::SmallInt(2)),
            literal(LiteralValue::Int(5)),
        )
        .unwrap();

        assert_eq!(expr.lhs().data_type(), ColumnType::SmallInt);
        assert_eq!(expr.rhs().data_type(), ColumnType::Int);
        assert_eq!(
            expr.data_type(),
            ColumnType::Decimal75(Precision::new(11).unwrap(), 0)
        );
    }

    #[test]
    fn add_expr_evaluates_literal_sum_in_first_and_final_rounds() {
        let alloc = Bump::new();
        let data = table([borrowed_int::<TestScalar>("rows", [0, 1, 2], &alloc)]);
        let expr = AddExpr::try_new(
            literal(LiteralValue::BigInt(5)),
            literal(LiteralValue::Int(2)),
        )
        .unwrap();

        for result in [
            expr.first_round_evaluate(&alloc, &data, &[]).unwrap(),
            expr.final_round_evaluate(
                &mut FinalRoundBuilder::new(2, VecDeque::new()),
                &alloc,
                &data,
                &[],
            )
            .unwrap(),
        ] {
            let Column::Decimal75(precision, scale, values) = result else {
                panic!("expected Decimal75 result");
            };
            assert_eq!(precision, Precision::new(20).unwrap());
            assert_eq!(scale, 0);
            assert_eq!(values, [TestScalar::from(7_u64); 3]);
        }
    }

    #[test]
    fn add_expr_verifier_adds_child_evaluations() {
        let expr = AddExpr::try_new(
            literal(LiteralValue::BigInt(5)),
            literal(LiteralValue::Int(2)),
        )
        .unwrap();
        let mut builder =
            MockVerificationBuilder::new(vec![], 2, vec![], vec![], vec![], vec![], vec![]);

        let result = expr
            .verifier_evaluate(
                &mut builder,
                &IndexMap::default(),
                TestScalar::from(3_u64),
                &[],
            )
            .unwrap();

        assert_eq!(result, TestScalar::from(21_u64));
    }
}
