use crate::{
    base::{
        proof::ProofError,
        scalar::{Scalar, ScalarExt},
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_gadgets::{
            final_round_evaluate_sign, first_round_evaluate_sign, sign_expr::alloc_signs,
            verifier_evaluate_sign,
        },
    },
};
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shr;
use itertools::Itertools;

/// Compute the sign bit for a column of scalars.
#[tracing::instrument(
    name = "SignExpr::first_round_evaluate_sign",
    level = "debug",
    skip_all
)]
pub fn first_round_evaluate_sign_for_scalars<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    first_round_evaluate_sign(table_length, alloc, expr)
}

/// Prove the sign decomposition for a column of scalars.
///
/// If x1, ..., xn denotes the data, prove the column of
/// booleans, i.e. sign bits, s1, ..., sn where si == 1 if xi > MID and
/// `si == 1` if `xi <= MID` and `MID` is defined in `base/bit/abs_bit_mask.rs`
///
/// Note: We can only prove the sign bit for non-zero scalars, and we restrict
/// the range of non-zero scalars so that there is a unique sign representation.
#[tracing::instrument(
    name = "SignExpr::final_round_evaluate_sign",
    level = "debug",
    skip_all
)]
pub fn final_round_evaluate_sign_for_scalars<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    let chi = alloc.alloc_slice_fill_copy(expr.len(), true);
    let (lead_bits, half_signed_max_complement, trail_bits): (Vec<S>, Vec<S>, Vec<S>) = expr
        .iter()
        .copied()
        .map(|value| {
            let abs_value = if (value > S::MAX_SIGNED) {
                -value
            } else {
                value
            };
            let abs_value_as_u256 = (abs_value).into_u256_wrapping();
            let lead_bits = abs_value_as_u256.shr(1);
            let trail_bit = abs_value_as_u256 & U256::ONE;
            let lead_bits = S::from_wrapping(lead_bits);
            let half_signed_max_complement = lead_bits - S::HALF_MAX_SIGNED;
            (
                lead_bits,
                half_signed_max_complement,
                S::from_wrapping(trail_bit),
            )
        })
        .multiunzip();
    let lead_bits_column = alloc.alloc_slice_copy(&lead_bits);
    builder.produce_intermediate_mle(lead_bits_column as &[S]);
    final_round_evaluate_sign(builder, alloc, lead_bits_column);
    let half_signed_max_complement_column = alloc.alloc_slice_copy(&half_signed_max_complement);
    let half_max_signed_tracker =
        final_round_evaluate_sign(builder, alloc, half_signed_max_complement_column);
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(chi as &[_]), Box::new(lead_bits_column as &[_])],
            ),
            (-S::HALF_MAX_SIGNED, vec![Box::new(chi as &[_])]),
            (
                -S::one(),
                vec![
                    Box::new(half_max_signed_tracker),
                    Box::new(lead_bits_column as &[_]),
                ],
            ),
            (S::HALF_MAX_SIGNED, vec![Box::new(half_max_signed_tracker)]),
        ],
    );
    let trail_bit_column = alloc.alloc_slice_copy(&trail_bits);
    builder.produce_intermediate_mle(trail_bit_column as &[S]);
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(trail_bit_column as &[_])]),
            (
                -S::one(),
                vec![
                    Box::new(trail_bit_column as &[_]),
                    Box::new(trail_bit_column as &[_]),
                ],
            ),
        ],
    );

    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(
            S::one(),
            vec![
                Box::new(trail_bit_column as &[_]),
            ],
        ),(
            -S::one(),
            vec![
                Box::new(half_max_signed_tracker),
                Box::new(trail_bit_column as &[_]),
            ],
        )],
    );

    let sign_column = alloc_signs(alloc, expr);
    builder.produce_intermediate_mle(sign_column as &[_]);
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(sign_column as &[_])]),
            (
                -S::one(),
                vec![Box::new(sign_column as &[_]), Box::new(sign_column as &[_])],
            ),
        ],
    );

    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::ONE, vec![Box::new(expr as &[_])]),
            (
                S::from(4),
                vec![
                    Box::new(lead_bits_column as &[_]),
                    Box::new(sign_column as &[_]),
                ],
            ),
            (-S::TWO, vec![Box::new(lead_bits_column as &[_])]),
            (
                S::TWO,
                vec![
                    Box::new(sign_column as &[_]),
                    Box::new(trail_bit_column as &[_]),
                ],
            ),
            (-S::ONE, vec![Box::new(trail_bit_column as &[_])]),
        ],
    );

    sign_column
}

/// Verify the sign decomposition for a column of scalars.
///
/// # Panics
/// Panics if `bit_evals` is empty and `dist` indicates a variable lead bit.
/// This would mean that there is no way to determine the sign bit.
///
/// See [`final_round_evaluate_sign`].
pub fn verifier_evaluate_sign_for_scalars<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    eval: S,
    chi_eval: S,
) -> Result<S, ProofError> {
    // Consume all but the trail bit.
    let lead_bits = builder.try_consume_final_round_mle_evaluation()?;
    // Ensure that the lead bits are between 0 and half of the max signed value, inclusive.
    let expect_zero = verifier_evaluate_sign(builder, lead_bits, chi_eval, None)?;
    if expect_zero != S::ZERO {
        return Err(ProofError::VerificationError {
            error: "invalid lead bits",
        });
    }
    let offset_lead_bits = lead_bits - S::HALF_MAX_SIGNED * chi_eval;
    // Any one on this eval should represent a half max signed row.
    // These are the only rows that should be 0 or above for both sign verifications.
    let half_max_signed_tracker = verifier_evaluate_sign(builder, offset_lead_bits, chi_eval, None)?;
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        (chi_eval - half_max_signed_tracker) * (lead_bits - S::HALF_MAX_SIGNED),
        2,
    )?;

    // Confirm the last bit is, in fact, a bit.
    let trail_bit = builder.try_consume_final_round_mle_evaluation()?;
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        trail_bit - trail_bit * trail_bit,
        2,
    )?;

    // If S::MAX_SIGNED is even, we need to verify that the reconstructed value is not S::MAX_SIGNED + S::ONE.
    // So we conditionally check that the lead bits and trail bit are not simultaneously maxed out.
    // We need this check for Bn 254, so it is need for evm verification.
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        trail_bit - half_max_signed_tracker * trail_bit,
        2,
    )?;

    // Verify that the sign eval is a bit column.
    let sign_eval = builder.try_consume_final_round_mle_evaluation()?;
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        sign_eval - sign_eval * sign_eval,
        2,
    )?;

    // Verify that the reconstruction of the original eval is correct.
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        eval + (S::TWO * lead_bits + trail_bit) * (S::TWO * sign_eval - S::ONE),
        2,
    )?;

    Ok(sign_eval)
}
