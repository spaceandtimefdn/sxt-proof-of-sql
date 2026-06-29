use super::{
    dory_commitment_evaluation_proof::DoryError, test_rng, DoryEvaluationProof,
    DoryProverPublicSetup, DoryScalar, DoryVerifierPublicSetup, ProverSetup, PublicParameters,
    VerifierSetup,
};
use crate::base::{
    commitment::{
        commitment_evaluation_proof_test::*, CommitmentEvaluationProof, VecCommitmentExt,
    },
    database::Column,
};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn test_simple_ipa() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&prover_setup, 4),
        &DoryVerifierPublicSetup::new(&verifier_setup, 4),
    );
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&prover_setup, 3),
        &DoryVerifierPublicSetup::new(&verifier_setup, 3),
    );
    let public_parameters = PublicParameters::test_rand(6, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&prover_setup, 2),
        &DoryVerifierPublicSetup::new(&verifier_setup, 2),
    );
}

#[test]
fn test_random_ipa_with_length_1() {
    let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&prover_setup, 4),
        &DoryVerifierPublicSetup::new(&verifier_setup, 4),
    );
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&prover_setup, 3),
        &DoryVerifierPublicSetup::new(&verifier_setup, 3),
    );
    let public_parameters = PublicParameters::test_rand(6, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &DoryProverPublicSetup::new(&prover_setup, 2),
        &DoryVerifierPublicSetup::new(&verifier_setup, 2),
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let lengths = [128, 100, 64, 50, 32, 20, 16, 10, 8, 5, 4, 3, 2];
    let setup_setup = [(4, 4), (4, 3), (6, 2)];
    for setup_p in setup_setup {
        let public_parameters = PublicParameters::test_rand(setup_p.0, &mut test_rng());
        let prover_setup = ProverSetup::from(&public_parameters);
        let verifier_setup = VerifierSetup::from(&public_parameters);
        for length in lengths {
            test_random_commitment_evaluation_proof::<DoryEvaluationProof>(
                length,
                0,
                &DoryProverPublicSetup::new(&prover_setup, setup_p.1),
                &DoryVerifierPublicSetup::new(&verifier_setup, setup_p.1),
            );
        }
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

#[test]
fn we_get_empty_dory_evaluation_proofs_for_invalid_prover_inputs() {
    let public_parameters = PublicParameters::test_rand(2, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let a = [DoryScalar::from(1), DoryScalar::from(2)];
    let b_point = [DoryScalar::from(3)];

    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = DoryEvaluationProof::new(&mut transcript, &a, &b_point, 1, &setup);
    assert_eq!(proof, DoryEvaluationProof::default());

    let oversized_b_point = vec![DoryScalar::from(3); 5];
    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = DoryEvaluationProof::new(&mut transcript, &a, &oversized_b_point, 0, &setup);
    assert_eq!(proof, DoryEvaluationProof::default());
}

#[test]
fn we_get_an_error_when_verifier_setup_is_too_small() {
    let public_parameters = PublicParameters::test_rand(6, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let prover_setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let small_public_parameters = PublicParameters::test_rand(2, &mut test_rng());
    let small_verifier_setup = VerifierSetup::from(&small_public_parameters);
    let small_verifier_setup = DoryVerifierPublicSetup::new(&small_verifier_setup, 2);
    let a = (1..=8).map(DoryScalar::from).collect::<Vec<_>>();
    let b_point = vec![DoryScalar::from(3); 5];

    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = DoryEvaluationProof::new(&mut transcript, &a, &b_point, 0, &prover_setup);
    let commits = Vec::from_columns_with_offset([Column::Scalar(&a)], 0, &prover_setup);

    let mut transcript = Transcript::new(b"evaluation_proof");
    let err = proof
        .verify_proof(
            &mut transcript,
            &commits[0],
            &DoryScalar::from(0),
            &b_point,
            0,
            a.len(),
            &small_verifier_setup,
        )
        .unwrap_err();

    assert!(matches!(
        err,
        DoryError::SmallSetup {
            actual: 2,
            required: 3
        }
    ));
}
