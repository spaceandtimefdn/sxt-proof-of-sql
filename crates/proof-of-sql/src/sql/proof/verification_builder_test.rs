use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{bit::BitDistribution, map::IndexMap, proof::ProofSizeMismatch},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::{SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::collections::VecDeque;
use num_traits::Zero;

fn s(value: u64) -> Curve25519Scalar {
    Curve25519Scalar::from(value)
}

#[test]
fn an_empty_sumcheck_polynomial_evaluates_to_zero() {
    let mle_evaluations = SumcheckMleEvaluations {
        ..Default::default()
    };
    let builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert_eq!(builder.sumcheck_evaluation(), Curve25519Scalar::zero());
}

#[test]
fn we_build_up_a_sumcheck_polynomial_evaluation_from_subpolynomial_evaluations() {
    let mle_evaluations = SumcheckMleEvaluations {
        ..Default::default()
    };
    let subpolynomial_multipliers = [
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(100u64),
    ];
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &subpolynomial_multipliers,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    builder
        .try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(2u64),
            1,
        )
        .unwrap();
    builder
        .try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(3u64),
            1,
        )
        .unwrap();
    let expected_sumcheck_evaluation = subpolynomial_multipliers[0] * Curve25519Scalar::from(2u64)
        + subpolynomial_multipliers[1] * Curve25519Scalar::from(3u64);
    assert_eq!(builder.sumcheck_evaluation(), expected_sumcheck_evaluation);
}

#[test]
fn we_can_consume_verification_builder_evaluation_inputs() {
    let first_round_evals = [s(1), s(2), s(3)];
    let final_round_evals = [s(4), s(5), s(6)];
    let subpolynomial_multipliers = [s(13), s(17)];
    let mut chi_evaluations = IndexMap::default();
    chi_evaluations.insert(2, s(20));
    let mut rho_evaluations = IndexMap::default();
    rho_evaluations.insert(3, s(30));
    let bit_distribution = BitDistribution {
        vary_mask: [1, 0, 0, 0],
        leading_bit_mask: [2, 0, 0, 0],
    };
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations,
        rho_evaluations,
        singleton_chi_evaluation: s(7),
        random_evaluation: s(11),
        first_round_pcs_proof_evaluations: &first_round_evals,
        final_round_pcs_proof_evaluations: &final_round_evals,
        rho_256_evaluation: Some(s(256)),
    };
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        mle_evaluations,
        core::slice::from_ref(&bit_distribution),
        &subpolynomial_multipliers,
        [s(29)].into(),
        vec![2],
        vec![3],
        3,
    );

    assert_eq!(builder.try_consume_chi_evaluation().unwrap(), (s(20), 2));
    assert_eq!(builder.try_consume_rho_evaluation().unwrap(), s(30));
    assert_eq!(
        builder.try_consume_first_round_mle_evaluation().unwrap(),
        s(1)
    );
    assert_eq!(
        builder.try_consume_first_round_mle_evaluations(2).unwrap(),
        vec![s(2), s(3)]
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluations(2).unwrap(),
        vec![s(4), s(5)]
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluation().unwrap(),
        s(6)
    );
    assert_eq!(
        builder.try_consume_bit_distribution().unwrap(),
        bit_distribution
    );
    builder
        .try_produce_sumcheck_subpolynomial_evaluation(SumcheckSubpolynomialType::Identity, s(2), 2)
        .unwrap();
    builder
        .try_produce_sumcheck_subpolynomial_evaluation(SumcheckSubpolynomialType::ZeroSum, s(5), 3)
        .unwrap();
    assert_eq!(builder.try_consume_post_result_challenge().unwrap(), s(29));
    assert_eq!(builder.singleton_chi_evaluation(), s(7));
    assert_eq!(builder.rho_256_evaluation(), Some(s(256)));
    assert_eq!(
        builder.sumcheck_evaluation(),
        s(13) * s(2) * s(11) + s(17) * s(5)
    );
}

#[test]
fn we_get_size_errors_for_missing_verification_builder_inputs() {
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::TooFewChiLengths)
    ));

    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![2],
        Vec::new(),
        0,
    );
    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::ChiLengthNotFound)
    ));

    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::TooFewRhoLengths)
    ));

    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        vec![3],
        0,
    );
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::RhoLengthNotFound)
    ));

    let first_round_evals = [s(1)];
    let final_round_evals = [s(2)];
    let mle_evaluations = SumcheckMleEvaluations {
        first_round_pcs_proof_evaluations: &first_round_evals,
        final_round_pcs_proof_evaluations: &final_round_evals,
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert!(matches!(
        builder.try_consume_first_round_mle_evaluations(2),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
    assert!(matches!(
        builder.try_consume_final_round_mle_evaluations(2),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));

    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert!(matches!(
        builder.try_consume_bit_distribution(),
        Err(ProofSizeMismatch::TooFewBitDistributions)
    ));
    assert!(matches!(
        builder.try_consume_post_result_challenge(),
        Err(ProofSizeMismatch::PostResultCountMismatch)
    ));
}

#[test]
fn we_get_size_errors_for_invalid_subpolynomial_accounting() {
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            s(1),
            1
        ),
        Err(ProofSizeMismatch::ConstraintCountMismatch)
    ));

    let subpolynomial_multipliers = [s(1)];
    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &subpolynomial_multipliers,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            s(1),
            1
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));

    let subpolynomial_multipliers = [s(1)];
    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &subpolynomial_multipliers,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            s(1),
            2
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));
}

#[test]
fn we_can_consume_post_result_challenges_in_verification_builder() {
    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        [
            Curve25519Scalar::from(123),
            Curve25519Scalar::from(456),
            Curve25519Scalar::from(789),
        ]
        .into(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert_eq!(
        Curve25519Scalar::from(123),
        builder.try_consume_post_result_challenge().unwrap()
    );
    assert_eq!(
        Curve25519Scalar::from(456),
        builder.try_consume_post_result_challenge().unwrap()
    );
    assert_eq!(
        Curve25519Scalar::from(789),
        builder.try_consume_post_result_challenge().unwrap()
    );
}
