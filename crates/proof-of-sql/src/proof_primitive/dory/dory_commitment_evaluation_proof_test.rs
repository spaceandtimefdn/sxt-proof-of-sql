use super::{
    test_utils::dory_setup_cache::{prover_setup, public_parameters, verifier_setup, MAX_NU},
    DoryCommitmentEvaluationProof, DoryEvaluationProof, DoryProverPublicSetup,
    DoryVerifierPublicSetup,
};
use crate::base::{
    commitment::CommitmentEvaluationProof,
    database::{Column, ColumnType, OwnedTable, OwnedTableTestAccessor, TestAccessor},
    scalar::Scalar,
};
use ark_std::test_rng;

/// Uses the *shared* cached setup so this does not re-run the expensive
/// `PublicParameters::test_rand` / `ProverSetup::from` / `VerifierSetup::from`
/// computation.
fn setup() -> (
    &'static super::ProverSetup<'static>,
    &'static super::VerifierSetup,
) {
    (prover_setup(), verifier_setup())
}

#[test]
fn test_simple_ipa() {
    let (ps, vs) = setup();
    let prover_setup = DoryProverPublicSetup::new(ps, 2);
    let verifier_setup = DoryVerifierPublicSetup::new(vs, 2);

    let mut rng = test_rng();
    let scalars: Vec<_> = (0..2).map(|_| crate::base::scalar::Curve25519Scalar::rand(&mut rng)).collect();

    let point: Vec<_> = (0..1).map(|_| crate::base::scalar::Curve25519Scalar::rand(&mut rng)).collect();

    // Delegate to the real test logic via the evaluation proof API.
    DoryCommitmentEvaluationProof::rand_test(&scalars, &point, &prover_setup, &verifier_setup, &mut rng);
}

#[test]
fn test_random_ipa_with_length_1() {
    let (ps, vs) = setup();
    let prover_setup = DoryProverPublicSetup::new(ps, 2);
    let verifier_setup = DoryVerifierPublicSetup::new(vs, 2);

    let mut rng = test_rng();
    let scalars: Vec<_> = (0..1).map(|_| crate::base::scalar::Curve25519Scalar::rand(&mut rng)).collect();
    let point: Vec<_> = (0..1).map(|_| crate::base::scalar::Curve25519Scalar::rand(&mut rng)).collect();

    DoryCommitmentEvaluationProof::rand_test(&scalars, &point, &prover_setup, &verifier_setup, &mut rng);
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let (ps, vs) = setup();
    let prover_setup = DoryProverPublicSetup::new(ps, MAX_NU);
    let verifier_setup = DoryVerifierPublicSetup::new(vs, MAX_NU);

    let mut rng = test_rng();
    for len in [1, 2, 3, 4, 7, 8, 15, 16] {
        let scalars: Vec<_> = (0..len).map(|_| crate::base::scalar::Curve25519Scalar::rand(&mut rng)).collect();
        let point: Vec<_> = (0..len.next_power_of_two().trailing_zeros() as usize)
            .map(|_| crate::base::scalar::Curve25519Scalar::rand(&mut rng))
            .collect();
        DoryCommitmentEvaluationProof::rand_test(&scalars, &point, &prover_setup, &verifier_setup, &mut rng);
    }
}
