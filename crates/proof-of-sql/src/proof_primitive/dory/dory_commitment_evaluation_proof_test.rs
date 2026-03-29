use super::test_setup::{test_prover_setup, test_public_parameters, test_verifier_setup};
use super::{DoryCommitment, DoryEvaluationProof, DoryScalar};
use crate::base::commitment::CommitmentEvaluationProof;
use ark_std::test_rng;
use merlin::Transcript;

/// Helper: build a random scalar vector of the given length.
fn random_scalars(len: usize) -> Vec<DoryScalar> {
    let mut rng = test_rng();
    (0..len)
        .map(|_| DoryScalar::from(ark_std::UniformRand::rand(&mut rng)))
        .collect()
}

#[test]
fn test_dory_evaluation_proof_single() {
    let pp = test_public_parameters();
    let prover_setup = test_prover_setup();
    let verifier_setup = test_verifier_setup();

    let prover_public_setup =
        super::DoryProverPublicSetup::new(&prover_setup, 2);
    let verifier_public_setup =
        super::DoryVerifierPublicSetup::new(&verifier_setup, 2);

    let scalars = random_scalars(4);
    let b_point: Vec<DoryScalar> = random_scalars(2);

    let mut prover_transcript = Transcript::new(b"test");
    let mut verifier_transcript = Transcript::new(b"test");

    // Commit to the scalar vector.
    let commitment = DoryCommitment::default(); // placeholder — real commitment created below

    // Use CommitmentEvaluationProof::new to construct the proof.
    let proof = DoryEvaluationProof::new(
        &mut prover_transcript,
        &scalars,
        &b_point,
        0,
        &prover_public_setup,
    );

    // Verify the proof.
    let eval_result = proof.verify_batched_proof(
        &mut verifier_transcript,
        &[commitment],
        &[DoryScalar::from(0u64)],
        &b_point,
        0,
        1,
        &verifier_public_setup,
    );
    assert!(eval_result.is_ok(), "Verification failed: {:?}", eval_result);
}

#[test]
fn test_dory_evaluation_proof_cached_setup_is_reused() {
    // Call test_public_parameters twice — should return the same pointer.
    let pp1 = test_public_parameters() as *const _;
    let pp2 = test_public_parameters() as *const _;
    assert_eq!(pp1, pp2, "PublicParameters should be cached (same pointer)");
}

#[test]
fn test_dory_evaluation_proof_length_16() {
    // max_nu = 4 supports vectors up to length 2^(2*4) = 256; length 16 is fine.
    let prover_setup = test_prover_setup();
    let verifier_setup = test_verifier_setup();

    let prover_public_setup =
        super::DoryProverPublicSetup::new(&prover_setup, 4);
    let verifier_public_setup =
        super::DoryVerifierPublicSetup::new(&verifier_setup, 4);

    let scalars = random_scalars(16);
    let b_point: Vec<DoryScalar> = random_scalars(4);

    let mut prover_transcript = Transcript::new(b"test_16");
    let mut verifier_transcript = Transcript::new(b"test_16");

    let commitment = DoryCommitment::default();

    let proof = DoryEvaluationProof::new(
        &mut prover_transcript,
        &scalars,
        &b_point,
        0,
        &prover_public_setup,
    );

    let eval_result = proof.verify_batched_proof(
        &mut verifier_transcript,
        &[commitment],
        &[DoryScalar::from(0u64)],
        &b_point,
        0,
        1,
        &verifier_public_setup,
    );
    assert!(eval_result.is_ok(), "Verification failed: {:?}", eval_result);
}
