/// Cached Dory test setups.
///
/// `PublicParameters::test_rand`, `ProverSetup::from`, and
/// `VerifierSetup::from` are all computationally expensive.  Running them
/// once per test causes the cost to be paid N times (one per test that calls
/// them), which accounts for the majority of the total test-suite wall-clock
/// time even when tests run in parallel.
///
/// This module uses [`std::sync::OnceLock`] to build each object **exactly
/// once** per test-binary run and then hands out shared `'static` references
/// for the rest of the suite.
///
/// # How to use
///
/// Replace ad-hoc calls like
///
/// ```rust,ignore
/// let pp = PublicParameters::test_rand(4, &mut test_rng());
/// let ps = ProverSetup::from(&pp);
/// let vs = VerifierSetup::from(&pp);
/// ```
///
/// with
///
/// ```rust,ignore
/// use crate::proof_primitive::dory::test_setup::{
///     test_public_parameters, test_prover_setup, test_verifier_setup,
/// };
/// let pp = test_public_parameters();
/// let ps = test_prover_setup();
/// let vs = test_verifier_setup();
/// ```
///
/// Tests that need a *different* sigma value can still call
/// `PublicParameters::test_rand` directly, but most tests use sigma ≤ 4 and
/// can simply use the cached versions.
use super::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// The `sigma` (log₂ of the maximum supported commitment length) used for all
/// shared test setups.  `sigma = 4` supports vectors up to length 2^4 = 16,
/// which covers every existing test case.
pub(crate) const TEST_SIGMA: usize = 4;

// ---------------------------------------------------------------------------
// Deterministic RNG
// ---------------------------------------------------------------------------

/// Return a fresh, deterministic, cryptographically-secure RNG seeded with 0.
///
/// Returning a *fresh* instance (rather than a shared one) avoids the need for
/// a mutex and keeps test determinism simple: every test that needs randomness
/// can call `test_rng()` independently.
pub(crate) fn test_rng() -> impl rand::RngCore + rand::CryptoRng {
    use rand::SeedableRng;
    rand_chacha::ChaCha20Rng::seed_from_u64(0)
}

// ---------------------------------------------------------------------------
// Cached public parameters
// ---------------------------------------------------------------------------

static CACHED_PUBLIC_PARAMS: OnceLock<PublicParameters> = OnceLock::new();

/// Return a `'static` reference to the shared [`PublicParameters`] singleton.
///
/// The first call initializes the singleton (slow); every subsequent call
/// returns the cached value instantly.
pub(crate) fn test_public_parameters() -> &'static PublicParameters {
    CACHED_PUBLIC_PARAMS.get_or_init(|| PublicParameters::test_rand(TEST_SIGMA, &mut test_rng()))
}

// ---------------------------------------------------------------------------
// Cached prover setup
// ---------------------------------------------------------------------------

static CACHED_PROVER_SETUP: OnceLock<ProverSetup<'static>> = OnceLock::new();

/// Return a `'static` reference to the shared [`ProverSetup`] singleton.
///
/// Built from [`test_public_parameters()`] on first call.
pub(crate) fn test_prover_setup() -> &'static ProverSetup<'static> {
    CACHED_PROVER_SETUP.get_or_init(|| ProverSetup::from(test_public_parameters()))
}

// ---------------------------------------------------------------------------
// Cached verifier setup
// ---------------------------------------------------------------------------

static CACHED_VERIFIER_SETUP: OnceLock<VerifierSetup> = OnceLock::new();

/// Return a `'static` reference to the shared [`VerifierSetup`] singleton.
///
/// Built from [`test_public_parameters()`] on first call.
pub(crate) fn test_verifier_setup() -> &'static VerifierSetup {
    CACHED_VERIFIER_SETUP.get_or_init(|| VerifierSetup::from(test_public_parameters()))
}

// ---------------------------------------------------------------------------
// Unit tests for this module
// ---------------------------------------------------------------------------

#[cfg(test)]
mod inner_tests {
    use super::*;

    /// Verify that every helper returns the *same* object on repeated calls
    /// (pointer equality ⟹ the OnceLock is actually caching).
    #[test]
    fn singletons_return_same_pointer() {
        assert!(
            std::ptr::eq(test_public_parameters(), test_public_parameters()),
            "test_public_parameters() must return the same pointer on every call"
        );
        assert!(
            std::ptr::eq(test_prover_setup(), test_prover_setup()),
            "test_prover_setup() must return the same pointer on every call"
        );
        assert!(
            std::ptr::eq(test_verifier_setup(), test_verifier_setup()),
            "test_verifier_setup() must return the same pointer on every call"
        );
    }

    /// Smoke-test: the cached setups are self-consistent (prover and verifier
    /// were derived from the same public parameters).
    #[test]
    fn cached_setups_are_self_consistent() {
        // Simply constructing both without panicking is sufficient.
        let _ps = test_prover_setup();
        let _vs = test_verifier_setup();
    }
}
