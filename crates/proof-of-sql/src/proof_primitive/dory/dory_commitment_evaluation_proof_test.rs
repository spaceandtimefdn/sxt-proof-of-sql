use super::test_setup::{test_prover_setup, test_public_parameters, test_verifier_setup};
use crate::{
    base::commitment::CommitmentEvaluationProof,
    proof_primitive::dory::{
        DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
    },
};
use ark_std::test_rng;

/// Helper: build a [`DoryProverPublicSetup`] from the cached test parameters.
fn prover_setup(sigma: usize) -> DoryProverPublicSetup<'static> {
    DoryProverPublicSetup::new(test_prover_setup(), sigma)
}

/// Helper: build a [`DoryVerifierPublicSetup`] from the cached test parameters.
fn verifier_setup(sigma: usize) -> DoryVerifierPublicSetup<'static> {
    DoryVerifierPublicSetup::new(test_verifier_setup(), sigma)
}

#[test]
fn test_dory_evaluation_proof_round_trip() {
    let sigma = 3;
    let ps = prover_setup(sigma);
    let vs = verifier_setup(sigma);
    let mut rng = test_rng();

    // Number of scalars: use a small vector that fits within max_nu = 4.
    let length = 8usize;
    let scalars: Vec<_> = (0..length)
        .map(|i| ark_bls12_381::Fr::from(i as u64))
        .collect();

    let b_point: Vec<_> = (0..sigma)
        .map(|i| ark_bls12_381::Fr::from((i + 1) as u64))
        .collect();

    let commit =
        DoryEvaluationProof::compute_commitments(&scalars, 0, &ps).expect("commitment failed");

    let proof = DoryEvaluationProof::new(
        &mut merlin::Transcript::new(b"test"),
        &scalars,
        0,
        &b_point,
        &mut rng,
        &ps,
    )
    .expect("proof creation failed");

    proof
        .verify_batched_proof(
            &mut merlin::Transcript::new(b"test"),
            &[commit],
            &[ark_bls12_381::Fr::from(1u64)],
            &b_point,
            &[ark_bls12_381::Fr::from(0u64)],
            0,
            1,
            &vs,
        )
        .expect("proof verification failed");
}

#[test]
fn test_dory_evaluation_proof_with_cached_setup() {
    // Verify that the cached public parameters are accessible and consistent.
    let pp = test_public_parameters();
    assert!(
        pp.max_nu() > 0,
        "PublicParameters should have a positive max_nu"
    );

    let ps = test_prover_setup();
    let vs = test_verifier_setup();

    // Confirm that setups derived from the same PublicParameters are coherent.
    let dory_ps = DoryProverPublicSetup::new(ps, 2);
    let dory_vs = DoryVerifierPublicSetup::new(vs, 2);

    let scalars: Vec<_> = (0..4)
        .map(|i| ark_bls12_381::Fr::from(i as u64))
        .collect();
    let b_point: Vec<_> = (0..2)
        .map(|i| ark_bls12_381::Fr::from((i + 1) as u64))
        .collect();

    let mut rng = test_rng();

    let commit = DoryEvaluationProof::compute_commitments(&scalars, 0, &dory_ps)
        .expect("commitment failed");

    let proof = DoryEvaluationProof::new(
        &mut merlin::Transcript::new(b"cached"),
        &scalars,
        0,
        &b_point,
        &mut rng,
        &dory_ps,
    )
    .expect("proof creation failed");

    proof
        .verify_batched_proof(
            &mut merlin::Transcript::new(b"cached"),
            &[commit],
            &[ark_bls12_381::Fr::from(1u64)],
            &b_point,
            &[ark_bls12_381::Fr::from(0u64)],
            0,
            1,
            &dory_vs,
        )
        .expect("proof verification failed");
}
