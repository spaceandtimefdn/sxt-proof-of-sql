//! Cached Dory test setup parameters.
//!
//! The Dory tests repeatedly create identical `PublicParameters`, `VerifierSetup`,
//! and `ProverSetup` instances from the same deterministic RNG seed.  Since
//! `VerifierSetup::from(&PublicParameters)` performs expensive BLS12-381 pairings,
//! caching these across tests within the same binary eliminates a large amount of
//! redundant work.
//!
//! Each `LazyLock` is initialized at most once per test binary invocation.
use super::{ProverSetup, PublicParameters, VerifierSetup};
use std::sync::LazyLock;

/// A cached set of Dory test parameters for a specific `nu` value.
///
/// `PublicParameters` is heap-allocated and leaked to obtain `&'static` references,
/// which allows `ProverSetup<'static>` to live in a `LazyLock` as well.
struct CachedSetup {
    public_parameters: &'static PublicParameters,
    verifier_setup: VerifierSetup,
    prover_setup: ProverSetup<'static>,
}

impl CachedSetup {
    fn new(nu: usize) -> Self {
        let public_parameters =
            Box::leak(Box::new(PublicParameters::test_rand(nu, &mut test_rng())));
        let verifier_setup = VerifierSetup::from(&*public_parameters);
        let prover_setup = ProverSetup::from(&*public_parameters);
        Self {
            public_parameters,
            verifier_setup,
            prover_setup,
        }
    }
}

fn test_rng() -> impl ark_std::rand::Rng {
    ark_std::test_rng()
}

// --- nu = 2 -----------------------------------------------------------------
static SETUP_NU_2: LazyLock<CachedSetup> = LazyLock::new(|| CachedSetup::new(2));

// --- nu = 3 -----------------------------------------------------------------
static SETUP_NU_3: LazyLock<CachedSetup> = LazyLock::new(|| CachedSetup::new(3));

// --- nu = 4 -----------------------------------------------------------------
static SETUP_NU_4: LazyLock<CachedSetup> = LazyLock::new(|| CachedSetup::new(4));

// --- nu = 5 -----------------------------------------------------------------
static SETUP_NU_5: LazyLock<CachedSetup> = LazyLock::new(|| CachedSetup::new(5));

// --- nu = 6 -----------------------------------------------------------------
static SETUP_NU_6: LazyLock<CachedSetup> = LazyLock::new(|| CachedSetup::new(6));

/// Returns the cached `PublicParameters` for the given `nu`.
///
/// # Panics
/// Panics if `nu` is not one of the pre-configured values (0..=6).
pub fn cached_public_parameters(nu: usize) -> &'static PublicParameters {
    match nu {
        2 => SETUP_NU_2.public_parameters,
        3 => SETUP_NU_3.public_parameters,
        4 => SETUP_NU_4.public_parameters,
        5 => SETUP_NU_5.public_parameters,
        6 => SETUP_NU_6.public_parameters,
        _ => {
            panic!("No cached PublicParameters for nu={nu}. Add an entry to test_dory_setup_cache.")
        }
    }
}

/// Returns the cached `VerifierSetup` for the given `nu`.
///
/// # Panics
/// Panics if `nu` is not one of the pre-configured values (0..=6).
pub fn cached_verifier_setup(nu: usize) -> &'static VerifierSetup {
    match nu {
        2 => &SETUP_NU_2.verifier_setup,
        3 => &SETUP_NU_3.verifier_setup,
        4 => &SETUP_NU_4.verifier_setup,
        5 => &SETUP_NU_5.verifier_setup,
        6 => &SETUP_NU_6.verifier_setup,
        _ => panic!("No cached VerifierSetup for nu={nu}. Add an entry to test_dory_setup_cache."),
    }
}

/// Returns the cached `ProverSetup` for the given `nu`.
///
/// # Panics
/// Panics if `nu` is not one of the pre-configured values (0..=6).
pub fn cached_prover_setup(nu: usize) -> &'static ProverSetup<'static> {
    match nu {
        2 => &SETUP_NU_2.prover_setup,
        3 => &SETUP_NU_3.prover_setup,
        4 => &SETUP_NU_4.prover_setup,
        5 => &SETUP_NU_5.prover_setup,
        6 => &SETUP_NU_6.prover_setup,
        _ => panic!("No cached ProverSetup for nu={nu}. Add an entry to test_dory_setup_cache."),
    }
}
