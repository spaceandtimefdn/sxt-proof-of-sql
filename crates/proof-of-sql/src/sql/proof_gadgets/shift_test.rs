//! This module contains tests for the shift gadget. The gadget proves that a
//! candidate column of length `n + 1` is the downward shift of a column of
//! length `n`, i.e. that the candidate is the column prepended with a zero.
use super::shift::{final_round_evaluate_shift, first_round_evaluate_shift, verify_shift};
use crate::{
    base::{
        polynomial::MultilinearExtension,
        proof::{ProofError, ProofSizeMismatch},
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::proof::{
        mock_verification_builder::{run_verify_for_each_row, MockVerificationBuilder},
        FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomial, SumcheckSubpolynomialType,
    },
};
use bumpalo::Bump;
use num_traits::Inv;
use std::collections::VecDeque;

/// Returns a one-hot evaluation point which reads the `index`-th entry of an MLE.
fn one_hot(index: usize, length: usize) -> Vec<TestScalar> {
    let mut evaluation_point = vec![TestScalar::ZERO; length];
    evaluation_point[index] = TestScalar::ONE;
    evaluation_point
}

/// Runs `verify_shift` against a mock verification builder and returns the result.
fn try_verify_shift_with_mock(
    subpolynomial_max_multiplicands: usize,
    first_round_mles: Vec<Vec<TestScalar>>,
    final_round_mles: Vec<Vec<TestScalar>>,
    chi_evaluation_length_queue: Vec<usize>,
    rho_evaluation_length_queue: Vec<usize>,
) -> Result<(TestScalar, TestScalar), ProofError> {
    let mut verification_builder = MockVerificationBuilder::new(
        Vec::new(),
        subpolynomial_max_multiplicands,
        first_round_mles,
        final_round_mles,
        Vec::new(),
        chi_evaluation_length_queue,
        rho_evaluation_length_queue,
    );
    verify_shift(
        &mut verification_builder,
        TestScalar::TWO,
        TestScalar::TEN,
        TestScalar::ZERO,
        TestScalar::ONE,
    )
}

#[test]
fn we_can_produce_the_shifted_column_and_evaluation_lengths_in_the_first_round() {
    let alloc = Bump::new();
    let column = alloc.alloc_slice_copy(&[
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ]) as &[_];
    let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    first_round_evaluate_shift(&mut first_round_builder, &alloc, column);
    // The shifted column is the original column prepended with a zero.
    let expected_shifted_column = [
        TestScalar::ZERO,
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ];
    assert_eq!(first_round_builder.range_length(), 4);
    assert_eq!(first_round_builder.chi_evaluation_lengths(), &[4]);
    assert_eq!(first_round_builder.rho_evaluation_lengths(), &[3, 4]);
    assert_eq!(first_round_builder.pcs_proof_mles().len(), 1);
    for (index, expected) in expected_shifted_column.iter().enumerate() {
        assert_eq!(
            first_round_builder.evaluate_pcs_proof_mles(&one_hot(index, 4)),
            vec![*expected]
        );
    }
}

#[expect(clippy::similar_names)]
#[test]
fn we_can_produce_the_star_columns_and_subpolynomials_in_the_final_round() {
    let alloc = Bump::new();
    let column = alloc.alloc_slice_copy(&[
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ]) as &[_];
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    let shifted_column = final_round_evaluate_shift(
        &mut final_round_builder,
        &alloc,
        TestScalar::TWO,
        TestScalar::TEN,
        column,
    );
    assert_eq!(
        shifted_column,
        [
            TestScalar::ZERO,
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3)
        ]
    );
    // With alpha = 2 and beta = 10 the fold of rho + chi_n and the column is
    // c_fold = alpha * (beta * (rho + chi_n) + column) = [22, 44, 66, 0] and the
    // fold of rho_{n + 1} and the shifted column is
    // d_fold = alpha * (beta * rho_{n + 1} + shifted_column) = [0, 22, 44, 66].
    // The star columns are the entrywise inverses of one plus the folds.
    let expected_c_star = [
        TestScalar::from(23).inv().unwrap(),
        TestScalar::from(45).inv().unwrap(),
        TestScalar::from(67).inv().unwrap(),
        TestScalar::ONE,
    ];
    let expected_d_star = [
        TestScalar::ONE,
        TestScalar::from(23).inv().unwrap(),
        TestScalar::from(45).inv().unwrap(),
        TestScalar::from(67).inv().unwrap(),
    ];
    assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
    for (index, (expected_c, expected_d)) in expected_c_star
        .iter()
        .zip(expected_d_star.iter())
        .enumerate()
    {
        assert_eq!(
            final_round_builder.evaluate_pcs_proof_mles(&one_hot(index, 4)),
            vec![*expected_c, *expected_d]
        );
    }
    assert_eq!(
        final_round_builder
            .sumcheck_subpolynomials()
            .iter()
            .map(SumcheckSubpolynomial::subpolynomial_type)
            .collect::<Vec<_>>(),
        vec![
            SumcheckSubpolynomialType::ZeroSum,
            SumcheckSubpolynomialType::Identity,
            SumcheckSubpolynomialType::Identity
        ]
    );
}

#[test]
fn we_can_verify_shift() {
    let alloc = Bump::new();
    let column = alloc.alloc_slice_copy(&[
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ]) as &[_];
    let alpha = TestScalar::TWO;
    let beta = TestScalar::TEN;
    let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    first_round_evaluate_shift(&mut first_round_builder, &alloc, column);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    let shifted_column =
        final_round_evaluate_shift(&mut final_round_builder, &alloc, alpha, beta, column);
    let verification_builder = run_verify_for_each_row(
        3,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            let (shifted_column_eval, chi_n_plus_1_eval) = verify_shift(
                verification_builder,
                alpha,
                beta,
                column.inner_product(evaluation_point),
                chi_n_eval,
            )
            .unwrap();
            assert_eq!(
                shifted_column_eval,
                shifted_column.inner_product(evaluation_point)
            );
            assert_eq!(chi_n_plus_1_eval, TestScalar::ONE);
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true; 2]; 4]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![true]);
}

#[test]
fn we_can_verify_shift_for_an_empty_column() {
    let alloc = Bump::new();
    let column: &[TestScalar] = &[];
    let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(1);
    first_round_evaluate_shift(&mut first_round_builder, &alloc, column);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(1, VecDeque::new());
    let shifted_column = final_round_evaluate_shift(
        &mut final_round_builder,
        &alloc,
        TestScalar::TWO,
        TestScalar::TEN,
        column,
    );
    assert_eq!(shifted_column, [TestScalar::ZERO]);
    let verification_builder = run_verify_for_each_row(
        0,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            verify_shift(
                verification_builder,
                TestScalar::TWO,
                TestScalar::TEN,
                column.inner_product(evaluation_point),
                chi_n_eval,
            )
            .unwrap();
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true; 2]]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![true]);
}

#[test]
fn we_cannot_verify_shift_if_the_candidate_is_not_the_shifted_column() {
    let alloc = Bump::new();
    let column = alloc.alloc_slice_copy(&[
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ]) as &[_];
    // The candidate shifted column should be [0, 1, 2, 3], but the first entry is wrong.
    let candidate_shifted_column = alloc.alloc_slice_copy(&[
        TestScalar::from(2),
        TestScalar::from(1),
        TestScalar::from(2),
        TestScalar::from(3),
    ]) as &[_];
    let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    first_round_builder.produce_intermediate_mle(candidate_shifted_column);
    first_round_builder.produce_chi_evaluation_length(4);
    first_round_builder.produce_rho_evaluation_length(3);
    first_round_builder.produce_rho_evaluation_length(4);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    final_round_evaluate_shift(
        &mut final_round_builder,
        &alloc,
        TestScalar::TWO,
        TestScalar::TEN,
        column,
    );
    let verification_builder = run_verify_for_each_row(
        3,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            verify_shift(
                verification_builder,
                TestScalar::TWO,
                TestScalar::TEN,
                column.inner_product(evaluation_point),
                chi_n_eval,
            )
            .unwrap();
        },
    );
    // The constraint involving the candidate column fails exactly at the wrong entry.
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![
            vec![true, false],
            vec![true, true],
            vec![true, true],
            vec![true, true]
        ]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![true]);
}

#[test]
fn we_cannot_verify_shift_with_too_few_chi_lengths() {
    let err = try_verify_shift_with_mock(
        3,
        vec![vec![TestScalar::ZERO]],
        vec![vec![TestScalar::ZERO, TestScalar::ZERO]],
        vec![],
        vec![3, 4],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::TooFewChiLengths
        }
    ));
}

#[test]
fn we_cannot_verify_shift_with_too_few_first_round_mle_evaluations() {
    let err = try_verify_shift_with_mock(
        3,
        vec![vec![]],
        vec![vec![TestScalar::ZERO, TestScalar::ZERO]],
        vec![4],
        vec![3, 4],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::TooFewMLEEvaluations
        }
    ));
}

#[test]
fn we_cannot_verify_shift_with_too_few_rho_lengths() {
    let err = try_verify_shift_with_mock(
        3,
        vec![vec![TestScalar::ZERO]],
        vec![vec![TestScalar::ZERO, TestScalar::ZERO]],
        vec![4],
        vec![],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::TooFewRhoLengths
        }
    ));
}

#[test]
fn we_cannot_verify_shift_with_too_few_final_round_mle_evaluations() {
    let err = try_verify_shift_with_mock(
        3,
        vec![vec![TestScalar::ZERO]],
        vec![vec![]],
        vec![4],
        vec![3, 4],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::TooFewMLEEvaluations
        }
    ));
}

#[test]
fn we_cannot_verify_shift_if_the_sumcheck_proof_cannot_hold_the_zerosum_constraint() {
    let err = try_verify_shift_with_mock(
        0,
        vec![vec![TestScalar::ZERO]],
        vec![vec![TestScalar::ZERO, TestScalar::ZERO]],
        vec![4],
        vec![3, 4],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::SumcheckProofTooSmall
        }
    ));
}

#[test]
fn we_cannot_verify_shift_if_the_sumcheck_proof_cannot_hold_the_identity_constraints() {
    let err = try_verify_shift_with_mock(
        1,
        vec![vec![TestScalar::ZERO]],
        vec![vec![TestScalar::ZERO, TestScalar::ZERO]],
        vec![4],
        vec![3, 4],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::SumcheckProofTooSmall
        }
    ));
}
