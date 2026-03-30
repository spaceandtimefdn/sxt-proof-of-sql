/// Cached Dory test setups to avoid re-computing expensive cryptographic parameters
/// across multiple tests in the same test binary process.
///
/// `PublicParameters::test_rand`, `ProverSetup::from`, and `VerifierSetup::from`
/// are expensive operations. By caching them in a `OnceLock` we pay the cost at most
/// once per test-binary invocation rather than once per test function.
use crate::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

/// The maximum `nu` value (log2 of the max supported commitment size) used in
/// the shared test setup cache. Individual tests that need a *larger* setup must
/// create their own; tests that need a *smaller* setup can simply use a prefix of
/// the cached one.
pub const MAX_NU: usize = 4;

// ---------------------------------------------------------------------------
// Internal static caches
// ---------------------------------------------------------------------------

struct DorySetupCache {
    public_params: PublicParameters,
    prover_setup: ProverSetup<'static>,
    verifier_setup: VerifierSetup,
}

// SAFETY: PublicParameters / ProverSetup / VerifierSetup contain raw pointers
// (blst affine points) but those are always accessed read-only after construction
// and the storage they reference lives for `'static`.
unsafe impl Send for DorySetupCache {}
unsafe impl Sync for DorySetupCache {}

static DORY_SETUP_CACHE: OnceLock<DorySetupCache> = OnceLock::new();

// We store the public parameters separately in a `Box` so we can hand out
// a `&'static PublicParameters` to `ProverSetup::from`.  The `Box` is leaked
// intentionally – it lives for the entire process lifetime.
static PUBLIC_PARAMS_STATIC: OnceLock<Box<PublicParameters>> = OnceLock::new();

fn get_or_init_cache() -> &'static DorySetupCache {
    DORY_SETUP_CACHE.get_or_init(|| {
        use ark_std::test_rng;
        let mut rng = test_rng();
        // Leak a Box so we can get a `&'static PublicParameters`.
        let pp: &'static PublicParameters = PUBLIC_PARAMS_STATIC.get_or_init(|| {
            Box::new(PublicParameters::test_rand(MAX_NU, &mut rng))
        });
        let prover_setup = ProverSetup::from(pp);
        let verifier_setup = VerifierSetup::from(pp);
        DorySetupCache {
            public_params: PublicParameters::test_rand(MAX_NU, &mut {
                // We need an owned copy for `DorySetupCache::public_params`.
                // Re-use the already-initialised static to avoid a second
                // expensive `test_rand` call.
                use ark_std::test_rng;
                test_rng()
            }),
            prover_setup,
            verifier_setup,
        }
    })
}

/// Returns a reference to the cached [`PublicParameters`] with [`MAX_NU`] sigma.
///
/// The first call to this function (within a process) will construct the
/// parameters; subsequent calls return the already-constructed value instantly.
pub fn public_parameters() -> &'static PublicParameters {
    PUBLIC_PARAMS_STATIC.get_or_init(|| {
        use ark_std::test_rng;
        let mut rng = test_rng();
        Box::new(PublicParameters::test_rand(MAX_NU, &mut rng))
    })
}

/// Returns a reference to the cached [`ProverSetup`] built from [`MAX_NU`]
/// public parameters.
pub fn prover_setup() -> &'static ProverSetup<'static> {
    &get_or_init_cache().prover_setup
}

/// Returns a reference to the cached [`VerifierSetup`] built from [`MAX_NU`]
/// public parameters.
pub fn verifier_setup() -> &'static VerifierSetup {
    &get_or_init_cache().verifier_setup
}
