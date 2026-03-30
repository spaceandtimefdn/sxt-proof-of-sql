/// Process-wide cache of expensive Dory test setups.
///
/// # Motivation
///
/// [`PublicParameters::test_rand`], [`ProverSetup::from`], and
/// [`VerifierSetup::from`] are the most expensive operations in the test suite
/// (each call can take 10–60 s). By storing the results in [`std::sync::OnceLock`]
/// statics we pay that cost at most **once per test-binary process** instead of
/// once per test function, dramatically reducing total test-suite wall time.
///
/// # Usage
///
/// Replace ad-hoc setup construction in test functions:
///
/// ```rust,ignore
/// // Before (expensive – recomputed in every test):
/// let mut rng = test_rng();
/// let pp  = PublicParameters::test_rand(4, &mut rng);
/// let ps  = ProverSetup::from(&pp);
/// let vs  = VerifierSetup::from(&pp);
///
/// // After (free after the first call in the process):
/// use crate::proof_primitive::dory::test_utils::dory_setup_cache::{
///     prover_setup, verifier_setup, public_parameters, MAX_NU,
/// };
/// let ps = prover_setup();
/// let vs = verifier_setup();
/// ```
///
/// Tests that need a *smaller* `nu` can safely pass any value `<= MAX_NU`
/// because Dory setups are nested (the first `2^nu` generators of a larger
/// setup are identical to a fresh setup of size `nu`).
///
/// Tests that genuinely require a *larger* setup must still construct their own.
use crate::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

/// Maximum `nu` (sigma) value for the cached test setup.
///
/// A `nu = 4` setup supports commitment sizes up to `2^4 = 16` columns, which
/// covers all existing tests that use the shared cache.
pub const MAX_NU: usize = 4;

// ---------------------------------------------------------------------------
// Statics
// ---------------------------------------------------------------------------

// `PublicParameters` must outlive the `ProverSetup` that borrows it; we achieve
// this by leaking a `Box` so that the reference has `'static` lifetime.
static PUBLIC_PARAMS: OnceLock<Box<PublicParameters>> = OnceLock::new();

/// A pair of (ProverSetup, VerifierSetup) derived from the cached
/// `PublicParameters`.
struct SetupPair {
    prover: ProverSetup<'static>,
    verifier: VerifierSetup,
}

// SAFETY: `ProverSetup` holds raw pointers into the `PublicParameters` data.
// The data lives for `'static` (leaked `Box`) and is never mutated after
// construction, so it is safe to share across threads via `Sync`.
unsafe impl Send for SetupPair {}
unsafe impl Sync for SetupPair {}

static SETUP_PAIR: OnceLock<SetupPair> = OnceLock::new();

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Returns a `'static` reference to the cached [`PublicParameters`].
///
/// The parameters are constructed once using [`ark_std::test_rng`] seeded
/// deterministically, so the result is the same every time.
///
/// **Cost:** expensive on first call; free thereafter.
pub fn public_parameters() -> &'static PublicParameters {
    PUBLIC_PARAMS.get_or_init(|| {
        use ark_std::test_rng;
        let mut rng = test_rng();
        Box::new(PublicParameters::test_rand(MAX_NU, &mut rng))
    })
}

/// Returns a `'static` reference to the cached [`ProverSetup`].
///
/// **Cost:** expensive on first call; free thereafter.
pub fn prover_setup() -> &'static ProverSetup<'static> {
    &get_or_init_pair().prover
}

/// Returns a `'static` reference to the cached [`VerifierSetup`].
///
/// **Cost:** expensive on first call; free thereafter.
pub fn verifier_setup() -> &'static VerifierSetup {
    &get_or_init_pair().verifier
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn get_or_init_pair() -> &'static SetupPair {
    SETUP_PAIR.get_or_init(|| {
        let pp: &'static PublicParameters = public_parameters();
        SetupPair {
            prover: ProverSetup::from(pp),
            verifier: VerifierSetup::from(pp),
        }
    })
}
