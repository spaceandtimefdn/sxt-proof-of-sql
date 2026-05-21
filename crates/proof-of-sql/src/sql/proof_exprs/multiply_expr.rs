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
                table_utility::{borrowed_int, table},
                Column, ColumnRef, ColumnType, TableRef,
            },
            map::{IndexMap, IndexSet},
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::{
            proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
            proof_exprs::test_utility::{const_bigint, const_int, const_varchar},
            AnalyzeError,
        },
    };
    use alloc::{boxed::Box, collections::VecDeque, vec};
    use bumpalo::Bump;

    fn literal_multiply_expr() -> MultiplyExpr {
        MultiplyExpr::try_new(Box::new(const_int(3)), Box::new(const_bigint(4))).unwrap()
    }

    #[test]
    fn we_can_inspect_multiply_expr_inputs_and_references() {
        let table_ref = TableRef::new("sxt", "orders");
        let quantity_ref = ColumnRef::new(table_ref.clone(), "quantity".into(), ColumnType::Int);
        let price_ref = ColumnRef::new(table_ref, "price".into(), ColumnType::BigInt);
        let expr = MultiplyExpr::try_new(
            Box::new(DynProofExpr::new_column(quantity_ref.clone())),
            Box::new(DynProofExpr::new_column(price_ref.clone())),
        )
        .unwrap();

        assert_eq!(expr.lhs().data_type(), ColumnType::Int);
        assert_eq!(expr.rhs().data_type(), ColumnType::BigInt);
        assert!(matches!(expr.data_type(), ColumnType::Decimal75(_, 0)));

        let mut columns = IndexSet::default();
        expr.get_column_references(&mut columns);
        assert_eq!(columns.len(), 2);
        assert!(columns.contains(&quantity_ref));
        assert!(columns.contains(&price_ref));
    }

    #[test]
    fn we_cannot_build_multiply_expr_for_non_numeric_inputs() {
        let err = MultiplyExpr::try_new(Box::new(const_int(3)), Box::new(const_varchar("x")))
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
    fn we_can_evaluate_literal_multiply_expr_rounds() {
        let alloc = Bump::new();
        let data = table([borrowed_int("rows", [0, 0], &alloc)]);
        let expr = literal_multiply_expr();
        let expected_values = [TestScalar::from(12_u64), TestScalar::from(12_u64)];
        let expected = Column::Decimal75(expr.precision(), expr.scale(), &expected_values);

        assert_eq!(
            expr.first_round_evaluate(&alloc, &data, &[]).unwrap(),
            expected
        );

        let mut builder = FinalRoundBuilder::<TestScalar>::new(1, VecDeque::new());
        assert_eq!(
            expr.final_round_evaluate(&mut builder, &alloc, &data, &[])
                .unwrap(),
            expected
        );
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
        assert_eq!(
            builder.evaluate_pcs_proof_mles(&[TestScalar::ONE, TestScalar::ZERO]),
            [TestScalar::from(12_u64)]
        );
    }

    #[test]
    fn we_can_verify_literal_multiply_expr_evaluation() {
        let expr = literal_multiply_expr();
        let mut builder = MockVerificationBuilder::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(12_u64)]],
            vec![],
            vec![],
            vec![],
        );
        let accessor = IndexMap::default();

        assert_eq!(
            expr.verifier_evaluate(&mut builder, &accessor, TestScalar::ONE, &[])
                .unwrap(),
            TestScalar::from(12_u64)
        );
        assert_eq!(builder.get_identity_results(), vec![vec![true]]);
    }
}
