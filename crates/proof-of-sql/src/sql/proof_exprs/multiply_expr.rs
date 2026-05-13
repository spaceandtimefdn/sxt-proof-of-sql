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
            database::{table_utility::*, TableRef},
            math::decimal::Precision,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec, vec::Vec};

    fn literal(value: LiteralValue) -> Box<DynProofExpr> {
        Box::new(DynProofExpr::new_literal(value))
    }

    fn column_ref(name: &str) -> ColumnRef {
        ColumnRef::new(TableRef::new("sxt", "t"), name.into(), ColumnType::BigInt)
    }

    fn column(name: &str) -> Box<DynProofExpr> {
        Box::new(DynProofExpr::new_column(column_ref(name)))
    }

    fn scalar_values(values: impl IntoIterator<Item = u64>) -> Vec<TestScalar> {
        values.into_iter().map(TestScalar::from).collect()
    }

    fn assert_decimal_column(
        column: Column<TestScalar>,
        precision: Precision,
        scale: i8,
        expected: impl IntoIterator<Item = u64>,
    ) {
        match column {
            Column::Decimal75(actual_precision, actual_scale, values) => {
                assert_eq!(actual_precision, precision);
                assert_eq!(actual_scale, scale);
                assert_eq!(values, scalar_values(expected).as_slice());
            }
            other => panic!("expected decimal column, got {other:?}"),
        }
    }

    #[test]
    fn we_can_create_a_multiply_expr_and_read_its_metadata() {
        let expr = MultiplyExpr::try_new(
            literal(LiteralValue::SmallInt(2)),
            literal(LiteralValue::Int(3)),
        )
        .unwrap();

        assert!(matches!(expr.lhs(), DynProofExpr::Literal(_)));
        assert!(matches!(expr.rhs(), DynProofExpr::Literal(_)));
        assert_eq!(
            expr.data_type(),
            ColumnType::Decimal75(Precision::new(16).unwrap(), 0)
        );
    }

    #[test]
    fn we_cannot_create_a_multiply_expr_for_non_numeric_types() {
        let error = MultiplyExpr::try_new(
            literal(LiteralValue::VarChar("abc".to_string())),
            literal(LiteralValue::BigInt(3)),
        )
        .unwrap_err();

        assert!(matches!(error, AnalyzeError::DataTypeMismatch { .. }));
    }

    #[test]
    fn we_can_evaluate_multiply_expr_in_prover_rounds() {
        let alloc = Bump::new();
        let table = table([
            borrowed_bigint("a", [2_i64, 3, 4], &alloc),
            borrowed_bigint("b", [5_i64, 6, 7], &alloc),
        ]);
        let expr = MultiplyExpr::try_new(column("a"), column("b")).unwrap();

        let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_decimal_column(first_round, expr.precision(), expr.scale(), [10, 18, 28]);

        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut builder, &alloc, &table, &[])
            .unwrap();
        assert_decimal_column(final_round, expr.precision(), expr.scale(), [10, 18, 28]);
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }

    #[test]
    fn we_can_verify_multiply_expr_and_record_identity_constraint() {
        let expr = MultiplyExpr::try_new(
            literal(LiteralValue::BigInt(2)),
            literal(LiteralValue::BigInt(3)),
        )
        .unwrap();
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            vec![scalar_values([6])],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let result = expr
            .verifier_evaluate(&mut builder, &IndexMap::default(), TestScalar::ONE, &[])
            .unwrap();

        assert_eq!(result, TestScalar::from(6_u64));
        assert_eq!(builder.get_identity_results(), vec![vec![true]]);
    }

    #[test]
    fn we_collect_column_references_from_both_operands() {
        let left_ref = column_ref("a");
        let right_ref = column_ref("b");
        let expr = MultiplyExpr::try_new(
            Box::new(DynProofExpr::new_column(left_ref.clone())),
            Box::new(DynProofExpr::new_column(right_ref.clone())),
        )
        .unwrap();
        let mut refs = IndexSet::default();

        expr.get_column_references(&mut refs);

        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&left_ref));
        assert!(refs.contains(&right_ref));
    }
}
