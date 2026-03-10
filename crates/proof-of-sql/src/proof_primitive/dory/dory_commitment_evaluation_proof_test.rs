use super::{
    cached_prover_setup, cached_public_parameters, cached_verifier_setup, test_rng,
    DoryEvaluationProof, DoryProverPublicSetup, DoryScalar, DoryVerifierPublicSetup, ProverSetup,
};
use crate::base::commitment::{commitment_evaluation_proof_test::*, CommitmentEvaluationProof};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn test_simple_ipa() {
    let prover_setup = cached_prover_setup(4);
    let verifier_setup = cached_verifier_setup(4);
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(prover_setup, 4),
        &DoryVerifierPublicSetup::new(verifier_setup, 4),
    );
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(prover_setup, 3),
        &DoryVerifierPublicSetup::new(verifier_setup, 3),
    );
    let prover_setup = cached_prover_setup(6);
    let verifier_setup = cached_verifier_setup(6);
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(prover_setup, 2),
        &DoryVerifierPublicSetup::new(verifier_setup, 2),
    );
}

#[test]
fn test_random_ipa_with_length_1() {
    let prover_setup = cached_prover_setup(4);
    let verifier_setup = cached_verifier_setup(4);
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(prover_setup, 4),
        &DoryVerifierPublicSetup::new(verifier_setup, 4),
    );
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(prover_setup, 3),
        &DoryVerifierPublicSetup::new(verifier_setup, 3),
    );
    let prover_setup = cached_prover_setup(6);
    let verifier_setup = cached_verifier_setup(6);
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(prover_setup, 2),
        &DoryVerifierPublicSetup::new(verifier_setup, 2),
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let lengths = [128, 100, 64, 50, 32, 20, 16, 10, 8, 5, 4, 3, 2];
    let setup_setup = [(4, 4), (4, 3), (6, 2)];
    for setup_p in setup_setup {
        let prover_setup = cached_prover_setup(setup_p.0);
        let verifier_setup = cached_verifier_setup(setup_p.0);
        for length in lengths {
            test_random_commitment_evaluation_proof::<DoryEvaluationProof>(
                length,
                0,
                &DoryProverPublicSetup::new(prover_setup, setup_p.1),
                &DoryVerifierPublicSetup::new(verifier_setup, setup_p.1),
            );
        }
    }
}

#[test]
fn we_can_serialize_and_deserialize_dory_evaluation_proofs() {
    let mut rng = test_rng();
    let public_parameters = cached_public_parameters(4);
    let prover_setup = ProverSetup::from(public_parameters);
    let a = core::iter::repeat_with(|| DoryScalar::rand(&mut rng))
        .take(30)
        .collect::<Vec<_>>();
    let b_point = core::iter::repeat_with(|| DoryScalar::rand(&mut rng))
        .take(5)
        .collect::<Vec<_>>();
    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = DoryEvaluationProof::new(
        &mut transcript,
        &a,
        &b_point,
        0,
        &DoryProverPublicSetup::new(&prover_setup, 3),
    );
    let encoded = postcard::to_allocvec(&proof).unwrap();
    let decoded: DoryEvaluationProof = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, proof);
}
