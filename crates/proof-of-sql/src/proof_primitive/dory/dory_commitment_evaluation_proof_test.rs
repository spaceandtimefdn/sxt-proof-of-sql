use super::{
    test_rng, DoryEvaluationProof, DoryProverPublicSetup, DoryScalar, DoryVerifierPublicSetup,
    ProverSetup, PublicParameters, VerifierSetup,
    PROVER_SETUP_4, PROVER_SETUP_6, PUBLIC_PARAMETERS_4, PUBLIC_PARAMETERS_6,
    VERIFIER_SETUP_4, VERIFIER_SETUP_6,
};
use crate::base::commitment::{commitment_evaluation_proof_test::*, CommitmentEvaluationProof};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn test_simple_ipa() {
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&PROVER_SETUP_4, 4),
        &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_4, 4),
    );
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&PROVER_SETUP_4, 3),
        &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_4, 3),
    );
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&PROVER_SETUP_6, 2),
        &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_6, 2),
    );
}

#[test]
fn test_random_ipa_with_length_1() {
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&PROVER_SETUP_4, 4),
        &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_4, 4),
    );
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&PROVER_SETUP_4, 3),
        &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_4, 3),
    );
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&PROVER_SETUP_6, 2),
        &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_6, 2),
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let lengths = [128, 100, 64, 50, 32, 20, 16, 10, 8, 5, 4, 3, 2];

    // Test with nu=4, offset=4
    for length in lengths {
        test_random_commitment_evaluation_proof::<DoryEvaluationProof>(
            length,
            0,
            &DoryProverPublicSetup::new(&PROVER_SETUP_4, 4),
            &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_4, 4),
        );
    }

    // Test with nu=4, offset=3
    for length in lengths {
        test_random_commitment_evaluation_proof::<DoryEvaluationProof>(
            length,
            0,
            &DoryProverPublicSetup::new(&PROVER_SETUP_4, 3),
            &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_4, 3),
        );
    }

    // Test with nu=6, offset=2
    for length in lengths {
        test_random_commitment_evaluation_proof::<DoryEvaluationProof>(
            length,
            0,
            &DoryProverPublicSetup::new(&PROVER_SETUP_6, 2),
            &DoryVerifierPublicSetup::new(&VERIFIER_SETUP_6, 2),
        );
    }
}

#[test]
fn we_can_serialize_and_deserialize_dory_evaluation_proofs() {
    let mut rng = test_rng();
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
        &DoryProverPublicSetup::new(&PROVER_SETUP_4, 3),
    );
    let encoded = postcard::to_allocvec(&proof).unwrap();
    let decoded: DoryEvaluationProof = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, proof);
}
