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
                table_utility::{borrowed_bigint, table},
                TableRef,
            },
            map::{IndexMap, IndexSet},
            math::decimal::Precision,
            scalar::test_scalar::TestScalar,
        },
        sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
    };
    use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    fn literal(value: LiteralValue) -> DynProofExpr {
        DynProofExpr::new_literal(value)
    }

    #[test]
    fn subtract_expr_exposes_operands_and_decimal_type() {
        let lhs = literal(LiteralValue::BigInt(10));
        let rhs = literal(LiteralValue::Int(3));

        let expr = SubtractExpr::try_new(Box::new(lhs.clone()), Box::new(rhs.clone())).unwrap();

        assert_eq!(expr.lhs(), &lhs);
        assert_eq!(expr.rhs(), &rhs);
        assert_eq!(
            expr.data_type(),
            ColumnType::Decimal75(Precision::new(20).unwrap(), 0)
        );
    }

    #[test]
    fn subtract_expr_rejects_mismatched_operand_types() {
        let err = SubtractExpr::try_new(
            Box::new(literal(LiteralValue::VarChar("abc".into()))),
            Box::new(literal(LiteralValue::BigInt(1))),
        )
        .unwrap_err();

        assert!(matches!(
            err,
            AnalyzeError::DataTypeMismatch {
                left_type: _,
                right_type: _
            }
        ));
    }

    #[test]
    fn subtract_expr_evaluates_literal_columns_in_first_and_final_rounds() {
        let alloc = Bump::new();
        let table = table([borrowed_bigint::<TestScalar>(
            "unused",
            [0_i64, 1, 2],
            &alloc,
        )]);
        let expr = SubtractExpr::try_new(
            Box::new(literal(LiteralValue::BigInt(10))),
            Box::new(literal(LiteralValue::Int(3))),
        )
        .unwrap();
        let expected = [TestScalar::from(7_i64); 3];

        let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert!(matches!(
            first_round,
            Column::Decimal75(precision, 0, values)
                if precision == Precision::new(20).unwrap() && values == expected
        ));

        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut builder, &alloc, &table, &[])
            .unwrap();
        assert!(matches!(
            final_round,
            Column::Decimal75(precision, 0, values)
                if precision == Precision::new(20).unwrap() && values == expected
        ));
    }

    #[test]
    fn subtract_expr_verifier_evaluation_subtracts_rhs_from_lhs() {
        let expr = SubtractExpr::try_new(
            Box::new(literal(LiteralValue::BigInt(10))),
            Box::new(literal(LiteralValue::Int(3))),
        )
        .unwrap();
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let accessor = IndexMap::<Ident, TestScalar>::default();

        let eval = expr
            .verifier_evaluate(&mut builder, &accessor, TestScalar::from(5_i64), &[])
            .unwrap();

        assert_eq!(eval, TestScalar::from(35_i64));
    }

    #[test]
    fn subtract_expr_collects_column_references_from_both_sides() {
        let table_ref = TableRef::new("sxt", "t");
        let lhs = ColumnRef::new(table_ref.clone(), "a".into(), ColumnType::BigInt);
        let rhs = ColumnRef::new(table_ref, "b".into(), ColumnType::BigInt);
        let expr = SubtractExpr::try_new(
            Box::new(DynProofExpr::new_column(lhs.clone())),
            Box::new(DynProofExpr::new_column(rhs.clone())),
        )
        .unwrap();
        let mut columns = IndexSet::<ColumnRef>::default();

        expr.get_column_references(&mut columns);

        assert!(columns.contains(&lhs));
        assert!(columns.contains(&rhs));
    }
}
