use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{bit::BitDistribution, map::IndexMap, proof::ProofSizeMismatch},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::{SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{collections::VecDeque, vec, vec::Vec};
use num_traits::Zero;

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
fn we_build_identity_sumcheck_polynomial_evaluation() {
    let mle_evaluations = SumcheckMleEvaluations {
        random_evaluation: Curve25519Scalar::from(7u64),
        ..Default::default()
    };
    let subpolynomial_multipliers = [Curve25519Scalar::from(11u64)];
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
            Curve25519Scalar::from(13u64),
            1,
        )
        .unwrap();

    assert_eq!(
        builder.sumcheck_evaluation(),
        Curve25519Scalar::from(11u64)
            * Curve25519Scalar::from(13u64)
            * Curve25519Scalar::from(7u64)
    );
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

#[test]
fn we_can_consume_queued_verification_inputs() {
    let mut chi_evaluations = IndexMap::default();
    chi_evaluations.insert(3, Curve25519Scalar::from(33u64));
    chi_evaluations.insert(5, Curve25519Scalar::from(55u64));
    let mut rho_evaluations = IndexMap::default();
    rho_evaluations.insert(2, Curve25519Scalar::from(22u64));
    let first_round_evaluations = [
        Curve25519Scalar::from(101u64),
        Curve25519Scalar::from(102u64),
    ];
    let final_round_evaluations = [
        Curve25519Scalar::from(201u64),
        Curve25519Scalar::from(202u64),
    ];
    let bit_distribution =
        BitDistribution::new::<Curve25519Scalar, _>(&[Curve25519Scalar::zero(), 1u64.into()]);
    let bit_distributions = [bit_distribution.clone()];
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations,
        rho_evaluations,
        singleton_chi_evaluation: Curve25519Scalar::from(9u64),
        first_round_pcs_proof_evaluations: &first_round_evaluations,
        final_round_pcs_proof_evaluations: &final_round_evaluations,
        rho_256_evaluation: Some(Curve25519Scalar::from(256u64)),
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &bit_distributions,
        &[][..],
        VecDeque::new(),
        vec![3, 5],
        vec![2],
        0,
    );

    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(33u64), 3)
    );
    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(55u64), 5)
    );
    assert_eq!(
        builder.try_consume_rho_evaluation().unwrap(),
        Curve25519Scalar::from(22u64)
    );
    assert_eq!(
        builder.try_consume_first_round_mle_evaluations(2).unwrap(),
        first_round_evaluations
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluations(2).unwrap(),
        final_round_evaluations
    );
    assert_eq!(
        builder.try_consume_bit_distribution().unwrap(),
        bit_distribution
    );
    assert_eq!(
        builder.singleton_chi_evaluation(),
        Curve25519Scalar::from(9u64)
    );
    assert_eq!(
        builder.rho_256_evaluation(),
        Some(Curve25519Scalar::from(256u64))
    );
}

#[test]
fn we_get_errors_for_missing_verification_inputs() {
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
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::TooFewRhoLengths)
    ));
    assert!(matches!(
        builder.try_consume_first_round_mle_evaluation(),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
    assert!(matches!(
        builder.try_consume_final_round_mle_evaluation(),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
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
fn we_get_errors_for_missing_queued_mle_lengths() {
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![3],
        vec![5],
        0,
    );

    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::ChiLengthNotFound)
    ));
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::RhoLengthNotFound)
    ));
}

#[test]
fn we_get_errors_for_invalid_subpolynomial_accounting() {
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
            Curve25519Scalar::from(2u64),
            1,
        ),
        Err(ProofSizeMismatch::ConstraintCountMismatch)
    ));

    let subpolynomial_multipliers = [Curve25519Scalar::from(1u64)];
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
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
            Curve25519Scalar::from(2u64),
            1,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));

    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
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
            Curve25519Scalar::from(2u64),
            2,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));
}
