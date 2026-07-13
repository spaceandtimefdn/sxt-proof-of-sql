use super::SumcheckMleEvaluations;
use crate::{
    base::polynomial::{compute_rho_eval, compute_truncated_lagrange_basis_sum},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::SumcheckRandomScalars,
};
use num_traits::One;

#[test]
fn we_can_track_the_evaluation_of_mles_used_within_sumcheck() {
    let evaluation_point = [Curve25519Scalar::from(3u64), Curve25519Scalar::from(5u64)];
    let random_scalars = [
        Curve25519Scalar::from(123u64),
        Curve25519Scalar::from(456u64),
    ];

    let sumcheck_random_scalars = SumcheckRandomScalars::new(&random_scalars, 3, 2);

    let pcs_proof_evaluations = [Curve25519Scalar::from(42u64)];
    let evals = SumcheckMleEvaluations::new(
        3,
        [3, 3],
        [],
        &evaluation_point,
        &sumcheck_random_scalars,
        &pcs_proof_evaluations,
        &[],
    );
    let expected_eval = (Curve25519Scalar::one() - evaluation_point[0])
        * (Curve25519Scalar::one() - evaluation_point[1])
        * (Curve25519Scalar::one() - random_scalars[0])
        * (Curve25519Scalar::one() - random_scalars[1])
        + (evaluation_point[0])
            * (Curve25519Scalar::one() - evaluation_point[1])
            * (random_scalars[0])
            * (Curve25519Scalar::one() - random_scalars[1])
        + (Curve25519Scalar::one() - evaluation_point[0])
            * (evaluation_point[1])
            * (Curve25519Scalar::one() - random_scalars[0])
            * (random_scalars[1]);
    assert_eq!(evals.random_evaluation, expected_eval);

    let expected_eval = (Curve25519Scalar::one() - evaluation_point[0])
        * (Curve25519Scalar::one() - evaluation_point[1])
        + (evaluation_point[0]) * (Curve25519Scalar::one() - evaluation_point[1])
        + (Curve25519Scalar::one() - evaluation_point[0]) * (evaluation_point[1]);
    assert_eq!(
        *evals.chi_evaluations.values().next().unwrap(),
        expected_eval
    );
}

#[test]
fn we_can_track_rho_singleton_rho_256_and_pcs_evaluations() {
    let evaluation_point = [
        Curve25519Scalar::from(1u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
    ];
    let random_scalars = [
        Curve25519Scalar::from(11u64),
        Curve25519Scalar::from(12u64),
        Curve25519Scalar::from(13u64),
        Curve25519Scalar::from(14u64),
        Curve25519Scalar::from(15u64),
        Curve25519Scalar::from(16u64),
        Curve25519Scalar::from(17u64),
        Curve25519Scalar::from(18u64),
        Curve25519Scalar::from(19u64),
    ];
    let sumcheck_random_scalars = SumcheckRandomScalars::new(&random_scalars, 5, 9);
    let first_round_pcs_proof_evaluations =
        [Curve25519Scalar::from(21u64), Curve25519Scalar::from(22u64)];
    let final_round_pcs_proof_evaluations = [Curve25519Scalar::from(31u64)];

    let evals = SumcheckMleEvaluations::new(
        5,
        [5, 1, 5],
        [5, 3],
        &evaluation_point,
        &sumcheck_random_scalars,
        &first_round_pcs_proof_evaluations,
        &final_round_pcs_proof_evaluations,
    );

    assert_eq!(evals.chi_evaluations.len(), 2);
    assert_eq!(
        evals.chi_evaluations[&5],
        compute_truncated_lagrange_basis_sum(5, &evaluation_point)
    );
    assert_eq!(evals.rho_evaluations.len(), 2);
    assert_eq!(
        evals.rho_evaluations[&5],
        compute_rho_eval(5, &evaluation_point)
    );
    assert_eq!(
        evals.singleton_chi_evaluation,
        compute_truncated_lagrange_basis_sum(1, &evaluation_point)
    );
    assert_eq!(evals.rho_256_evaluation, Some(Curve25519Scalar::from(1u64)));
    assert_eq!(
        evals.first_round_pcs_proof_evaluations,
        first_round_pcs_proof_evaluations
    );
    assert_eq!(
        evals.final_round_pcs_proof_evaluations,
        final_round_pcs_proof_evaluations
    );
}
