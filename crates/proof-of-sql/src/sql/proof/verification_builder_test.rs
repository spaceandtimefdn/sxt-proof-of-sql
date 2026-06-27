use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{map::indexmap, proof::ProofSizeMismatch},
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
fn we_consume_chi_and_rho_evaluations_by_requested_lengths() {
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations: indexmap! {
            3 => Curve25519Scalar::from(30u64),
            5 => Curve25519Scalar::from(50u64),
        },
        rho_evaluations: indexmap! {
            2 => Curve25519Scalar::from(20u64),
            4 => Curve25519Scalar::from(40u64),
        },
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![3, 5],
        vec![2, 4],
        0,
    );

    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(30u64), 3)
    );
    assert_eq!(
        builder.try_consume_rho_evaluation().unwrap(),
        Curve25519Scalar::from(20u64)
    );
    assert_eq!(
        builder.try_consume_chi_evaluation().unwrap(),
        (Curve25519Scalar::from(50u64), 5)
    );
    assert_eq!(
        builder.try_consume_rho_evaluation().unwrap(),
        Curve25519Scalar::from(40u64)
    );
    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::TooFewChiLengths)
    ));
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::TooFewRhoLengths)
    ));
}

#[test]
fn missing_chi_or_rho_length_is_reported() {
    let mut missing_chi_builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations {
            rho_evaluations: indexmap! {
                2 => Curve25519Scalar::from(20u64),
            },
            ..Default::default()
        },
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![3],
        vec![2],
        0,
    );
    assert!(matches!(
        missing_chi_builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::ChiLengthNotFound)
    ));

    let mut missing_rho_builder = VerificationBuilderImpl::<Curve25519Scalar>::new(
        SumcheckMleEvaluations {
            chi_evaluations: indexmap! {
                3 => Curve25519Scalar::from(30u64),
            },
            ..Default::default()
        },
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![3],
        vec![2],
        0,
    );
    assert!(matches!(
        missing_rho_builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::RhoLengthNotFound)
    ));
}
