/// Cached Dory test setup helpers using [`OnceLock`] singletons.
///
/// These functions lazily initialize and cache expensive Dory objects
/// ([`PublicParameters`], [`ProverSetup`], [`VerifierSetup`]) so they are
/// computed at most once per test process, reducing overall test runtime.
///
/// # Terminology: `max_nu` vs `sigma`
///
/// - **`max_nu`** (used by [`PublicParameters::test_rand`]): controls the
///   maximum depth of the Dory inner-product reduction tree.  The maximum
///   supported **committed vector length** is `2^(2 * max_nu)`.  With
///   `max_nu = 4` that is up to **256** elements.
///
/// - **`sigma`** (used by [`DoryProverPublicSetup`] /
///   [`DoryVerifierPublicSetup`]): an independent parameter that must satisfy
///   `sigma <= max_nu`.  It is a separate bound on the commitment scheme; it
///   is *not* the same as `max_nu`.
use std::sync::OnceLock;

use super::{ProverSetup, PublicParameters, VerifierSetup};

/// `max_nu` value shared by all cached test setups.
///
/// Supports vector lengths up to `2^(2 * TEST_MAX_NU)` = 256.
const TEST_MAX_NU: usize = 4;

static PUBLIC_PARAMETERS: OnceLock<PublicParameters> = OnceLock::new();

/// Returns a reference to the cached [`PublicParameters`] for tests.
///
/// On the first call this calls [`PublicParameters::test_rand`] with
/// `max_nu = 4` (supports vectors up to length 256). Subsequent calls return
/// the same instance without recomputing.
pub(super) fn test_public_parameters() -> &'static PublicParameters {
    PUBLIC_PARAMETERS.get_or_init(|| {
        use ark_std::test_rng;
        PublicParameters::test_rand(TEST_MAX_NU, &mut test_rng())
    })
}

/// Returns a [`ProverSetup`] built from the cached [`PublicParameters`].
pub(super) fn test_prover_setup() -> ProverSetup<'static> {
    ProverSetup::from(test_public_parameters())
}

/// Returns a [`VerifierSetup`] built from the cached [`PublicParameters`].
pub(super) fn test_verifier_setup() -> VerifierSetup {
    VerifierSetup::from(test_public_parameters())
}
