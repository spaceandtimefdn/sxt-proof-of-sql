/// Cached Dory test setups.
///
/// `PublicParameters::test_rand`, `ProverSetup::from`, and
/// `VerifierSetup::from` are all computationally expensive.  When tests call
/// them independently the cost is paid once *per test*, which can account for
/// the majority of the total test-suite wall-clock time.
///
/// This module uses [`std::sync::OnceLock`] to build each object exactly once
/// per test-binary run and then hand out shared references for the rest of the
/// suite.
///
/// # Usage
///
/// ```rust,ignore
/// use crate::proof_primitive::dory::test_setup::{
///     test_public_parameters, test_prover_setup, test_verifier_setup,
/// };
///
/// let pp  = test_public_parameters();
/// let ps  = test_prover_setup();
/// let vs  = test_verifier_setup();
/// ```
use super::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

/// `sigma` value used for all cached test setups.
///
/// `sigma = 4` means the setup supports vectors up to length 2^4 = 16, which
/// is enough for every existing test.  If a test needs a larger sigma it
/// should construct its own `PublicParameters` directly.
pub(crate) const TEST_SIGMA: usize = 4;

// ---- deterministic RNG --------------------------------------------------

/// Return a fresh deterministic RNG suitable for generating test parameters.
///
/// We use a fixed seed so that generated parameters are reproducible across
/// runs while still exercising non-trivial cryptographic material.
pub(crate) fn test_rng() -> impl rand::RngCore + rand::CryptoRng {
    use rand::SeedableRng;
    rand_chacha::ChaCha20Rng::seed_from_u64(0)
}

// ---- cached public parameters -------------------------------------------

static CACHED_PUBLIC_PARAMS: OnceLock<PublicParameters> = OnceLock::new();

/// Return a reference to the shared [`PublicParameters`] singleton.
///
/// The first call is slow (it runs the same computation that
/// `PublicParameters::test_rand` would run in every test), but every
/// subsequent call is essentially free.
pub(crate) fn test_public_parameters() -> &'static PublicParameters {
    CACHED_PUBLIC_PARAMS.get_or_init(|| PublicParameters::test_rand(TEST_SIGMA, &mut test_rng()))
}

// ---- cached prover setup ------------------------------------------------

static CACHED_PROVER_SETUP: OnceLock<ProverSetup<'static>> = OnceLock::new();

/// Return a reference to the shared [`ProverSetup`] singleton.
///
/// Built from [`test_public_parameters()`] on first call.
pub(crate) fn test_prover_setup() -> &'static ProverSetup<'static> {
    CACHED_PROVER_SETUP.get_or_init(|| ProverSetup::from(test_public_parameters()))
}

// ---- cached verifier setup ----------------------------------------------

static CACHED_VERIFIER_SETUP: OnceLock<VerifierSetup> = OnceLock::new();

/// Return a reference to the shared [`VerifierSetup`] singleton.
///
/// Built from [`test_public_parameters()`] on first call.
pub(crate) fn test_verifier_setup() -> &'static VerifierSetup {
    CACHED_VERIFIER_SETUP.get_or_init(|| VerifierSetup::from(test_public_parameters()))
}

// ---- tests for this module ----------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_setups_are_consistent() {
        // Calling twice must return the same pointer (not just equal values).
        assert!(
            std::ptr::eq(test_public_parameters(), test_public_parameters()),
            "PublicParameters singleton should be the same pointer on every call"
        );
        assert!(
            std::ptr::eq(test_prover_setup(), test_prover_setup()),
            "ProverSetup singleton should be the same pointer on every call"
        );
        assert!(
            std::ptr::eq(test_verifier_setup(), test_verifier_setup()),
            "VerifierSetup singleton should be the same pointer on every call"
        );
    }
}
