/// Cached Dory test setup helpers.
///
/// These helpers use [`OnceLock`] to lazily initialize and cache expensive
/// Dory setup objects (e.g. [`PublicParameters`], [`ProverSetup`],
/// [`VerifierSetup`]) so that they are only computed once per test process.
///
/// # Note on `max_nu`
///
/// The [`PublicParameters`] are initialized with `max_nu = 4`.
/// `max_nu` controls the maximum depth of the Dory reduction tree and bounds
/// the length of vectors that can be committed: specifically, the maximum
/// supported vector length is `2^(2 * max_nu)` (i.e. up to 256 elements when
/// `max_nu = 4`). This is distinct from the `sigma` parameter used in
/// [`DoryProverPublicSetup`] / [`DoryVerifierPublicSetup`], which is a
/// separate bound that must satisfy `sigma <= max_nu`.
use std::sync::OnceLock;

use super::{ProverSetup, PublicParameters, VerifierSetup};

/// The `max_nu` value used for all cached test setups.
const TEST_MAX_NU: usize = 4;

static PUBLIC_PARAMETERS: OnceLock<PublicParameters> = OnceLock::new();
static PROVER_SETUP_STORAGE: OnceLock<Vec<ark_bn254::G1Affine>> = OnceLock::new();

/// Returns a reference to the cached [`PublicParameters`] for tests.
///
/// On the first call, [`PublicParameters::test_rand`] is invoked with
/// `max_nu = 4`, which supports vector lengths up to `2^(2 * max_nu) = 256`.
pub fn test_public_parameters() -> &'static PublicParameters {
    PUBLIC_PARAMETERS.get_or_init(|| {
        use ark_std::test_rng;
        PublicParameters::test_rand(TEST_MAX_NU, &mut test_rng())
    })
}

/// Returns a [`ProverSetup`] constructed from the cached [`PublicParameters`].
pub fn test_prover_setup() -> ProverSetup<'static> {
    let pp = test_public_parameters();
    let storage = PROVER_SETUP_STORAGE.get_or_init(|| ProverSetup::blitzar_handle_storage(pp));
    ProverSetup::from_public_parameters_and_blitzar_storage(pp, storage)
}

/// Returns a [`VerifierSetup`] constructed from the cached [`PublicParameters`].
pub fn test_verifier_setup() -> VerifierSetup {
    VerifierSetup::from(test_public_parameters())
}
