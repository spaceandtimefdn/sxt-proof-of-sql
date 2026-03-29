/// Cached Dory test setup helpers using `OnceLock` singletons.
///
/// These helpers avoid re-generating expensive cryptographic parameters on
/// every test by computing them once and reusing the result for the lifetime
/// of the test process.
///
/// # Parameters
///
/// The helpers use `max_nu = 4`.  `max_nu` controls the maximum number of
/// doublings that the Dory protocol can handle; concretely it bounds the
/// supported vector length as `2^(2 * max_nu)` = 2^8 = 256 scalar elements.
/// This is sufficient for the unit-test workloads in this crate.
///
/// Note: `max_nu` is a property of the [`PublicParameters`] / Dory setup and
/// is distinct from the `sigma` parameter used in
/// [`DoryProverPublicSetup`] / [`DoryVerifierPublicSetup`], which is a
/// separate commitment parameter.
use crate::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

/// A small `max_nu` value that is sufficient for unit tests.
///
/// With `max_nu = 4` the Dory setup supports vectors of up to
/// 2^(2 * 4) = 256 scalar elements.
const TEST_MAX_NU: usize = 4;

/// Returns a reference to a process-wide cached [`PublicParameters`] instance
/// suitable for unit tests.
///
/// The parameters are generated once using a deterministic RNG seeded with
/// `[0u8; 32]` and then reused for every subsequent call.
pub fn test_public_parameters() -> &'static PublicParameters {
    static INSTANCE: OnceLock<PublicParameters> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use ark_std::test_rng;
        PublicParameters::test_rand(TEST_MAX_NU, &mut test_rng())
    })
}

/// Returns a reference to a process-wide cached [`ProverSetup`] instance
/// suitable for unit tests.
pub fn test_prover_setup() -> &'static ProverSetup<'static> {
    static INSTANCE: OnceLock<ProverSetup<'static>> = OnceLock::new();
    INSTANCE.get_or_init(|| ProverSetup::from(test_public_parameters()))
}

/// Returns a reference to a process-wide cached [`VerifierSetup`] instance
/// suitable for unit tests.
pub fn test_verifier_setup() -> &'static VerifierSetup {
    static INSTANCE: OnceLock<VerifierSetup> = OnceLock::new();
    INSTANCE.get_or_init(|| VerifierSetup::from(test_public_parameters()))
}
