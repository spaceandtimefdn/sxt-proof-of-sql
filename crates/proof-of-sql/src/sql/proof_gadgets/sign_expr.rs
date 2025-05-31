use crate::{
    base::{
        bit::{
            bit_mask_utils::{is_bit_mask_negative_representation, make_bit_mask},
            compute_varying_bit_matrix, BitDistribution, BitDistributionError,
        },
        proof::ProofError,
        scalar::{Scalar, ScalarExt},
    },
    sql::proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec, vec::Vec};
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shl;

/// Compute the sign bit for a column of scalars.
///
/// # Panics
/// Panics if `bits.last()` is `None` or if `result.len()` does not match `table_length`.
///
/// todo! make this more efficient and targeted at just the sign bit rather than all bits to create a proof
pub fn first_round_evaluate_sign<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, expr.len());
    let signs = expr
        .iter()
        .map(|s| make_bit_mask(*s))
        .map(is_bit_mask_negative_representation)
        .collect::<Vec<_>>();
    assert_eq!(table_length, signs.len());
    alloc.alloc_slice_copy(&signs)
}

/// Prove the sign decomposition for a column of scalars.
///
/// # Panics
/// Panics if `bits.last()` is `None`.
///
/// If x1, ..., xn denotes the data, prove the column of
/// booleans, i.e. sign bits, s1, ..., sn where si == 1 if xi > MID and
/// `si == 1` if `xi <= MID` and `MID` is defined in `base/bit/abs_bit_mask.rs`
///
/// Note: We can only prove the sign bit for non-zero scalars, and we restict
/// the range of non-zero scalar so that there is a unique sign representation.
pub fn final_round_evaluate_sign<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    // bit_distribution
    let dist = BitDistribution::new::<S, _>(expr);
    builder.produce_bit_distribution(dist.clone());

    if dist.num_varying_bits() > 0 {
        // prove that the bits are binary
        let bits = compute_varying_bit_matrix(alloc, expr, &dist);
        prove_bits_are_binary(builder, &bits);
    }

    // This might panic if `bits.last()` returns `None`.

    let signs = expr
        .iter()
        .map(|s| make_bit_mask(*s))
        .map(is_bit_mask_negative_representation)
        .collect::<Vec<_>>();
    alloc.alloc_slice_copy(&signs)
}

/// Verify the sign decomposition for a column of scalars.
///
/// # Panics
/// Panics if `bit_evals` is empty and `dist` indicates a variable lead bit.
/// This would mean that there is no way to determine the sign bit.
///
/// See [`final_round_evaluate_sign`].
pub fn verifier_evaluate_sign<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    eval: S,
    chi_eval: S,
    num_bits_allowed: Option<u8>,
) -> Result<S, ProofError> {
    // bit_distribution
    let dist = builder.try_consume_bit_distribution()?;
    let num_varying_bits = dist.num_varying_bits();

    // extract evaluations and commitmens of the multilinear extensions for the varying
    // bits of the expression
    let mut bit_evals = Vec::with_capacity(num_varying_bits);
    for _ in 0..num_varying_bits {
        let eval = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            eval - eval * eval,
            2,
        )?;
        bit_evals.push(eval);
    }

    verify_bit_decomposition(eval, chi_eval, &bit_evals, &dist, num_bits_allowed)
        .map(|sign_eval| chi_eval - sign_eval)
        .map_err(|err| match err {
            BitDistributionError::NoLeadBit => {
                panic!("No lead bit available despite variable lead bit.")
            }
            BitDistributionError::Verification => ProofError::VerificationError {
                error: "invalid bit_decomposition",
            },
        })
}

/// This function checks the consistency of the bit evaluations with the expression evaluation.
/// The column of data is restricted to an unsigned integer type of `num_bits_allowed` bits.
fn verify_bit_decomposition<S: ScalarExt>(
    expr_eval: S,
    chi_eval: S,
    bit_evals: &[S],
    dist: &BitDistribution,
    num_bits_allowed: Option<u8>,
) -> Result<S, BitDistributionError> {
    let sign_eval = dist.leading_bit_eval(bit_evals, chi_eval)?;
    let mut rhs = sign_eval * S::from_wrapping(dist.leading_bit_mask())
        + (chi_eval - sign_eval) * S::from_wrapping(dist.leading_bit_inverse_mask())
        - chi_eval * S::from_wrapping(U256::ONE.shl(255));

    for (vary_index, bit_index) in dist.vary_mask_iter().enumerate() {
        if bit_index != 255 {
            let mult = U256::ONE.shl(bit_index);
            let bit_eval = bit_evals[vary_index];
            rhs += S::from_wrapping(mult) * bit_eval;
        }
    }
    let num_bits_allowed = num_bits_allowed.unwrap_or(S::MAX_BITS);
    if num_bits_allowed > S::MAX_BITS {
        return Err(BitDistributionError::Verification);
    }
    let bits_that_must_match_inverse_lead_bit =
        U256::MAX.shl(num_bits_allowed - 1) ^ U256::ONE.shl(255);
    let is_eval_correct_number_of_bits = bits_that_must_match_inverse_lead_bit
        & dist.leading_bit_inverse_mask()
        == bits_that_must_match_inverse_lead_bit;
    (rhs == expr_eval && is_eval_correct_number_of_bits)
        .then_some(sign_eval)
        .ok_or(BitDistributionError::Verification)
}

fn prove_bits_are_binary<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    bits: &[&'a [bool]],
) {
    for &seq in bits {
        builder.produce_intermediate_mle(seq);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(seq)]),
                (-S::one(), vec![Box::new(seq), Box::new(seq)]),
            ],
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            bit::BitDistribution,
            proof::ProofError,
            scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
        },
        sql::{
            proof::mock_verification_builder::MockVerificationBuilder,
            proof_gadgets::verifier_evaluate_sign,
        },
    };
    use bnum::{
        cast::As,
        types::{I256, U256},
    };
    use core::ops::Shl;

    fn evaluate_matrix(matrix: &[&[I256]], terms: &[TestScalar]) -> Vec<TestScalar> {
        matrix
            .iter()
            .map(|row| evaluate_terms(row, terms))
            .collect()
    }

    fn evaluate_terms(coeffs: &[I256], terms: &[TestScalar]) -> TestScalar {
        coeffs
            .iter()
            .zip(terms)
            .map(|(&coef, &term)| {
                if coef < I256::ZERO {
                    -TestScalar::from_wrapping((-coef).as_::<U256>()) * term
                } else {
                    TestScalar::from_wrapping(coef.as_::<U256>()) * term
                }
            })
            .sum()
    }

    #[test]
    fn we_can_verify_evaluate_sign() {
        let dist = BitDistribution {
            vary_mask: [629, 0, 0, 0],
            leading_bit_mask: [2, 0, 0, 9_223_372_036_854_775_808],
        };
        let chi_eval = TestScalar::ONE;
        let bit_evals = [0, 0, 1, 1, 0, 1].map(TestScalar::from);
        let expr_eval = TestScalar::from(562);
        let mut builder = MockVerificationBuilder::new(
            vec![dist],
            3,
            Vec::new(),
            vec![bit_evals.to_vec()],
            Vec::new(),
            vec![],
            Vec::new(),
        );
        let sign_eval = verifier_evaluate_sign(&mut builder, expr_eval, chi_eval, None).unwrap();
        assert_eq!(sign_eval, TestScalar::ZERO);
        assert!(builder.get_identity_results().iter().flatten().all(|b| *b));
    }

    #[test]
    fn we_can_verify_evaluate_sign_positive_sign() {
        let dist = BitDistribution {
            vary_mask: [629, 0, 0, 0],
            leading_bit_mask: [2, 0, 0, 9_223_372_036_854_775_808],
        };
        let a = TestScalar::TEN;
        let b = TestScalar::TWO;
        let expr_eval = TestScalar::from(118) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
            + TestScalar::from(562) * a * (TestScalar::ONE - b)
            + TestScalar::from(3) * (TestScalar::ONE - a) * b;
        let chi_eval = TestScalar::from(1) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
            + TestScalar::from(1) * a * (TestScalar::ONE - b)
            + TestScalar::from(1) * (TestScalar::ONE - a) * b;
        let bit_evals = [
            TestScalar::from(0) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
                + TestScalar::from(0) * a * (TestScalar::ONE - b)
                + TestScalar::from(1) * (TestScalar::ONE - a) * b,
            TestScalar::from(1) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
                + TestScalar::from(0) * a * (TestScalar::ONE - b)
                + TestScalar::from(0) * (TestScalar::ONE - a) * b,
            TestScalar::from(1) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
                + TestScalar::from(1) * a * (TestScalar::ONE - b)
                + TestScalar::from(0) * (TestScalar::ONE - a) * b,
            TestScalar::from(1) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
                + TestScalar::from(1) * a * (TestScalar::ONE - b)
                + TestScalar::from(0) * (TestScalar::ONE - a) * b,
            TestScalar::from(1) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
                + TestScalar::from(0) * a * (TestScalar::ONE - b)
                + TestScalar::from(0) * (TestScalar::ONE - a) * b,
            TestScalar::from(0) * (TestScalar::ONE - a) * (TestScalar::ONE - b)
                + TestScalar::from(1) * a * (TestScalar::ONE - b)
                + TestScalar::from(0) * (TestScalar::ONE - a) * b,
        ];
        let mut builder = MockVerificationBuilder::new(
            vec![dist],
            3,
            Vec::new(),
            vec![bit_evals.to_vec()],
            Vec::new(),
            vec![],
            Vec::new(),
        );
        let sign_eval = verifier_evaluate_sign(&mut builder, expr_eval, chi_eval, None).unwrap();
        assert_eq!(sign_eval, TestScalar::ZERO);
    }

    #[test]
    fn we_can_verify_evaluate_sign_i8_sign() {
        let dist = BitDistribution {
            vary_mask: [125, 0, 0, 9_223_372_036_854_775_808],
            leading_bit_mask: [2, 0, 0, 9_223_372_036_854_775_808],
        };
        let a = TestScalar::TEN;
        let b = TestScalar::TWO;
        let one_minus_a = TestScalar::ONE - a;
        let one_minus_b = TestScalar::ONE - b;

        let s = [
            one_minus_a * one_minus_b,
            a * one_minus_b,
            one_minus_a * b,
            a * b,
        ];

        let expr_eval = evaluate_terms(&[106, 23, -60, -76].map(I256::from), &s);
        let chi_eval = evaluate_terms(&[1, 1, 1, 1].map(I256::from), &s);

        let bit_matrix: &[&[I256]] = &[
            &[0, 1, 0, 0].map(I256::from),
            &[0, 1, 1, 1].map(I256::from),
            &[1, 0, 0, 0].map(I256::from),
            &[0, 1, 0, 1].map(I256::from),
            &[1, 0, 0, 1].map(I256::from),
            &[1, 0, 1, 0].map(I256::from),
            &[1, 1, 0, 0].map(I256::from),
        ];

        let bit_evals = evaluate_matrix(bit_matrix, &s);

        let mut builder = MockVerificationBuilder::new(
            vec![dist],
            3,
            Vec::new(),
            vec![bit_evals],
            Vec::new(),
            vec![],
            Vec::new(),
        );

        let expected_eval = evaluate_terms(&[I256::ZERO, I256::ZERO, I256::ONE, I256::ONE], &s);

        let sign_eval =
            verifier_evaluate_sign(&mut builder.clone(), expr_eval, chi_eval, Some(8)).unwrap();
        assert_eq!(sign_eval, expected_eval);
        let err = verifier_evaluate_sign(&mut builder, expr_eval, chi_eval, Some(7)).unwrap_err();
        assert!(matches!(
            err,
            ProofError::VerificationError {
                error: "invalid bit_decomposition"
            }
        ));
    }

    #[test]
    fn we_can_verify_evaluate_sign_with_max_data_type() {
        // Note that this is not i251 because i251::MIN would theoretically be -2^250
        let i252_val = -TestScalar::from_wrapping(U256::ONE.shl(250)) - TestScalar::ONE;
        let data = [TestScalar::ZERO, i252_val];
        let dist = BitDistribution::new::<TestScalar, TestScalar>(&data);
        let a = TestScalar::TEN;
        let b = TestScalar::TWO;
        let one_minus_a = TestScalar::ONE - a;
        let one_minus_b = TestScalar::ONE - b;

        let s = [
            one_minus_a * one_minus_b,
            a * one_minus_b,
            one_minus_a * b,
            a * b,
        ];

        let expr_eval = evaluate_terms(&[I256::ZERO, -I256::ONE.shl(250u8) - I256::ONE], &s);
        let chi_eval = evaluate_terms(&[1, 1].map(I256::from), &s);

        let bit_matrix: &[&[I256]] = &[&[0, 0].map(I256::from), &[1, 0].map(I256::from)];

        let bit_evals = evaluate_matrix(bit_matrix, &s);

        let mut builder = MockVerificationBuilder::new(
            vec![dist],
            3,
            Vec::new(),
            vec![bit_evals],
            Vec::new(),
            vec![],
            Vec::new(),
        );

        let expected_eval = evaluate_terms(&[I256::ZERO, I256::ONE], &s);

        let sign_eval =
            verifier_evaluate_sign(&mut builder.clone(), expr_eval, chi_eval, Some(252)).unwrap();
        assert_eq!(sign_eval, expected_eval);
        // Should fail because the TestScalar can only securely hold i252 values
        let err = verifier_evaluate_sign(&mut builder.clone(), expr_eval, chi_eval, Some(253))
            .unwrap_err();
        assert!(matches!(
            err,
            ProofError::VerificationError {
                error: "invalid bit_decomposition"
            }
        ));
        // Should fail because the highest value is too big to be held by an i251
        let err = verifier_evaluate_sign(&mut builder, expr_eval, chi_eval, Some(251)).unwrap_err();
        assert!(matches!(
            err,
            ProofError::VerificationError {
                error: "invalid bit_decomposition"
            }
        ));
    }
}
