use super::test_setup::{test_prover_setup, test_verifier_setup};
use super::{DoryProverPublicSetup, DoryVerifierPublicSetup};
use crate::base::commitment::CommitmentEvaluationProof;
use crate::proof_primitive::dory::{DoryEvaluationProof, DoryScalar};
use ark_std::{test_rng, UniformRand};
use merlin::Transcript;

fn random_scalars(len: usize) -> Vec<DoryScalar> {
    let mut rng = test_rng();
    (0..len).map(|_| DoryScalar(ark_bn254::Fr::rand(&mut rng))).collect()
}

/// Test that a Dory evaluation proof can be created and verified using
/// the cached test setup helpers.
#[test]
fn test_dory_evaluation_proof_with_cached_setup() {
    let prover_setup = test_prover_setup();
    let verifier_setup = test_verifier_setup();

    let prover_public_setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let verifier_public_setup = DoryVerifierPublicSetup::new(&verifier_setup, 2);

    let nu = prover_public_setup.nu();
    let len = 1 << nu; // 2^nu scalars
    let scalars = random_scalars(len);
    let b_point: Vec<DoryScalar> = random_scalars(nu);

    let mut prover_transcript = Transcript::new(b"dory_eval_proof_test");
    let mut verifier_transcript = Transcript::new(b"dory_eval_proof_test");

    let proof = DoryEvaluationProof::new(
        &mut prover_transcript,
        &scalars,
        &b_point,
        0,
        &prover_public_setup,
    );

    let eval = proof.verify_batched_proof(
        &mut verifier_transcript,
        std::slice::from_ref(proof.commit()),
        &[DoryScalar::from(0u64)],
        &b_point,
        0,
        1,
        &verifier_public_setup,
    );
    assert!(eval.is_ok(), "Dory evaluation proof verification failed: {:?}", eval);
}

/// Verify that the OnceLock cache returns the same PublicParameters instance
/// across multiple calls (i.e. initialization runs only once).
#[test]
fn test_cached_public_parameters_are_reused() {
    use super::test_setup::test_public_parameters;
    let ptr_a = test_public_parameters() as *const _;
    let ptr_b = test_public_parameters() as *const _;
    assert_eq!(
        ptr_a, ptr_b,
        "PublicParameters should be initialized only once (OnceLock cache)"
    );
}
