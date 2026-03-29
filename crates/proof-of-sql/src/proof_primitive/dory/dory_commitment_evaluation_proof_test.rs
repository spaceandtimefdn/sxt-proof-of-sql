use super::{
    test_utility::{prover_setup_for_testing, public_parameters_for_testing, verifier_setup_for_testing},
    DoryCommitment, DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
};
use crate::{
    base::{commitment::CommitmentEvaluationProof, database::Column},
    proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup},
};
use ark_std::test_rng;
use merlin::Transcript;

/// A simple IPA test using the cached shared setup.
#[test]
fn test_simple_ipa() {
    let pp = public_parameters_for_testing();
    let prover_setup = prover_setup_for_testing();
    let verifier_setup = verifier_setup_for_testing();
    let dory_prover_setup = DoryProverPublicSetup::new(prover_setup, 3);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(verifier_setup, 3);

    let mut rng = test_rng();
    let column_length = 1 << 6;
    let challenge = [0u8; 32];

    let a: Vec<u64> = (0..column_length).collect();
    let column = Column::BigInt(&a);

    let (commitment, _) = DoryCommitment::compute_commitments(&[column], 0, &dory_prover_setup);
    let b_point: Vec<ark_bn254::Fr> = (0..6).map(|i| ark_bn254::Fr::from(i as u64 + 1)).collect();

    let mut transcript = Transcript::new(b"test");
    let proof = DoryEvaluationProof::new(
        &mut transcript,
        &a,
        &b_point,
        0,
        &dory_prover_setup,
    );

    let mut transcript = Transcript::new(b"test");
    let _ = proof.verify_batched_proof(
        &mut transcript,
        &[commitment],
        &challenge,
        &b_point,
        &[],
        0,
        0,
        &dory_verifier_setup,
    );
}

/// Parametric IPA test with a single scalar using the cached shared setup.
#[test]
fn test_random_ipa_with_length_1() {
    let pp = public_parameters_for_testing();
    let prover_setup = prover_setup_for_testing();
    let verifier_setup = verifier_setup_for_testing();
    let dory_prover_setup = DoryProverPublicSetup::new(prover_setup, 1);
    let dory_verifier_setup = DoryVerifierPublicSetup::new(verifier_setup, 1);

    let mut rng = test_rng();
    let a = vec![1u64];
    let column = Column::BigInt(&a);

    let (commitment, _) = DoryCommitment::compute_commitments(&[column], 0, &dory_prover_setup);
    let b_point: Vec<ark_bn254::Fr> = vec![ark_bn254::Fr::from(2u64)];

    let challenge = [0u8; 32];
    let mut transcript = Transcript::new(b"test");
    let proof = DoryEvaluationProof::new(
        &mut transcript,
        &a,
        &b_point,
        0,
        &dory_prover_setup,
    );

    let mut transcript = Transcript::new(b"test");
    let _ = proof.verify_batched_proof(
        &mut transcript,
        &[commitment],
        &challenge,
        &b_point,
        &[],
        0,
        0,
        &dory_verifier_setup,
    );
}

/// Parametric IPA test with various lengths using the cached shared setup.
#[test]
fn test_random_ipa_with_various_lengths() {
    let pp = public_parameters_for_testing();
    let prover_setup = prover_setup_for_testing();
    let verifier_setup = verifier_setup_for_testing();

    let mut rng = test_rng();
    for nu in 1..=4usize {
        let dory_prover_setup = DoryProverPublicSetup::new(prover_setup, nu);
        let dory_verifier_setup = DoryVerifierPublicSetup::new(verifier_setup, nu);
        let column_length = 1 << (2 * nu);
        let a: Vec<u64> = (0..column_length).collect();
        let column = Column::BigInt(&a);
        let (commitment, _) =
            DoryCommitment::compute_commitments(&[column], 0, &dory_prover_setup);
        let b_point: Vec<ark_bn254::Fr> = (0..(2 * nu))
            .map(|i| ark_bn254::Fr::from(i as u64 + 1))
            .collect();
        let challenge = [0u8; 32];
        let mut transcript = Transcript::new(b"test");
        let proof = DoryEvaluationProof::new(
            &mut transcript,
            &a,
            &b_point,
            0,
            &dory_prover_setup,
        );
        let mut transcript = Transcript::new(b"test");
        let _ = proof.verify_batched_proof(
            &mut transcript,
            &[commitment],
            &challenge,
            &b_point,
            &[],
            0,
            0,
            &dory_verifier_setup,
        );
    }
}
