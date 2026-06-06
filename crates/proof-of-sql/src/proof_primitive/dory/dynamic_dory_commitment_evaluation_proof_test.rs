use super::{
    test_rng, DoryScalar, DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    PROVER_SETUP_4, PROVER_SETUP_6, VERIFIER_SETUP_4, VERIFIER_SETUP_6,
};
use crate::base::commitment::{commitment_evaluation_proof_test::*, CommitmentEvaluationProof};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn test_simple_ipa() {
    test_simple_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
        &&*PROVER_SETUP_4,
        &&*VERIFIER_SETUP_4,
    );
}

#[test]
fn test_random_ipa_with_length_1() {
    test_commitment_evaluation_proof_with_length_1::<DynamicDoryEvaluationProof>(
        &&*PROVER_SETUP_4,
        &&*VERIFIER_SETUP_4,
    );
}

#[test]
#[should_panic = "verification improperly failed"]
fn test_random_ipa_fails_with_too_small_of_verifier_setup() {
    let public_parameters = PublicParameters::test_rand(6, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let public_parameters = PublicParameters::test_rand(2, &mut test_rng());
    let verifier_setup = VerifierSetup::from(&public_parameters);
    test_random_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
        128,
        0,
        &&prover_setup,
        &&verifier_setup,
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let lengths = [128, 100, 64, 50, 32, 20, 16, 10, 8, 5, 4, 3, 2];
    for length in lengths {
        test_random_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
            length,
            0,
            &&*PROVER_SETUP_6,
            &&*VERIFIER_SETUP_6,
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
    let proof = DynamicDoryEvaluationProof::new(&mut transcript, &a, &b_point, 0, &&*PROVER_SETUP_4);
    let encoded = postcard::to_allocvec(&proof).unwrap();
    let decoded: DynamicDoryEvaluationProof = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, proof);
}
