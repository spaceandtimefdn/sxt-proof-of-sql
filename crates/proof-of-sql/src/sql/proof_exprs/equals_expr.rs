use super::{add_subtract_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_equals_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
        slice_ops,
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

/// Provable AST expression for an equals expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EqualsExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl EqualsExpr {
    /// Create a new equals expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_equals_types(left_datatype, right_datatype)
            .map(|()| Self { lhs, rhs })
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

impl ProofExpr for EqualsExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "EqualsExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column = self.rhs.first_round_evaluate(alloc, table, params)?;
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, true);
        let res = Column::Boolean(first_round_evaluate_equals_zero(
            table.num_rows(),
            alloc,
            res,
        ));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "EqualsExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let scale_and_subtract_res = add_subtract_columns(lhs_column, rhs_column, alloc, true);
        let res = Column::Boolean(final_round_evaluate_equals_zero(
            table.num_rows(),
            builder,
            alloc,
            scale_and_subtract_res,
        ));

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
        let lhs_eval = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs_eval = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        verifier_evaluate_equals_zero(builder, lhs_eval - rhs_eval, chi_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[tracing::instrument(
    name = "EqualsExpr::first_round_evaluate_equals_zero",
    level = "debug",
    skip_all
)]
pub fn first_round_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] == S::zero())
}

#[tracing::instrument(
    name = "EqualsExpr::final_round_evaluate_equals_zero",
    level = "debug",
    skip_all
)]
pub fn final_round_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    // lhs_pseudo_inv
    let lhs_pseudo_inv = alloc.alloc_slice_copy(lhs);
    slice_ops::batch_inversion(lhs_pseudo_inv);

    builder.produce_intermediate_mle(lhs_pseudo_inv as &[_]);

    // selection_not
    let selection_not: &[_] = alloc.alloc_slice_fill_with(table_length, |i| lhs[i] != S::zero());

    // selection
    let selection: &[_] = alloc.alloc_slice_fill_with(table_length, |i| !selection_not[i]);
    builder.produce_intermediate_mle(selection);

    // subpolynomial: selection * lhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(S::one(), vec![Box::new(lhs), Box::new(selection)])],
    );

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(selection_not)]),
            (
                -S::one(),
                vec![Box::new(lhs), Box::new(lhs_pseudo_inv as &[_])],
            ),
        ],
    );

    selection
}

pub fn verifier_evaluate_equals_zero<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs_eval: S,
    chi_eval: S,
) -> Result<S, ProofError> {
    // consume mle evaluations
    let lhs_pseudo_inv_eval = builder.try_consume_final_round_mle_evaluation()?;
    let selection_eval = builder.try_consume_final_round_mle_evaluation()?;
    let selection_not_eval = chi_eval - selection_eval;

    // subpolynomial: selection * lhs
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        selection_eval * lhs_eval,
        2,
    )?;

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        selection_not_eval - lhs_eval * lhs_pseudo_inv_eval,
        2,
    )?;

    Ok(selection_eval)
}

#[cfg(test)]
mod tests {
    use super::{
        final_round_evaluate_equals_zero, first_round_evaluate_equals_zero,
        verifier_evaluate_equals_zero, EqualsExpr,
    };
    use crate::{
        base::{
            database::{Column, ColumnType, LiteralValue, Table, TableOptions},
            map::IndexMap,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::{
            proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
            proof_exprs::{DynProofExpr, ProofExpr},
            AnalyzeError,
        },
    };
    use alloc::{collections::VecDeque, string::ToString, vec, vec::Vec};
    use bumpalo::Bump;

    fn unit_vectors(len: usize) -> Vec<Vec<TestScalar>> {
        (0..len)
            .map(|i| {
                (0..len)
                    .map(|j| {
                        if i == j {
                            TestScalar::ONE
                        } else {
                            TestScalar::ZERO
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn we_can_construct_equals_expr_and_read_operands() {
        let lhs = DynProofExpr::new_literal(LiteralValue::BigInt(7));
        let rhs = DynProofExpr::new_literal(LiteralValue::Int(7));

        let expr = EqualsExpr::try_new(Box::new(lhs), Box::new(rhs)).unwrap();

        assert_eq!(expr.data_type(), ColumnType::Boolean);
        assert_eq!(expr.lhs().data_type(), ColumnType::BigInt);
        assert_eq!(expr.rhs().data_type(), ColumnType::Int);
    }

    #[test]
    fn we_cannot_construct_equals_expr_for_incompatible_types() {
        let lhs = DynProofExpr::new_literal(LiteralValue::VarChar("abc".to_string()));
        let rhs = DynProofExpr::new_literal(LiteralValue::BigInt(7));

        let err = EqualsExpr::try_new(Box::new(lhs), Box::new(rhs)).unwrap_err();

        assert!(matches!(
            err,
            AnalyzeError::DataTypeMismatch {
                left_type: _,
                right_type: _
            }
        ));
    }

    #[test]
    fn first_round_equals_zero_marks_only_zero_entries() {
        let alloc = Bump::new();
        let lhs = alloc.alloc_slice_copy(&[
            TestScalar::ZERO,
            TestScalar::from(3_u64),
            -TestScalar::TWO,
            TestScalar::ZERO,
        ]);

        let selection = first_round_evaluate_equals_zero(lhs.len(), &alloc, lhs);

        assert_eq!(selection, &[true, false, false, true]);
    }

    #[test]
    fn full_equals_expr_evaluation_rejects_unequal_literals() {
        let alloc = Bump::new();
        let table = Table::<TestScalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions::new(Some(4)),
        )
        .unwrap();
        let expr = EqualsExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(7))),
            Box::new(DynProofExpr::new_literal(LiteralValue::Int(3))),
        )
        .unwrap();

        let first_round_result = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(
            first_round_result,
            Column::Boolean(&[false, false, false, false])
        );

        let mut final_round_builder =
            FinalRoundBuilder::<TestScalar>::new(table.num_rows(), VecDeque::new());
        let final_round_result = expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(
            final_round_result,
            Column::Boolean(&[false, false, false, false])
        );

        let final_round_mles: Vec<_> = unit_vectors(table.num_rows())
            .iter()
            .map(|evaluation_point| final_round_builder.evaluate_pcs_proof_mles(evaluation_point))
            .collect();
        let mut verification_builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            final_round_mles,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        for _ in 0..table.num_rows() {
            let actual = expr
                .verifier_evaluate(
                    &mut verification_builder,
                    &IndexMap::default(),
                    TestScalar::ONE,
                    &[],
                )
                .unwrap();
            assert_eq!(actual, TestScalar::ZERO);
            verification_builder.increment_row_index();
        }

        assert_eq!(
            verification_builder.get_identity_results(),
            vec![vec![true, true]; table.num_rows()]
        );
    }

    #[test]
    fn final_and_verifier_equals_zero_constraints_match() {
        let alloc = Bump::new();
        let lhs = alloc.alloc_slice_copy(&[
            TestScalar::ZERO,
            TestScalar::from(3_u64),
            -TestScalar::TWO,
            TestScalar::ZERO,
        ]);
        let mut final_round_builder =
            FinalRoundBuilder::<TestScalar>::new(lhs.len(), VecDeque::new());

        let selection =
            final_round_evaluate_equals_zero(lhs.len(), &mut final_round_builder, &alloc, lhs);

        assert_eq!(selection, &[true, false, false, true]);
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 2);

        let final_round_mles: Vec<_> = unit_vectors(lhs.len())
            .iter()
            .map(|evaluation_point| final_round_builder.evaluate_pcs_proof_mles(evaluation_point))
            .collect();
        let mut verification_builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            final_round_mles,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let expected_selection = [
            TestScalar::ONE,
            TestScalar::ZERO,
            TestScalar::ZERO,
            TestScalar::ONE,
        ];

        for (lhs_eval, expected) in lhs.iter().copied().zip(expected_selection) {
            let actual =
                verifier_evaluate_equals_zero(&mut verification_builder, lhs_eval, TestScalar::ONE)
                    .unwrap();
            assert_eq!(actual, expected);
            verification_builder.increment_row_index();
        }

        assert_eq!(
            verification_builder.get_identity_results(),
            vec![vec![true, true]; lhs.len()]
        );
    }
}
