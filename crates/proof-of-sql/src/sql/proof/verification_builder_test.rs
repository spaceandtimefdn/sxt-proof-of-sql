use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{bit::BitDistribution, map::IndexMap},
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
fn we_can_consume_verification_builder_inputs() {
    let mut chi_evaluations = IndexMap::default();
    chi_evaluations.insert(4, Curve25519Scalar::from(40_u64));
    chi_evaluations.insert(7, Curve25519Scalar::from(70_u64));

    let mut rho_evaluations = IndexMap::default();
    rho_evaluations.insert(8, Curve25519Scalar::from(80_u64));

    let first_round = [
        Curve25519Scalar::from(101_u64),
        Curve25519Scalar::from(102_u64),
    ];
    let final_round = [
        Curve25519Scalar::from(201_u64),
        Curve25519Scalar::from(202_u64),
        Curve25519Scalar::from(203_u64),
    ];
    let bit_distribution = BitDistribution::new::<Curve25519Scalar, _>(&[1_u64, 2, 3]);
    let bit_distributions = [bit_distribution.clone()];
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations,
        rho_evaluations,
        singleton_chi_evaluation: Curve25519Scalar::from(1_u64),
        first_round_pcs_proof_evaluations: &first_round,
        final_round_pcs_proof_evaluations: &final_round,
        rho_256_evaluation: Some(Curve25519Scalar::from(256_u64)),
        ..Default::default()
    };

    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &bit_distributions,
        &[][..],
        VecDeque::new(),
        vec![4, 7],
        vec![8],
        0,
    );

    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(40_u64), 4)
    );
    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(70_u64), 7)
    );
    assert_eq!(
        builder.try_consume_rho_evaluation().unwrap(),
        Curve25519Scalar::from(80_u64)
    );
    assert_eq!(
        builder.try_consume_first_round_mle_evaluations(2).unwrap(),
        first_round
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluation().unwrap(),
        final_round[0]
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluations(2).unwrap(),
        &final_round[1..]
    );
    assert_eq!(
        builder.try_consume_bit_distribution().unwrap(),
        bit_distribution
    );
    assert_eq!(
        builder.singleton_chi_evaluation(),
        Curve25519Scalar::from(1_u64)
    );
    assert_eq!(
        builder.rho_256_evaluation(),
        Some(Curve25519Scalar::from(256_u64))
    );
}
