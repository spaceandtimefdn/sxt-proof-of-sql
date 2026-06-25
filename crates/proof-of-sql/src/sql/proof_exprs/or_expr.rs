use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{can_and_or_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable logical OR expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl OrExpr {
    /// Create logical OR expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        can_and_or_types(left_datatype, right_datatype)
            .then_some(Self { lhs, rhs })
            .ok_or_else(|| AnalyzeError::DataTypeMismatch {
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

impl ProofExpr for OrExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "OrExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result = Column::Boolean(first_round_evaluate_or(table.num_rows(), alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(result)
    }

    #[tracing::instrument(name = "OrExpr::final_round_evaluate", level = "debug", skip_all)]
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
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result = Column::Boolean(final_round_evaluate_or(builder, alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(result)
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

        verifier_evaluate_or(builder, &lhs, &rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[expect(
    clippy::missing_panics_doc,
    reason = "table_length matches lhs and rhs lengths, ensuring no panic occurs"
)]
pub fn first_round_evaluate_or<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &[bool],
    rhs: &[bool],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    assert_eq!(table_length, rhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] || rhs[i])
}

#[expect(
    clippy::missing_panics_doc,
    reason = "lhs and rhs are guaranteed to have the same length, ensuring no panic occurs"
)]
pub fn final_round_evaluate_or<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    let n = lhs.len();
    assert_eq!(n, rhs.len());

    // lhs_and_rhs
    let lhs_and_rhs: &[_] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
    builder.produce_intermediate_mle(lhs_and_rhs);

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(lhs_and_rhs)]),
            (-S::one(), vec![Box::new(lhs), Box::new(rhs)]),
        ],
    );

    // selection
    alloc.alloc_slice_fill_with(n, |i| lhs[i] || rhs[i])
}

pub fn verifier_evaluate_or<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs: &S,
    rhs: &S,
) -> Result<S, ProofError> {
    // lhs_and_rhs
    let lhs_and_rhs = builder.try_consume_final_round_mle_evaluation()?;

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        lhs_and_rhs - *lhs * *rhs,
        2,
    )?;

    // selection
    Ok(*lhs + *rhs - lhs_and_rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{Column, TableOptions},
            map::IndexMap,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::collections::VecDeque;

    fn empty_table(row_count: usize) -> Table<'static, TestScalar> {
        Table::try_new_with_options(IndexMap::default(), TableOptions::new(Some(row_count)))
            .unwrap()
    }

    fn bool_expr(value: bool) -> Box<DynProofExpr> {
        Box::new(DynProofExpr::new_literal(LiteralValue::Boolean(value)))
    }

    #[test]
    fn try_new_accepts_boolean_inputs_and_exposes_children() {
        let lhs = bool_expr(true);
        let rhs = bool_expr(false);
        let expr = OrExpr::try_new(lhs.clone(), rhs.clone()).unwrap();

        assert_eq!(expr.data_type(), ColumnType::Boolean);
        assert_eq!(expr.lhs(), lhs.as_ref());
        assert_eq!(expr.rhs(), rhs.as_ref());
    }

    #[test]
    fn first_round_evaluate_or_covers_all_truth_table_rows() {
        let alloc = Bump::new();
        let lhs = [false, true, false, true];
        let rhs = [false, false, true, true];

        assert_eq!(
            first_round_evaluate_or(4, &alloc, &lhs, &rhs),
            &[false, true, true, true]
        );
    }

    #[test]
    fn first_and_final_round_evaluate_boolean_literals() {
        let alloc = Bump::new();
        let table = empty_table(2);
        let expr = OrExpr::try_new(bool_expr(false), bool_expr(true)).unwrap();

        let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round, Column::Boolean(&[true, true]));

        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, Column::Boolean(&[true, true]));
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }

    #[test]
    fn verifier_evaluate_or_combines_inputs_and_checks_identity() {
        let chi_eval = TestScalar::from(7u64);
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            vec![vec![TestScalar::ZERO]],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let expr = OrExpr::try_new(bool_expr(true), bool_expr(false)).unwrap();

        let result = expr
            .verifier_evaluate(&mut builder, &IndexMap::default(), chi_eval, &[])
            .unwrap();

        assert_eq!(result, chi_eval);
        assert_eq!(
            builder.identity_subpolynomial_evaluations,
            vec![vec![TestScalar::ZERO]]
        );
    }
}
