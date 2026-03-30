use super::*;
use crate::proof_primitive::dory::test_utils::dory_setup_cache;
use ark_std::test_rng;

/// Helper that returns the shared cached prover/verifier setups trimmed to `nu`.
///
/// Using the cache avoids re-running the expensive `PublicParameters::test_rand`
/// / `ProverSetup::from` / `VerifierSetup::from` on every test invocation.
fn get_setups(nu: usize) -> (DoryProverPublicSetup<'static>, DoryVerifierPublicSetup<'static>) {
    assert!(
        nu <= dory_setup_cache::MAX_NU,
        "Requested nu={nu} exceeds cached MAX_NU={}. \
         Either raise MAX_NU in dory_setup_cache or create a local setup.",
        dory_setup_cache::MAX_NU,
    );
    let ps = DoryProverPublicSetup::new(dory_setup_cache::prover_setup(), nu);
    let vs = DoryVerifierPublicSetup::new(dory_setup_cache::verifier_setup(), nu);
    (ps, vs)
}

#[test]
fn test_simple_ipa() {
    let (ps, vs) = get_setups(2);
    let mut rng = test_rng();
    let scalars = vec![F::rand(&mut rng), F::rand(&mut rng)];
    let point = vec![F::rand(&mut rng)];
    test_dory_proof(scalars, point, &ps, &vs);
}

#[test]
fn test_random_ipa_with_length_1() {
    let (ps, vs) = get_setups(2);
    let mut rng = test_rng();
    let scalars = vec![F::rand(&mut rng)];
    let point = vec![F::rand(&mut rng)];
    test_dory_proof(scalars, point, &ps, &vs);
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let (ps, vs) = get_setups(dory_setup_cache::MAX_NU);
    let mut rng = test_rng();
    for len in [1usize, 2, 3, 4, 7, 8, 15, 16] {
        let n_vars = len.next_power_of_two().trailing_zeros() as usize;
        let scalars: Vec<_> = (0..len).map(|_| F::rand(&mut rng)).collect();
        let point: Vec<_> = (0..n_vars).map(|_| F::rand(&mut rng)).collect();
        test_dory_proof(scalars, point, &ps, &vs);
    }
}
