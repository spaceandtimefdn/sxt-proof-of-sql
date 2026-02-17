use crate::{
    base::{proof::ProofError, scalar::Scalar},
    sql::{
        proof::{SumcheckSubpolynomialType, VerificationBuilder},
        proof_gadgets::verifier_evaluate_sign,
    },
};

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
    if !S::should_check_trail_bit() {
        unimplemented!()
    }
    // Consume all but the trail bit.
    let lead_bits = builder.try_consume_final_round_mle_evaluation()?;
    // Ensure that the leead bits are between 0 and half of the max signed value, inclusive.
    let expect_zero = verifier_evaluate_sign(builder, lead_bits, chi_eval, None)?;
    if expect_zero != S::ZERO {
        return Err(ProofError::VerificationError {
            error: "invalid lead bits",
        });
    }
    let half_max_signed_eval = S::HALF_MAX_SIGNED * chi_eval;
    let offset_lead_bits = lead_bits - half_max_signed_eval;
    // Any one on this eval should represent a half max signed row.
    // These are the only rows that should be 0 or above for both sign verifications.
    let half_max_signed_tracker =
        verifier_evaluate_sign(builder, offset_lead_bits, chi_eval, None)?;
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        half_max_signed_tracker * (lead_bits - half_max_signed_eval),
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
        half_max_signed_tracker * trail_bit,
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
        eval + (S::TWO * lead_bits + trail_bit) * (S::TWO * sign_eval - chi_eval),
        2,
    )?;

    Ok(sign_eval)
}
