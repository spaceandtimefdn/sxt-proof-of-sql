use super::{SumcheckMleEvaluations, VerificationBuilderImpl};
use crate::{
    base::{bit::BitDistribution, proof::ProofSizeMismatch},
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
fn we_can_consume_all_verification_builder_inputs() {
    let chi_eval = Curve25519Scalar::from(11);
    let rho_eval = Curve25519Scalar::from(13);
    let singleton_chi_eval = Curve25519Scalar::from(17);
    let random_eval = Curve25519Scalar::from(19);
    let first_round_evals = [Curve25519Scalar::from(23), Curve25519Scalar::from(29)];
    let final_round_evals = [Curve25519Scalar::from(31), Curve25519Scalar::from(37)];
    let rho_256_eval = Curve25519Scalar::from(41);
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations: [(4, chi_eval)].into_iter().collect(),
        rho_evaluations: [(5, rho_eval)].into_iter().collect(),
        singleton_chi_evaluation: singleton_chi_eval,
        random_evaluation: random_eval,
        first_round_pcs_proof_evaluations: &first_round_evals,
        final_round_pcs_proof_evaluations: &final_round_evals,
        rho_256_evaluation: Some(rho_256_eval),
    };
    let bit_distribution = BitDistribution {
        vary_mask: [1, 2, 3, 4],
        leading_bit_mask: [5, 6, 7, 8],
    };
    let bit_distributions = [bit_distribution.clone()];
    let subpolynomial_multipliers = [Curve25519Scalar::from(2), Curve25519Scalar::from(3)];
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &bit_distributions,
        &subpolynomial_multipliers,
        [Curve25519Scalar::from(43)].into(),
        vec![4],
        vec![5],
        2,
    );

    assert_eq!(builder.try_consume_chi_evaluation().unwrap(), (chi_eval, 4));
    assert_eq!(builder.try_consume_rho_evaluation().unwrap(), rho_eval);
    assert_eq!(
        builder.try_consume_first_round_mle_evaluations(2).unwrap(),
        first_round_evals
    );
    assert_eq!(
        builder.try_consume_final_round_mle_evaluations(2).unwrap(),
        final_round_evals
    );
    assert_eq!(
        builder.try_consume_bit_distribution().unwrap(),
        bit_distribution
    );
    assert_eq!(
        builder.try_consume_post_result_challenge().unwrap(),
        Curve25519Scalar::from(43)
    );
    assert_eq!(builder.singleton_chi_evaluation(), singleton_chi_eval);
    assert_eq!(builder.rho_256_evaluation(), Some(rho_256_eval));

    builder
        .try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            Curve25519Scalar::from(5),
            1,
        )
        .unwrap();
    builder
        .try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(7),
            2,
        )
        .unwrap();

    let expected_sumcheck_evaluation =
        subpolynomial_multipliers[0] * Curve25519Scalar::from(5) * random_eval
            + subpolynomial_multipliers[1] * Curve25519Scalar::from(7);
    assert_eq!(builder.sumcheck_evaluation(), expected_sumcheck_evaluation);
}

#[test]
fn we_report_verification_builder_size_mismatches() {
    let first_round_evals = [Curve25519Scalar::from(1)];
    let final_round_evals = [Curve25519Scalar::from(2)];
    let mle_evaluations = SumcheckMleEvaluations {
        chi_evaluations: [(2, Curve25519Scalar::from(3))].into_iter().collect(),
        rho_evaluations: [(3, Curve25519Scalar::from(4))].into_iter().collect(),
        first_round_pcs_proof_evaluations: &first_round_evals,
        final_round_pcs_proof_evaluations: &final_round_evals,
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        vec![99],
        vec![98],
        0,
    );

    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::ChiLengthNotFound)
    ));
    assert!(matches!(
        builder.try_consume_chi_evaluation(),
        Err(ProofSizeMismatch::TooFewChiLengths)
    ));
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::RhoLengthNotFound)
    ));
    assert!(matches!(
        builder.try_consume_rho_evaluation(),
        Err(ProofSizeMismatch::TooFewRhoLengths)
    ));
    assert!(matches!(
        builder.try_consume_first_round_mle_evaluations(2),
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    ));
    assert!(matches!(
        builder.try_consume_final_round_mle_evaluations(2),
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
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(5),
            0,
        ),
        Err(ProofSizeMismatch::ConstraintCountMismatch)
    ));

    let subpolynomial_multipliers = [Curve25519Scalar::from(1)];
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
            Curve25519Scalar::from(5),
            1,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));
    assert!(matches!(
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            Curve25519Scalar::from(5),
            2,
        ),
        Err(ProofSizeMismatch::SumcheckProofTooSmall)
    ));
}
