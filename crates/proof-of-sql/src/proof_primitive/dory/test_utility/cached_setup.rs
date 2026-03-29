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
/// The `nu` value (log₂ of the maximum commitment size) used for the cached
/// objects is [`TEST_SETUP_MAX_NU`].  If a specific test requires a *larger* setup it
/// must still create its own; if it only needs a *smaller* one the cached version works
/// because Dory setups are hierarchical.
use std::sync::OnceLock;

use crate::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use ark_std::test_rng;

/// The `nu` value used for the shared test setups.
///
/// `nu = 4` supports commitment lengths up to 2^(2*4) = 256 rows, which is
/// sufficient for every existing test that does not explicitly construct a
/// larger setup.  Increase this constant (and re-run) if new tests require
/// larger commitments *and* you do not want to create a per-test setup.
pub const TEST_SETUP_MAX_NU: usize = 4;

// ---------------------------------------------------------------------------
// Static holders
// ---------------------------------------------------------------------------

static PUBLIC_PARAMETERS: OnceLock<PublicParameters> = OnceLock::new();
static VERIFIER_SETUP: OnceLock<VerifierSetup> = OnceLock::new();

// ProverSetup borrows from PublicParameters, so we store it as a raw-pointer
// wrapper that is safe because the public parameters live for `'static`.
struct StaticProverSetup(ProverSetup<'static>);
// SAFETY: tests run in a single process; setup is written once and only read
// afterwards.  The inner `ProverSetup` contains no interior mutability beyond
// what the GPU/CPU MSM back-end itself protects.
unsafe impl Send for StaticProverSetup {}
unsafe impl Sync for StaticProverSetup {}

static PROVER_SETUP: OnceLock<StaticProverSetup> = OnceLock::new();

// ---------------------------------------------------------------------------
// Public accessors
// ---------------------------------------------------------------------------

/// Return a reference to the shared [`PublicParameters`] test instance.
///
/// Computed once with a deterministic RNG so results are reproducible across
/// runs.
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
    &PROVER_SETUP
        .get_or_init(|| StaticProverSetup(ProverSetup::from(pp)))
        .0
}

/// Return a reference to the shared [`VerifierSetup`] test instance derived from
/// [`public_parameters_for_testing`].
pub fn verifier_setup_for_testing() -> &'static VerifierSetup {
    let pp: &'static PublicParameters = public_parameters_for_testing();
    VERIFIER_SETUP.get_or_init(|| VerifierSetup::from(pp))
}
