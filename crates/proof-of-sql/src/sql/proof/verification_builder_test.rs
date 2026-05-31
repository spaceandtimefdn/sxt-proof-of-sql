use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{bit::BitDistribution, map::IndexMap, proof::ProofSizeMismatch},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::{SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::collections::VecDeque;
use num_traits::Zero;

fn scalar(value: u64) -> Curve25519Scalar {
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
fn we_can_consume_chi_and_rho_evaluations_by_requested_lengths() {
    let mut chi_evaluations = IndexMap::default();
    chi_evaluations.insert(3, scalar(30));
    chi_evaluations.insert(7, scalar(70));
    let mut rho_evaluations = IndexMap::default();
    rho_evaluations.insert(4, scalar(40));
    rho_evaluations.insert(8, scalar(80));

    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations,
        rho_evaluations,
        singleton_chi_evaluation: scalar(1),
        rho_256_evaluation: Some(scalar(256)),
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![3, 7],
        vec![4, 8],
        0,
    );

    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (scalar(30), 3)
    );
    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (scalar(70), 7)
    );
    assert_eq!(builder.try_consume_rho_evaluation().unwrap(), scalar(40));
    assert_eq!(builder.try_consume_rho_evaluation().unwrap(), scalar(80));
    assert_eq!(builder.singleton_chi_evaluation(), scalar(1));
    assert_eq!(builder.rho_256_evaluation(), Some(scalar(256)));
}

#[test]
fn we_error_when_chi_or_rho_lengths_are_missing_or_unknown() {
    let mut chi_evaluations = IndexMap::default();
    chi_evaluations.insert(2, scalar(20));
    let mut rho_evaluations = IndexMap::default();
    rho_evaluations.insert(5, scalar(50));

    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations,
        rho_evaluations,
        ..Default::default()
    };
    let mut missing_lengths = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::<Curve25519Scalar>::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert!(matches!(
        missing_lengths.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::TooFewChiLengths)
    ));
    assert!(matches!(
        missing_lengths.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::TooFewRhoLengths)
    ));

    let mut unknown_lengths = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![9],
        vec![10],
        0,
    );
    assert!(matches!(
        unknown_lengths.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::ChiLengthNotFound)
    ));
    assert!(matches!(
        unknown_lengths.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::RhoLengthNotFound)
    ));
}

#[test]
fn we_can_consume_first_and_final_round_mle_evaluations_in_order() {
    let first_round = [scalar(11), scalar(12), scalar(13)];
    let final_round = [scalar(21), scalar(22)];
    let mle_evaluations = SumcheckMleEvaluations {
        first_round_pcs_proof_evaluations: &first_round,
        final_round_pcs_proof_evaluations: &final_round,
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );

    assert_eq!(
        builder.try_consume_first_round_mle_evaluations(2).unwrap(),
        vec![scalar(11), scalar(12)]
    );
    assert_eq!(
        builder.try_consume_first_round_mle_evaluation().unwrap(),
        scalar(13)
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluations(2).unwrap(),
        vec![scalar(21), scalar(22)]
    );
    assert!(matches!(
        builder.try_consume_first_round_mle_evaluation(),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
    assert!(matches!(
        builder.try_consume_final_round_mle_evaluation(),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
}

#[test]
fn we_can_consume_bit_distributions_in_order() {
    let bit_distributions = [
        BitDistribution::new::<Curve25519Scalar, _>(&[1u64, 1u64]),
        BitDistribution::new::<Curve25519Scalar, _>(&[1u64, 2u64]),
    ];
    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::<Curve25519Scalar>::default(),
        &bit_distributions,
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );

    assert_eq!(
        builder.try_consume_bit_distribution().unwrap(),
        bit_distributions[0]
    );
    assert_eq!(
        builder.try_consume_bit_distribution().unwrap(),
        bit_distributions[1]
    );
    assert!(matches!(
        builder.try_consume_bit_distribution(),
        Err(ProofSizeMismatch::TooFewBitDistributions)
    ));
}

#[test]
fn identity_sumcheck_subpolynomial_uses_random_evaluation_multiplier() {
    let mle_evaluations = SumcheckMleEvaluations {
        random_evaluation: scalar(9),
        ..Default::default()
    };
    let subpolynomial_multipliers = [scalar(7)];
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &subpolynomial_multipliers,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        2,
    );

    builder
        .try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            scalar(3),
            1,
        )
        .unwrap();

    assert_eq!(
        builder.sumcheck_evaluation(),
        scalar(7) * scalar(3) * scalar(9)
    );
}

#[test]
fn we_error_when_sumcheck_subpolynomial_accounting_does_not_match() {
    let mut no_multipliers = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::<Curve25519Scalar>::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        no_multipliers.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            scalar(1),
            1,
        ),
        Err(ProofSizeMismatch::ConstraintCountMismatch)
    ));

    let multipliers = [scalar(1), scalar(1)];
    let mut too_small_for_zero_sum = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &multipliers,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        too_small_for_zero_sum.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            scalar(1),
            2,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));

    let mut too_small_for_identity = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &multipliers,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        too_small_for_identity.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            scalar(1),
            1,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));
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
