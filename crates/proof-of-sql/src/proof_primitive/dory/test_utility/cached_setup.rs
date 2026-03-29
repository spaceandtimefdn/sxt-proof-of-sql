/// Cached Dory test setup objects.
///
/// These are computed lazily once per test-binary run using [`std::sync::OnceLock`].
/// Because `PublicParameters::test_rand`, `ProverSetup::from`, and `VerifierSetup::from`
/// are expensive (each can take 10-15 s), re-using a single instance across all tests
/// that share the same binary cuts total test time substantially.
///
/// # Usage
///
/// Replace per-test setup boilerplate such as:
/// ```ignore
/// let pp = PublicParameters::test_rand(4, &mut test_rng());
/// let ps = ProverSetup::from(&pp);
/// let vs = VerifierSetup::from(&pp);
/// ```
/// with:
/// ```ignore
/// let pp = public_parameters_for_testing();
/// let ps = prover_setup_for_testing();
/// let vs = verifier_setup_for_testing();
/// ```
///
/// The `sigma` value (log₂ of the maximum commitment length) used for the cached
/// objects is [`TEST_SETUP_MAX_NU`].  If a specific test requires a *larger* setup it
/// must still create its own; if it only needs a *smaller* one the cached version works
/// because Dory setups are hierarchical.
use std::sync::OnceLock;

use crate::proof_primitive::dory::{
    blitzar_handle::BlitzarHandle, ProverSetup, PublicParameters, VerifierSetup,
};
use ark_std::test_rng;

/// The `nu` (i.e. `max_nu`) value used for the shared test setups.
///
/// `nu = 4` supports commitment lengths up to 2^(2·4) = 256 rows, which is
/// sufficient for every test that does not explicitly construct a larger setup.
/// Increase this value if new tests require larger commitments *and* you do not
/// want to create a per-test setup.
pub const TEST_SETUP_MAX_NU: usize = 4;

// ---------------------------------------------------------------------------
// Static holders
// ---------------------------------------------------------------------------

static PUBLIC_PARAMETERS: OnceLock<PublicParameters> = OnceLock::new();
static PROVER_SETUP_STORAGE: OnceLock<ProverSetup<'static>> = OnceLock::new();
static VERIFIER_SETUP_STORAGE: OnceLock<VerifierSetup> = OnceLock::new();
static BLITZAR_HANDLE: OnceLock<BlitzarHandle> = OnceLock::new();

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

/// Return a reference to the shared [`PublicParameters`] test instance.
///
/// Computed once with a deterministic RNG so results are reproducible.
pub fn public_parameters_for_testing() -> &'static PublicParameters {
    PUBLIC_PARAMETERS.get_or_init(|| {
        let mut rng = test_rng();
        PublicParameters::test_rand(TEST_SETUP_MAX_NU, &mut rng)
    })
}

/// Return a reference to the shared [`ProverSetup`] test instance derived from
/// [`public_parameters_for_testing`].
pub fn prover_setup_for_testing() -> &'static ProverSetup<'static> {
    // Ensure the public parameters are initialised first.
    let pp: &'static PublicParameters = public_parameters_for_testing();
    PROVER_SETUP_STORAGE.get_or_init(|| {
        // SAFETY: `pp` is `'static` because it is stored in a `OnceLock`.
        ProverSetup::from(pp)
    })
}

/// Return a reference to the shared [`VerifierSetup`] test instance derived from
/// [`public_parameters_for_testing`].
pub fn verifier_setup_for_testing() -> &'static VerifierSetup {
    let pp: &'static PublicParameters = public_parameters_for_testing();
    VERIFIER_SETUP_STORAGE.get_or_init(|| VerifierSetup::from(pp))
}

/// Return a reference to a shared [`BlitzarHandle`] suitable for tests.
pub fn blitzar_handle_for_testing() -> &'static BlitzarHandle {
    BLITZAR_HANDLE.get_or_init(|| BlitzarHandle::new(TEST_SETUP_MAX_NU))
}
