use super::*;
use crate::proof_primitive::dory::test_utils::dory_setup_cache;
use ark_std::test_rng;

/// Helper: returns cached prover/verifier public setups for the given `nu`.
///
/// Using the shared cache means `PublicParameters::test_rand`, `ProverSetup::from`,
/// and `VerifierSetup::from` are executed **once** per process instead of once per
/// test function, dramatically cutting wall-clock test time.
fn get_setups(nu: usize) -> (DoryProverPublicSetup<'static>, DoryVerifierPublicSetup<'static>) {
    assert!(
        nu <= dory_setup_cache::MAX_NU,
        "Requested nu={nu} exceeds cached MAX_NU={}; raise MAX_NU or use a local setup.",
        dory_setup_cache::MAX_NU,
    );
    (
        DoryProverPublicSetup::new(dory_setup_cache::prover_setup(), nu),
        DoryVerifierPublicSetup::new(dory_setup_cache::verifier_setup(), nu),
    )
}

#[test]
fn test_simple_ipa() {
    let (ps, vs) = get_setups(2);
    let mut rng = test_rng();
    test_dory_commitment_evaluation_proof_with_length_2_prover_setup(&ps, &vs, &mut rng);
}

#[test]
fn test_random_ipa_with_length_1() {
    let (ps, vs) = get_setups(2);
    let mut rng = test_rng();
    test_dory_commitment_evaluation_proof_with_random_scalars_and_various_lengths(
        &[1],
        &ps,
        &vs,
        &mut rng,
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let (ps, vs) = get_setups(dory_setup_cache::MAX_NU);
    let mut rng = test_rng();
    test_dory_commitment_evaluation_proof_with_random_scalars_and_various_lengths(
        &[1, 2, 3, 4, 7, 8, 15, 16],
        &ps,
        &vs,
        &mut rng,
    );
}
