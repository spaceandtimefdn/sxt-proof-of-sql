/// Cached test setups for Dory to avoid recomputing expensive
/// `PublicParameters`, `ProverSetup`, and `VerifierSetup` across tests.
///
/// These are lazily initialized once and reused for the lifetime of the test
/// binary, significantly reducing total test-suite wall-clock time.
use super::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

/// The maximum `sigma` (log2 of the max commitment length) used in tests.
/// `sigma = 4` supports commitments up to length 2^4 = 16, which covers
/// all test cases. Increase if larger test vectors are ever needed.
pub(crate) const TEST_DORY_MAX_LOG_ROWS: usize = 4;

// ---------------------------------------------------------------------------
// Cached public parameters
// ---------------------------------------------------------------------------

static TEST_PUBLIC_PARAMETERS: OnceLock<PublicParameters> = OnceLock::new();

/// Return a reference to the shared `PublicParameters` used across all tests.
///
/// The parameters are generated once with a fixed seed for determinism and
/// then reused for every subsequent call.
pub(crate) fn test_public_parameters() -> &'static PublicParameters {
    TEST_PUBLIC_PARAMETERS
        .get_or_init(|| PublicParameters::test_rand(TEST_DORY_MAX_LOG_ROWS, &mut test_rng()))
}

// ---------------------------------------------------------------------------
// Cached prover setup
// ---------------------------------------------------------------------------

static TEST_PROVER_SETUP: OnceLock<ProverSetup<'static>> = OnceLock::new();

/// Return a reference to the shared `ProverSetup` used across all tests.
pub(crate) fn test_prover_setup() -> &'static ProverSetup<'static> {
    // We need the public parameters to live long enough.  Because
    // `test_public_parameters()` returns a `'static` reference we can safely
    // build a `'static` `ProverSetup` from it.
    TEST_PROVER_SETUP.get_or_init(|| ProverSetup::from(test_public_parameters()))
}

// ---------------------------------------------------------------------------
// Cached verifier setup
// ---------------------------------------------------------------------------

static TEST_VERIFIER_SETUP: OnceLock<VerifierSetup> = OnceLock::new();

/// Return a reference to the shared `VerifierSetup` used across all tests.
pub(crate) fn test_verifier_setup() -> &'static VerifierSetup {
    TEST_VERIFIER_SETUP.get_or_init(|| VerifierSetup::from(test_public_parameters()))
}

// ---------------------------------------------------------------------------
// Deterministic RNG helper
// ---------------------------------------------------------------------------

/// A tiny, fast, deterministic RNG that implements `rand::RngCore` so it can
/// be used wherever a `&mut impl Rng` is required.
///
/// We purposely use a fixed seed so that tests are reproducible.
pub(crate) fn test_rng() -> impl rand::RngCore + rand::CryptoRng {
    use rand::SeedableRng;
    rand_chacha::ChaCha20Rng::seed_from_u64(0)
}
