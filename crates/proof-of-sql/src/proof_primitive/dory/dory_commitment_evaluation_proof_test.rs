use super::test_setup::{test_prover_setup, test_verifier_setup};
use super::{DoryProverPublicSetup, DoryVerifierPublicSetup};
use crate::base::commitment::CommitmentEvaluationProof;
use crate::proof_primitive::dory::{DoryEvaluationProof, DoryScalar};
use ark_std::test_rng;
use merlin::Transcript;

/// Helper: build a random scalar vector of the given length using test_rng.
fn random_scalars(len: usize) -> Vec<DoryScalar> {
    use ark_std::UniformRand;
    let mut rng = test_rng();
    (0..len).map(|_| DoryScalar(ark_bn254::Fr::rand(&mut rng))).collect()
}

#[test]
fn test_dory_evaluation_proof_small() {
    let prover_setup = test_prover_setup();
    let verifier_setup = test_verifier_setup();

    // sigma = 2 supports vectors up to length 2^(2*2) = 16
    let prover_public_setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let verifier_public_setup = DoryVerifierPublicSetup::new(&verifier_setup, 2);

    let scalars = random_scalars(4);
    // b_point has length equal to the number of variables (log2 of vector length)
    let b_point: Vec<DoryScalar> = random_scalars(2);

    let mut prover_transcript = Transcript::new(b"dory_test");
    let mut verifier_transcript = Transcript::new(b"dory_test");

    // Use the CommitmentEvaluationProof trait's `new` method to build the proof.
    let proof = DoryEvaluationProof::new(
        &mut prover_transcript,
        &scalars,
        &b_point,
        0,
        &prover_public_setup,
    );

    // Verify the proof using the trait's verify_batched_proof method.
    let eval = proof.verify_batched_proof(
        &mut verifier_transcript,
        std::slice::from_ref(proof.commit()),
        &[DoryScalar::from(0u64)],
        &b_point,
        0,
        1,
        &verifier_public_setup,
    );
    assert!(eval.is_ok());
}

#[test]
fn test_cached_setup_identity() {
    use super::test_setup::test_public_parameters;
    // Verify that the OnceLock cache returns the same instance on repeated calls.
    let a = test_public_parameters() as *const _;
    let b = test_public_parameters() as *const _;
    assert_eq!(a, b);
}
