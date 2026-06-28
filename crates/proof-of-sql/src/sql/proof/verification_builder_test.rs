use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{bit::BitDistribution, map::indexmap, proof::ProofSizeMismatch},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::{SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::collections::VecDeque;
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
fn we_can_consume_chi_and_rho_evaluations_by_declared_lengths() {
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations: indexmap! {
            2 => Curve25519Scalar::from(20u64),
            4 => Curve25519Scalar::from(40u64),
        },
        rho_evaluations: indexmap! {
            3 => Curve25519Scalar::from(30u64),
        },
        singleton_chi_evaluation: Curve25519Scalar::from(1u64),
        rho_256_evaluation: Some(Curve25519Scalar::from(256u64)),
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![2, 4],
        vec![3],
        0,
    );

    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(20u64), 2)
    );
    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(40u64), 4)
    );
    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::TooFewChiLengths)
    ));
    assert_eq!(
        builder.try_consume_rho_evaluation().unwrap(),
        Curve25519Scalar::from(30u64)
    );
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::TooFewRhoLengths)
    ));
    assert_eq!(
        builder.singleton_chi_evaluation(),
        Curve25519Scalar::from(1u64)
    );
    assert_eq!(
        builder.rho_256_evaluation(),
        Some(Curve25519Scalar::from(256u64))
    );
}

#[test]
fn we_report_missing_chi_and_rho_lengths_after_queue_lookup() {
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations: indexmap! {
            2 => Curve25519Scalar::from(20u64),
        },
        rho_evaluations: indexmap! {
            3 => Curve25519Scalar::from(30u64),
        },
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![99],
        vec![88],
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
fn we_can_consume_mle_evaluation_batches_and_report_short_proofs() {
    let first_round = [Curve25519Scalar::from(11u64), Curve25519Scalar::from(12u64)];
    let final_round = [Curve25519Scalar::from(21u64)];
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
        first_round
    );
    assert!(matches!(
        builder.try_consume_first_round_mle_evaluation(),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
    assert_eq!(
        builder.try_consume_final_round_mle_evaluation().unwrap(),
        final_round[0]
    );
    assert!(matches!(
        builder.try_consume_final_round_mle_evaluation(),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
}

#[test]
fn we_can_consume_bit_distributions_in_order() {
    let bit_distributions = [
        BitDistribution {
            leading_bit_mask: [1, 0, 0, 0],
            vary_mask: [2, 0, 0, 0],
        },
        BitDistribution {
            leading_bit_mask: [3, 0, 0, 0],
            vary_mask: [4, 0, 0, 0],
        },
    ];
    let mut builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations::default(),
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
fn identity_subpolynomials_are_weighted_by_random_evaluation() {
    let mle_evaluations = SumcheckMleEvaluations {
        random_evaluation: Curve25519Scalar::from(7u64),
        ..Default::default()
    };
    let subpolynomial_multipliers = [Curve25519Scalar::from(10u64), Curve25519Scalar::from(11u64)];
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
            Curve25519Scalar::from(3u64),
            1,
        )
        .unwrap();
    builder
        .try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(5u64),
            2,
        )
        .unwrap();

    assert_eq!(
        builder.sumcheck_evaluation(),
        Curve25519Scalar::from(265u64)
    );
}

#[test]
fn we_report_constraint_and_sumcheck_degree_mismatches() {
    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(1u64),
            0,
        ),
        Err(ProofSizeMismatch::ConstraintCountMismatch)
    ));

    let multiplier = [Curve25519Scalar::from(1u64)];
    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &multiplier,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            Curve25519Scalar::from(1u64),
            1,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));

    let mut builder = VerificationBuilderImpl::new(
        SumcheckMleEvaluations::default(),
        &[][..],
        &multiplier,
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        1,
    );
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(1u64),
            2,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));
}
