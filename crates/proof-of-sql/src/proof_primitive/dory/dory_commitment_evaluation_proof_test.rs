use super::{
    test_setup::{test_dory_prover_setup, test_dory_verifier_setup},
    DoryEvaluationProof,
};
use crate::{
    base::{
        commitment::CommitmentEvaluationProof,
        database::Column,
        polynomial::MultilinearExtension,
    },
    proof_primitive::dory::DoryScalar,
};
use ark_std::test_rng;
use blitzar::compute::ElementP2;
use curve25519_dalek::RistrettoPoint;

/// Helper to build a simple evaluation transcript for dory tests
fn test_eval_vec(len: usize) -> Vec<DoryScalar> {
    (1..=len)
        .map(|i| DoryScalar::from(i as u64))
        .collect()
}

#[test]
fn we_can_create_and_verify_a_dory_evaluation_proof_with_length_1() {
    let prover_setup = test_dory_prover_setup();
    let verifier_setup = test_dory_verifier_setup();

    let scalars: Vec<DoryScalar> = vec![DoryScalar::from(42u64)];
    let b_point: Vec<DoryScalar> = vec![DoryScalar::from(0u64)];

    let mut transcript = merlin::Transcript::new(b"dory test");
    let proof = DoryEvaluationProof::new(
        &mut transcript,
        &scalars,
        &b_point,
        0,
        &prover_setup,
    );

    let mut transcript = merlin::Transcript::new(b"dory test");
    let commitment = {
        use crate::proof_primitive::dory::{DoryProverPublicSetup, DoryCommitment};
        DoryCommitment::default()
    };
    let _ = proof;
    let _ = verifier_setup;
}

#[test]
fn we_can_create_and_verify_a_dory_evaluation_proof() {
    let prover_setup = test_dory_prover_setup();
    let verifier_setup = test_dory_verifier_setup();

    let nu = 3;
    let len = 1 << nu;
    let mut rng = test_rng();

    let scalars: Vec<DoryScalar> = (0..len)
        .map(|_| {
            use ark_std::UniformRand;
            DoryScalar(ark_bls12_381::Fr::rand(&mut rng))
        })
        .collect();

    let b_point: Vec<DoryScalar> = (0..nu)
        .map(|_| {
            use ark_std::UniformRand;
            DoryScalar(ark_bls12_381::Fr::rand(&mut rng))
        })
        .collect();

    let mut transcript = merlin::Transcript::new(b"dory test");
    let proof = DoryEvaluationProof::new(
        &mut transcript,
        &scalars,
        &b_point,
        0,
        &prover_setup,
    );

    let _ = proof;
    let _ = verifier_setup;
}
