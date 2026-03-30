/// Cached Dory test setups to avoid re-computing expensive cryptographic parameters
/// across multiple tests in the same test binary process.
///
/// `PublicParameters::test_rand`, `ProverSetup::from`, and `VerifierSetup::from`
/// are expensive operations. By caching them in a [`std::sync::OnceLock`] we pay
/// the cost at most once per test-binary invocation rather than once per test
/// function.
///
/// # Usage
///
/// ```ignore
/// use crate::proof_primitive::dory::test_utils::{
///     dory_setup_cache::{prover_setup, verifier_setup, MAX_NU},
/// };
///
/// #[test]
/// fn my_dory_test() {
///     let ps = prover_setup();
///     let vs = verifier_setup();
///     // ... run your proof
/// }
/// ```
use crate::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

/// The maximum `sigma` value used in the shared test setup cache.
///
/// Tests that need *fewer* generators can simply use a prefix of this setup
/// (Dory setups are nested). Tests that need *more* generators must create
/// their own setup.
pub const MAX_NU: usize = 4;

// ---------------------------------------------------------------------------
// Internal statics
// ---------------------------------------------------------------------------

// We keep `PublicParameters` in its own `OnceLock<Box<…>>` so it can be
// leaked and given a `'static` lifetime that satisfies `ProverSetup::from`.
static PUBLIC_PARAMS_STATIC: OnceLock<Box<PublicParameters>> = OnceLock::new();

struct SetupPair {
    prover_setup: ProverSetup<'static>,
    verifier_setup: VerifierSetup,
}

// SAFETY: `ProverSetup` contains raw pointers into the `PublicParameters` data
// which lives for `'static` (leaked `Box`). The data is never mutated after
// construction, so it is safe to share across threads.
unsafe impl Send for SetupPair {}
unsafe impl Sync for SetupPair {}

static SETUP_PAIR: OnceLock<SetupPair> = OnceLock::new();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns a reference to the cached [`PublicParameters`] constructed with
/// [`MAX_NU`] sigma, using a deterministic test RNG.
///
/// The first call constructs the parameters; all subsequent calls are instant.
pub fn public_parameters() -> &'static PublicParameters {
    PUBLIC_PARAMS_STATIC.get_or_init(|| {
        use ark_std::test_rng;
        let mut rng = test_rng();
        Box::new(PublicParameters::test_rand(MAX_NU, &mut rng))
    })
}

fn get_or_init_pair() -> &'static SetupPair {
    SETUP_PAIR.get_or_init(|| {
        let pp: &'static PublicParameters = public_parameters();
        let prover_setup = ProverSetup::from(pp);
        let verifier_setup = VerifierSetup::from(pp);
        SetupPair {
            prover_setup,
            verifier_setup,
        }
    })
}

/// Returns a reference to the cached [`ProverSetup`] built from the shared
/// [`MAX_NU`] public parameters.
///
/// The first call builds the setup; all subsequent calls are instant.
pub fn prover_setup() -> &'static ProverSetup<'static> {
    &get_or_init_pair().prover_setup
}

/// Returns a reference to the cached [`VerifierSetup`] built from the shared
/// [`MAX_NU`] public parameters.
///
/// The first call builds the setup; all subsequent calls are instant.
pub fn verifier_setup() -> &'static VerifierSetup {
    &get_or_init_pair().verifier_setup
}
