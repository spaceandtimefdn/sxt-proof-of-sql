use super::SumcheckMleEvaluations;
use crate::{
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::SumcheckRandomScalars,
};
use alloc::{vec, vec::Vec};
use num_traits::{One, Zero};

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

fn compute_rho_256_evaluation(point: &[u64]) -> Option<Curve25519Scalar> {
    let evaluation_point: Vec<_> = point.iter().copied().map(Curve25519Scalar::from).collect();
    let random_scalars = vec![Curve25519Scalar::zero(); point.len()];
    let sumcheck_random_scalars =
        SumcheckRandomScalars::new(&random_scalars, 1, evaluation_point.len());

    SumcheckMleEvaluations::new(
        1,
        [],
        [],
        &evaluation_point,
        &sumcheck_random_scalars,
        &[],
        &[],
    )
    .rho_256_evaluation
}

#[test]
fn rho_256_evaluation_uses_little_endian_bits_and_gates_higher_variables() {
    let five = [1, 0, 1, 0, 0, 0, 0, 0];

    assert_eq!(
        compute_rho_256_evaluation(&five),
        Some(Curve25519Scalar::from(5_u64))
    );
    assert_eq!(
        compute_rho_256_evaluation(&[1, 0, 1, 0, 0, 0, 0, 0, 0, 0]),
        Some(Curve25519Scalar::from(5_u64))
    );
    assert_eq!(
        compute_rho_256_evaluation(&[1, 0, 1, 0, 0, 0, 0, 0, 0, 1]),
        Some(Curve25519Scalar::zero())
    );
}
