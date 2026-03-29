/// Cached Dory test setup objects.
///
/// These are computed lazily **once per test-binary invocation** via
/// [`std::sync::OnceLock`].  `PublicParameters::test_rand`,
/// `ProverSetup::from`, and `VerifierSetup::from` each take on the order of
/// 10-15 s; by sharing a single instance across all tests in the same binary
/// the total wall-time for the test suite drops substantially.
///
/// # Usage
///
/// Before (expensive – setup re-run for every test):
/// ```ignore
/// let pp = PublicParameters::test_rand(4, &mut test_rng());
/// let prover_setup = ProverSetup::from(&pp);
/// let verifier_setup = VerifierSetup::from(&pp);
/// ```
///
/// After (cheap – setup computed at most once):
/// ```ignore
/// use crate::proof_primitive::dory::test_utility::{
///     prover_setup_for_testing, public_parameters_for_testing,
///     verifier_setup_for_testing,
/// };
/// let pp = public_parameters_for_testing();
/// let prover_setup = prover_setup_for_testing();
/// let verifier_setup = verifier_setup_for_testing();
/// ```
use std::sync::OnceLock;

use crate::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use ark_std::test_rng;

/// The `nu` (max_nu) value used for the shared test setups.
///
/// `nu = 4` means the setup covers up to 2^(2·4) = 256 rows, which is
/// sufficient for every standard test.  Tests that need a *larger* setup must
/// create their own; tests that need a *smaller* one can use the cached version
/// because Dory setups are hierarchical.
pub const TEST_SETUP_MAX_NU: usize = 4;

// ---------------------------------------------------------------------------
// Static storage
// ---------------------------------------------------------------------------

static PUBLIC_PARAMETERS: OnceLock<PublicParameters> = OnceLock::new();
static VERIFIER_SETUP: OnceLock<VerifierSetup> = OnceLock::new();

/// Newtype wrapper that lets us store `ProverSetup<'static>` in a `OnceLock`.
///
/// `ProverSetup<'_>` borrows from `PublicParameters`.  Since `PUBLIC_PARAMETERS`
/// lives for `'static` (it is in a `OnceLock`), the borrow is valid for the
/// lifetime of the process.  We annotate the wrapper with `Send + Sync` because:
/// * `PublicParameters` is `Send + Sync`,
/// * `ProverSetup` is read-only after construction,
/// * tests only ever share the reference immutably.
struct SendSyncProverSetup(ProverSetup<'static>);
// SAFETY: see doc comment above.
unsafe impl Send for SendSyncProverSetup {}
unsafe impl Sync for SendSyncProverSetup {}

static PROVER_SETUP: OnceLock<SendSyncProverSetup> = OnceLock::new();

// ---------------------------------------------------------------------------
// Public accessors
// ---------------------------------------------------------------------------

/// Return the shared [`PublicParameters`] test instance (computed once).
///
/// The RNG seed comes from [`ark_std::test_rng`], which is deterministic, so
/// results are reproducible across runs on the same platform.
pub fn public_parameters_for_testing() -> &'static PublicParameters {
    PUBLIC_PARAMETERS.get_or_init(|| {
        let mut rng = test_rng();
        PublicParameters::test_rand(TEST_SETUP_MAX_NU, &mut rng)
    })
}

/// Return the shared [`ProverSetup`] test instance (computed once).
///
/// Derived from [`public_parameters_for_testing`].
pub fn prover_setup_for_testing() -> &'static ProverSetup<'static> {
    // Initialise public parameters first so the borrow is valid.
    let pp: &'static PublicParameters = public_parameters_for_testing();
    &PROVER_SETUP
        .get_or_init(|| SendSyncProverSetup(ProverSetup::from(pp)))
        .0
}

/// Return the shared [`VerifierSetup`] test instance (computed once).
///
/// Derived from [`public_parameters_for_testing`].
pub fn verifier_setup_for_testing() -> &'static VerifierSetup {
    let pp: &'static PublicParameters = public_parameters_for_testing();
    VERIFIER_SETUP.get_or_init(|| VerifierSetup::from(pp))
}
