use super::{
    test_params_nu_4, test_params_nu_6, test_rng, test_verifier_setup_nu_4,
    test_verifier_setup_nu_6, DoryScalar, DynamicDoryEvaluationProof, ProverSetup,
    PublicParameters, VerifierSetup,
};
use crate::base::commitment::{commitment_evaluation_proof_test::*, CommitmentEvaluationProof};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn test_simple_ipa() {
    let pp4 = test_params_nu_4();
    let prover_setup = ProverSetup::from(pp4);
    let verifier_setup = test_verifier_setup_nu_4();
    test_simple_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
        &&prover_setup,
        &&verifier_setup,
    );
}

#[test]
fn test_random_ipa_with_length_1() {
    let pp4 = test_params_nu_4();
    let prover_setup = ProverSetup::from(pp4);
    let verifier_setup = test_verifier_setup_nu_4();
    test_commitment_evaluation_proof_with_length_1::<DynamicDoryEvaluationProof>(
        &&prover_setup,
        &&verifier_setup,
    );
}

#[test]
#[should_panic = "verification improperly failed"]
fn test_random_ipa_fails_with_too_small_of_verifier_setup() {
    let pp6 = test_params_nu_6();
    let prover_setup = ProverSetup::from(pp6);
    let public_parameters_small = PublicParameters::test_rand(2, &mut test_rng());
    let verifier_setup = VerifierSetup::from(&public_parameters_small);
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
    let pp6 = test_params_nu_6();
    let prover_setup = ProverSetup::from(pp6);
    let verifier_setup = test_verifier_setup_nu_6();
    for length in lengths {
        test_random_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
            length,
            0,
            &&prover_setup,
            &&verifier_setup,
        );
    }
}

#[test]
fn we_can_serialize_and_deserialize_dory_evaluation_proofs() {
    let mut rng = test_rng();
    let pp4 = test_params_nu_4();
    let prover_setup = ProverSetup::from(pp4);
    let a = core::iter::repeat_with(|| DoryScalar::rand(&mut rng))
        .take(30)
        .collect::<Vec<_>>();
    let b_point = core::iter::repeat_with(|| DoryScalar::rand(&mut rng))
        .take(5)
        .collect::<Vec<_>>();
    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = DynamicDoryEvaluationProof::new(&mut transcript, &a, &b_point, 0, &&prover_setup);
    let encoded = postcard::to_allocvec(&proof).unwrap();
    let decoded: DynamicDoryEvaluationProof = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, proof);
}
