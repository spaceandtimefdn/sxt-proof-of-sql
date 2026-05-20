use super::SumcheckMleEvaluations;
use crate::{
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
fn we_can_compute_rho_256_evaluation_for_large_evaluation_points() {
    let evaluation_point = [
        Curve25519Scalar::from(1u64),
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(4u64),
        Curve25519Scalar::from(5u64),
        Curve25519Scalar::from(6u64),
        Curve25519Scalar::from(7u64),
        Curve25519Scalar::from(8u64),
        Curve25519Scalar::from(9u64),
    ];
    let random_scalars = evaluation_point;

    let sumcheck_random_scalars =
        SumcheckRandomScalars::new(&random_scalars, 4, evaluation_point.len());

    let evals = SumcheckMleEvaluations::new(
        4,
        [],
        [],
        &evaluation_point,
        &sumcheck_random_scalars,
        &[],
        &[],
    );

    let expected_rho_256_prefix = evaluation_point
        .iter()
        .take(8)
        .rev()
        .fold(Curve25519Scalar::from(0u64), |acc, &x| {
            acc * Curve25519Scalar::from(2u64) + x
        });
    let expected_rho_256 = evaluation_point
        .iter()
        .skip(8)
        .fold(expected_rho_256_prefix, |acc, &x| {
            acc * (Curve25519Scalar::one() - x)
        });

    assert_eq!(evals.rho_256_evaluation, Some(expected_rho_256));
}
