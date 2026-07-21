//! Cached test setups for Dory commitment scheme to improve test performance.
//!
//! This module provides cached `PublicParameters`, `ProverSetup`, and `VerifierSetup`
//! instances for commonly used `nu` values in tests. This significantly reduces test
//! execution time by avoiding repeated expensive setup operations.

use super::{test_rng, ProverSetup, PublicParameters, VerifierSetup};
use std::sync::LazyLock;

/// Cached public parameters for nu=4.
/// This is the most commonly used value in tests.
pub static PUBLIC_PARAMETERS_4: LazyLock<PublicParameters> = LazyLock::new(|| {
    PublicParameters::test_rand(4, &mut test_rng())
});

/// Cached prover setup for nu=4.
pub static PROVER_SETUP_4: LazyLock<ProverSetup<'static>> = LazyLock::new(|| {
    ProverSetup::from(&*PUBLIC_PARAMETERS_4)
});

/// Cached verifier setup for nu=4.
pub static VERIFIER_SETUP_4: LazyLock<VerifierSetup> = LazyLock::new(|| {
    VerifierSetup::from(&*PUBLIC_PARAMETERS_4)
});

/// Cached public parameters for nu=6.
/// Used in tests that require larger setups.
pub static PUBLIC_PARAMETERS_6: LazyLock<PublicParameters> = LazyLock::new(|| {
    PublicParameters::test_rand(6, &mut test_rng())
});

/// Cached prover setup for nu=6.
pub static PROVER_SETUP_6: LazyLock<ProverSetup<'static>> = LazyLock::new(|| {
    ProverSetup::from(&*PUBLIC_PARAMETERS_6)
});

/// Cached verifier setup for nu=6.
pub static VERIFIER_SETUP_6: LazyLock<VerifierSetup> = LazyLock::new(|| {
    VerifierSetup::from(&*PUBLIC_PARAMETERS_6)
});

/// Cached public parameters for nu=3.
/// Used in some inner product tests.
pub static PUBLIC_PARAMETERS_3: LazyLock<PublicParameters> = LazyLock::new(|| {
    PublicParameters::test_rand(3, &mut test_rng())
});

/// Cached prover setup for nu=3.
pub static PROVER_SETUP_3: LazyLock<ProverSetup<'static>> = LazyLock::new(|| {
    ProverSetup::from(&*PUBLIC_PARAMETERS_3)
});

/// Cached verifier setup for nu=3.
pub static VERIFIER_SETUP_3: LazyLock<VerifierSetup> = LazyLock::new(|| {
    VerifierSetup::from(&*PUBLIC_PARAMETERS_3)
});

/// Cached public parameters for nu=5.
/// Used in some tests that iterate over multiple nu values.
pub static PUBLIC_PARAMETERS_5: LazyLock<PublicParameters> = LazyLock::new(|| {
    PublicParameters::test_rand(5, &mut test_rng())
});

/// Cached prover setup for nu=5.
pub static PROVER_SETUP_5: LazyLock<ProverSetup<'static>> = LazyLock::new(|| {
    ProverSetup::from(&*PUBLIC_PARAMETERS_5)
});

/// Cached verifier setup for nu=5.
pub static VERIFIER_SETUP_5: LazyLock<VerifierSetup> = LazyLock::new(|| {
    VerifierSetup::from(&*PUBLIC_PARAMETERS_5)
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_setups_are_valid() {
        // Verify that cached setups can be accessed and have correct max_nu
        assert_eq!(PROVER_SETUP_3.max_nu, 3);
        assert_eq!(VERIFIER_SETUP_3.max_nu, 3);

        assert_eq!(PROVER_SETUP_4.max_nu, 4);
        assert_eq!(VERIFIER_SETUP_4.max_nu, 4);

        assert_eq!(PROVER_SETUP_5.max_nu, 5);
        assert_eq!(VERIFIER_SETUP_5.max_nu, 5);

        assert_eq!(PROVER_SETUP_6.max_nu, 6);
        assert_eq!(VERIFIER_SETUP_6.max_nu, 6);
    }
}
