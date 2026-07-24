//! This module contains tests for the filter gadget. The gadget proves that the
//! filtered columns consist exactly of the rows of the input columns picked by
//! the selection column.
use super::filter_base::{final_round_evaluate_filter, verify_evaluate_filter};
use crate::{
    base::{
        database::table_utility::borrowed_bigint,
        polynomial::MultilinearExtension,
        proof::{ProofError, ProofSizeMismatch},
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::{
        proof::{
            mock_verification_builder::{run_verify_for_each_row, MockVerificationBuilder},
            FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomial, SumcheckSubpolynomialType,
        },
        proof_plans::fold_vals,
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

/// Runs `verify_evaluate_filter` against a mock verification builder and returns the result.
fn try_verify_filter_with_mock(
    subpolynomial_max_multiplicands: usize,
    final_round_mles: Vec<Vec<TestScalar>>,
) -> Result<(), ProofError> {
    let mut verification_builder = MockVerificationBuilder::new(
        Vec::new(),
        subpolynomial_max_multiplicands,
        Vec::new(),
        final_round_mles,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    verify_evaluate_filter(
        &mut verification_builder,
        TestScalar::ZERO,
        TestScalar::ZERO,
        TestScalar::ONE,
        TestScalar::ONE,
        TestScalar::ONE,
    )
}

#[expect(clippy::similar_names)]
#[test]
fn we_can_produce_the_star_columns_and_subpolynomials_in_the_final_round() {
    let alloc = Bump::new();
    let column_a = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
    let column_b = borrowed_bigint::<TestScalar>("b", [4, 5, 6], &alloc).1;
    let filtered_a = borrowed_bigint::<TestScalar>("a", [1, 3], &alloc).1;
    let filtered_b = borrowed_bigint::<TestScalar>("b", [4, 6], &alloc).1;
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    final_round_evaluate_filter(
        &mut final_round_builder,
        &alloc,
        TestScalar::TWO,
        TestScalar::TEN,
        &[column_a, column_b],
        &[true, false, true],
        &[filtered_a, filtered_b],
        3,
        2,
    );
    // With alpha = 2 and beta = 10 the folds are
    // c_fold = alpha * (beta * a + b) = [28, 50, 72] and
    // d_fold = alpha * (beta * filtered_a + filtered_b) = [28, 72], and the star
    // columns are the entrywise inverses of one plus the folds.
    let expected_c_star = [
        TestScalar::from(29).inv().unwrap(),
        TestScalar::from(51).inv().unwrap(),
        TestScalar::from(73).inv().unwrap(),
    ];
    // d_star only has output_length entries, so it is padded with zeros.
    let expected_d_star = [
        TestScalar::from(29).inv().unwrap(),
        TestScalar::from(73).inv().unwrap(),
        TestScalar::ZERO,
    ];
    assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
    for (index, (expected_c, expected_d)) in expected_c_star
        .iter()
        .zip(expected_d_star.iter())
        .enumerate()
    {
        assert_eq!(
            final_round_builder.evaluate_pcs_proof_mles(&one_hot(index, 3)),
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
            SumcheckSubpolynomialType::Identity,
            SumcheckSubpolynomialType::Identity,
            SumcheckSubpolynomialType::ZeroSum,
            SumcheckSubpolynomialType::Identity
        ]
    );
}

#[test]
fn we_can_verify_filter() {
    let alloc = Bump::new();
    let column_a = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
    let column_b = borrowed_bigint::<TestScalar>("b", [4, 5, 6], &alloc).1;
    let filtered_a = borrowed_bigint::<TestScalar>("a", [1, 3], &alloc).1;
    let filtered_b = borrowed_bigint::<TestScalar>("b", [4, 6], &alloc).1;
    let alpha = TestScalar::TWO;
    let beta = TestScalar::TEN;
    let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    final_round_evaluate_filter(
        &mut final_round_builder,
        &alloc,
        alpha,
        beta,
        &[column_a, column_b],
        &[true, false, true],
        &[filtered_a, filtered_b],
        3,
        2,
    );
    let verification_builder = run_verify_for_each_row(
        3,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            let c_fold_eval = alpha
                * fold_vals(
                    beta,
                    &[
                        column_a.inner_product(evaluation_point),
                        column_b.inner_product(evaluation_point),
                    ],
                );
            let d_fold_eval = alpha
                * fold_vals(
                    beta,
                    &[
                        filtered_a.inner_product(evaluation_point),
                        filtered_b.inner_product(evaluation_point),
                    ],
                );
            verify_evaluate_filter(
                verification_builder,
                c_fold_eval,
                d_fold_eval,
                chi_n_eval,
                (&[true, true]).inner_product(evaluation_point),
                (&[true, false, true]).inner_product(evaluation_point),
            )
            .unwrap();
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true; 3]; 3]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![true]);
}

#[test]
fn we_can_verify_filter_when_no_rows_are_selected() {
    let alloc = Bump::new();
    let column_a = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
    let filtered_a = borrowed_bigint::<TestScalar>("a", [0_i64; 0], &alloc).1;
    let alpha = TestScalar::TWO;
    let beta = TestScalar::TEN;
    let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    final_round_evaluate_filter(
        &mut final_round_builder,
        &alloc,
        alpha,
        beta,
        &[column_a],
        &[false, false, false],
        &[filtered_a],
        3,
        0,
    );
    let verification_builder = run_verify_for_each_row(
        3,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            let c_fold_eval = alpha * fold_vals(beta, &[column_a.inner_product(evaluation_point)]);
            let d_fold_eval =
                alpha * fold_vals(beta, &[filtered_a.inner_product(evaluation_point)]);
            // The output is empty, so chi_m evaluates to zero.
            verify_evaluate_filter(
                verification_builder,
                c_fold_eval,
                d_fold_eval,
                chi_n_eval,
                TestScalar::ZERO,
                (&[false, false, false]).inner_product(evaluation_point),
            )
            .unwrap();
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true; 3]; 3]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![true]);
}

#[test]
fn we_cannot_verify_filter_if_the_filtered_columns_are_wrong() {
    let alloc = Bump::new();
    let column_a = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
    let column_b = borrowed_bigint::<TestScalar>("b", [4, 5, 6], &alloc).1;
    // The second row is not selected, so the filtered columns should be ([1, 3], [4, 6]).
    let filtered_a = borrowed_bigint::<TestScalar>("a", [1, 2], &alloc).1;
    let filtered_b = borrowed_bigint::<TestScalar>("b", [4, 5], &alloc).1;
    let alpha = TestScalar::TWO;
    let beta = TestScalar::TEN;
    let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    final_round_evaluate_filter(
        &mut final_round_builder,
        &alloc,
        alpha,
        beta,
        &[column_a, column_b],
        &[true, false, true],
        &[filtered_a, filtered_b],
        3,
        2,
    );
    let verification_builder = run_verify_for_each_row(
        3,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            let c_fold_eval = alpha
                * fold_vals(
                    beta,
                    &[
                        column_a.inner_product(evaluation_point),
                        column_b.inner_product(evaluation_point),
                    ],
                );
            let d_fold_eval = alpha
                * fold_vals(
                    beta,
                    &[
                        filtered_a.inner_product(evaluation_point),
                        filtered_b.inner_product(evaluation_point),
                    ],
                );
            verify_evaluate_filter(
                verification_builder,
                c_fold_eval,
                d_fold_eval,
                chi_n_eval,
                (&[true, true]).inner_product(evaluation_point),
                (&[true, false, true]).inner_product(evaluation_point),
            )
            .unwrap();
        },
    );
    // The identity constraints hold because the star columns are self-consistent,
    // but the zero sum constraint catches the wrong filtered rows.
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true; 3]; 3]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![false]);
}

#[test]
fn we_cannot_verify_filter_if_the_claimed_output_length_is_wrong() {
    let alloc = Bump::new();
    let column_a = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
    let column_b = borrowed_bigint::<TestScalar>("b", [4, 5, 6], &alloc).1;
    let filtered_a = borrowed_bigint::<TestScalar>("a", [1, 3], &alloc).1;
    let filtered_b = borrowed_bigint::<TestScalar>("b", [4, 6], &alloc).1;
    let alpha = TestScalar::TWO;
    let beta = TestScalar::TEN;
    let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(3, VecDeque::new());
    final_round_evaluate_filter(
        &mut final_round_builder,
        &alloc,
        alpha,
        beta,
        &[column_a, column_b],
        &[true, false, true],
        &[filtered_a, filtered_b],
        3,
        2,
    );
    let verification_builder = run_verify_for_each_row(
        3,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_n_eval, evaluation_point| {
            let c_fold_eval = alpha
                * fold_vals(
                    beta,
                    &[
                        column_a.inner_product(evaluation_point),
                        column_b.inner_product(evaluation_point),
                    ],
                );
            let d_fold_eval = alpha
                * fold_vals(
                    beta,
                    &[
                        filtered_a.inner_product(evaluation_point),
                        filtered_b.inner_product(evaluation_point),
                    ],
                );
            // The output has two rows, but the verifier claims it only has one.
            verify_evaluate_filter(
                verification_builder,
                c_fold_eval,
                d_fold_eval,
                chi_n_eval,
                (&[true]).inner_product(evaluation_point),
                (&[true, false, true]).inner_product(evaluation_point),
            )
            .unwrap();
        },
    );
    // The d_star and d_fold constraints fail exactly at the second output row,
    // which is not covered by the claimed chi_m.
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![
            vec![true, true, true],
            vec![true, false, false],
            vec![true, true, true]
        ]
    );
    assert_eq!(verification_builder.get_zero_sum_results(), vec![true]);
}

#[test]
fn we_cannot_verify_filter_with_too_few_final_round_mle_evaluations() {
    let err = try_verify_filter_with_mock(3, vec![vec![]]).unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::TooFewMLEEvaluations
        }
    ));
}

#[test]
fn we_cannot_verify_filter_if_the_sumcheck_proof_is_too_small() {
    let err =
        try_verify_filter_with_mock(2, vec![vec![TestScalar::ZERO, TestScalar::ZERO]]).unwrap_err();
    assert!(matches!(
        err,
        ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::SumcheckProofTooSmall
        }
    ));
}
