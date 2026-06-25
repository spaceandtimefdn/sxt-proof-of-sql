//! Cached Dory test setups using [`std::sync::OnceLock`].
//!
//! `PublicParameters::test_rand` and `VerifierSetup::from` are expensive operations
//! involving BLS12-381 pairings. Caching them with `OnceLock` means each unique
//! parameter set is created only once per test-binary execution, regardless of
//! how many test functions request it.
//!
//! # Safety
//! `test_rng()` is deterministic (fixed seed), so the cached values are identical
//! to what each test would create independently.
use super::{test_rng, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

static PP_NU_4: OnceLock<PublicParameters> = OnceLock::new();
static PP_NU_6: OnceLock<PublicParameters> = OnceLock::new();
static VERIFIER_SETUP_NU_4: OnceLock<VerifierSetup> = OnceLock::new();
static VERIFIER_SETUP_NU_6: OnceLock<VerifierSetup> = OnceLock::new();

/// Returns a reference to cached `PublicParameters` with `max_nu = 4`.
///
/// The first call to this function initialises the parameters; subsequent calls
/// return the cached value instantly.
pub fn test_params_nu_4() -> &'static PublicParameters {
    PP_NU_4.get_or_init(|| PublicParameters::test_rand(4, &mut test_rng()))
}

/// Returns a reference to cached `PublicParameters` with `max_nu = 6`.
pub fn test_params_nu_6() -> &'static PublicParameters {
    PP_NU_6.get_or_init(|| PublicParameters::test_rand(6, &mut test_rng()))
}

/// Returns a reference to a cached `VerifierSetup` derived from `max_nu = 4` parameters.
pub fn test_verifier_setup_nu_4() -> &'static VerifierSetup {
    VERIFIER_SETUP_NU_4.get_or_init(|| VerifierSetup::from(test_params_nu_4()))
}

/// Returns a reference to a cached `VerifierSetup` derived from `max_nu = 6` parameters.
pub fn test_verifier_setup_nu_6() -> &'static VerifierSetup {
    VERIFIER_SETUP_NU_6.get_or_init(|| VerifierSetup::from(test_params_nu_6()))
}
