use super::{
    test_rng, DoryScalar, DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
};
use crate::base::commitment::{commitment_evaluation_proof_test::*, CommitmentEvaluationProof};
use ark_std::UniformRand;
use merlin::Transcript;
use std::sync::LazyLock;

static PUBLIC_PARAMETERS_NU_2: LazyLock<PublicParameters> =
    LazyLock::new(|| PublicParameters::test_rand(2, &mut test_rng()));
static PUBLIC_PARAMETERS_NU_4: LazyLock<PublicParameters> =
    LazyLock::new(|| PublicParameters::test_rand(4, &mut test_rng()));
static PUBLIC_PARAMETERS_NU_6: LazyLock<PublicParameters> =
    LazyLock::new(|| PublicParameters::test_rand(6, &mut test_rng()));
static VERIFIER_SETUP_NU_2: LazyLock<VerifierSetup> =
    LazyLock::new(|| VerifierSetup::from(&*PUBLIC_PARAMETERS_NU_2));
static VERIFIER_SETUP_NU_4: LazyLock<VerifierSetup> =
    LazyLock::new(|| VerifierSetup::from(&*PUBLIC_PARAMETERS_NU_4));
static VERIFIER_SETUP_NU_6: LazyLock<VerifierSetup> =
    LazyLock::new(|| VerifierSetup::from(&*PUBLIC_PARAMETERS_NU_6));

fn public_parameters(max_nu: usize) -> &'static PublicParameters {
    match max_nu {
        4 => &PUBLIC_PARAMETERS_NU_4,
        6 => &PUBLIC_PARAMETERS_NU_6,
        _ => unreachable!("test public parameters are only cached for max_nu 4 and 6"),
    }
}

fn verifier_setup(max_nu: usize) -> &'static VerifierSetup {
    match max_nu {
        2 => &VERIFIER_SETUP_NU_2,
        4 => &VERIFIER_SETUP_NU_4,
        6 => &VERIFIER_SETUP_NU_6,
        _ => unreachable!("test verifier setup is only cached for max_nu 2, 4, and 6"),
    }
}

#[test]
fn test_simple_ipa() {
    let prover_setup = ProverSetup::from(public_parameters(4));
    let verifier_setup = verifier_setup(4);
    test_simple_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
        &&prover_setup,
        &verifier_setup,
    );
}

#[test]
fn test_random_ipa_with_length_1() {
    let prover_setup = ProverSetup::from(public_parameters(4));
    let verifier_setup = verifier_setup(4);
    test_commitment_evaluation_proof_with_length_1::<DynamicDoryEvaluationProof>(
        &&prover_setup,
        &verifier_setup,
    );
}

#[test]
#[should_panic = "verification improperly failed"]
fn test_random_ipa_fails_with_too_small_of_verifier_setup() {
    let prover_setup = ProverSetup::from(public_parameters(6));
    let verifier_setup = verifier_setup(2);
    test_random_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
        128,
        0,
        &&prover_setup,
        &verifier_setup,
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let lengths = [128, 100, 64, 50, 32, 20, 16, 10, 8, 5, 4, 3, 2];
    let prover_setup = ProverSetup::from(public_parameters(6));
    let verifier_setup = verifier_setup(6);
    for length in lengths {
        test_random_commitment_evaluation_proof::<DynamicDoryEvaluationProof>(
            length,
            0,
            &&prover_setup,
            &verifier_setup,
        );
    }
}

#[test]
fn we_can_serialize_and_deserialize_dory_evaluation_proofs() {
    let mut rng = test_rng();
    let public_parameters = PublicParameters::test_rand(4, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
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
