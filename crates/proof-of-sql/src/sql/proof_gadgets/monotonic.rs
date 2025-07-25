//! Prove that a column is increasing or decreasing, strictly or non-strictly.
use super::{
    final_round_evaluate_shift, final_round_evaluate_sign, first_round_evaluate_shift,
    verifier_evaluate_sign, verify_shift,
};
use crate::{
    base::{proof::ProofError, scalar::Scalar},
    sql::proof::{FinalRoundBuilder, FirstRoundBuilder, VerificationBuilder},
};
use alloc::vec;
use bumpalo::Bump;
use tracing::{span, Level};

/// Perform first round evaluation of monotonicity.
pub(crate) fn first_round_evaluate_monotonic<'a, S: Scalar>(
    builder: &mut FirstRoundBuilder<'a, S>,
    alloc: &'a Bump,
    column: &'a [S],
) {
    first_round_evaluate_shift(builder, alloc, column);
}

/// Perform final round evaluation of monotonicity.
#[tracing::instrument(
    name = "Monotonic::final_round_evaluate_monotonic",
    level = "debug",
    skip_all
)]
pub(crate) fn final_round_evaluate_monotonic<'a, S: Scalar, const STRICT: bool, const ASC: bool>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    column: &'a [S],
) {
    let num_rows = column.len();
    // 1. Prove that `shifted_column` is a shift of `column`
    let shifted_column = final_round_evaluate_shift(builder, alloc, alpha, beta, column);
    // 2. Construct an indicator `diff = column - shifted_column`
    let span = span!(Level::DEBUG, "allocate diff").entered();
    let diff = if num_rows >= 1 {
        alloc.alloc_slice_fill_with(num_rows + 1, |i| {
            if i == num_rows {
                -column[num_rows - 1]
            } else {
                column[i] - shifted_column[i]
            }
        })
    } else {
        alloc.alloc_slice_fill_copy(1, S::ZERO)
    };
    span.exit();

    // Since sign expr which we uses for the sign proof only distinguishes between nonnegative
    // and negative integers we need to transform the indicator to be either ind < 0 or ind >= 0
    //
    // Due to the fact that column is monotonic either column - shifted_column
    // or shifted_column - column will be all nonnegative or all negative
    // everywhere with the possible exception of the first and last element
    //
    // Hence we need to do the following transformation
    // column > shifted_column => shifted_column - column < 0
    // column >= shifted_column => column - shifted_column >= 0
    // column < shifted_column => column - shifted_column < 0
    // column <= shifted_column => shifted_column - column >= 0
    //
    // This is why ind is constructed as below
    let span = span!(Level::DEBUG, "allocate ind").entered();
    let ind = match (STRICT, ASC) {
        (true, true) | (false, false) => alloc.alloc_slice_fill_with(num_rows + 1, |i| -diff[i]),
        _ => diff as &[_],
    };
    span.exit();

    // 3. Prove the sign of `ind`
    final_round_evaluate_sign(builder, alloc, ind);
}

pub(crate) fn verify_monotonic<S: Scalar, const STRICT: bool, const ASC: bool>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    column_eval: S,
    chi_eval: S,
) -> Result<(), ProofError> {
    // 1. Verify that `shifted_column` is a shift of `column`
    let (shifted_column_eval, shifted_chi_eval) =
        verify_shift(builder, alpha, beta, column_eval, chi_eval)?;
    // 2. Verify that `ind_eval` is correct. See above for the explanation.
    let ind_eval = match (STRICT, ASC) {
        (true, true) | (false, false) => shifted_column_eval - column_eval,
        _ => column_eval - shifted_column_eval,
    };
    let sign_eval = verifier_evaluate_sign(builder, ind_eval, shifted_chi_eval, None)?;
    let singleton_chi_eval = builder.singleton_chi_evaluation();
    let allowed_evals = if STRICT {
        // sign(ind) == 1 for all but the first element and the last element
        // The first and last elements can only fit into three patterns
        // 1. negative and non-negative
        // 2. non-negative and negative
        // 3. non-negative and non-negative
        // Hence the evaluation of sign has to be in one of three cases
        // 1. chi_eval
        // 2. shifted_chi_eval - singleton_chi_eval
        // 3. chi_eval - singleton_chi_eval
        vec![
            chi_eval,
            shifted_chi_eval - singleton_chi_eval,
            chi_eval - singleton_chi_eval,
        ]
    } else {
        // sign(ind) == 0 for all but the first element and the last element
        // The first and last elements can only fit into four patterns
        // 1. negative and non-negative
        // 2. non-negative and negative
        // 3. negative and negative
        // 4. non-negative and non-negative (only the all zero case)
        // Hence the evaluation of sign has to be in one of four cases
        // 1. singleton_chi_eval
        // 2. shifted_chi_eval - chi_eval
        // 3. singleton_chi_eval + shifted_chi_eval - chi_eval
        // 4. 0
        vec![
            singleton_chi_eval,
            shifted_chi_eval - chi_eval,
            singleton_chi_eval + shifted_chi_eval - chi_eval,
            S::ZERO,
        ]
    };
    if !allowed_evals.contains(&sign_eval) {
        return Err(ProofError::VerificationError {
            error: "monotonicty check failed",
        });
    }
    Ok(())
}
