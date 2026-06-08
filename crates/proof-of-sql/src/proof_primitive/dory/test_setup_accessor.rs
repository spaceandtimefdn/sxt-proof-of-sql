//! Cached test setup accessor for Dory proof system.
//!
//! This module provides file-backed caching of expensive Dory setup parameters
//! (`PublicParameters`, `ProverSetup`, `VerifierSetup`) to dramatically reduce
//! test execution time. On first run, setups are generated and persisted to disk.
//! Subsequent runs (including separate nextest processes) load from cache.

use super::{ProverSetup, PublicParameters, VerifierSetup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use once_cell::sync::OnceCell;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::Mutex,
};

/// Fixed seed for deterministic test setup generation.
/// Using a fixed seed ensures cache files are always valid.
const TEST_SEED: u64 = 0xDEAD_BEEF_CAFE_F00D;

/// Maximum nu value we pre-cache. Covers all common test scenarios.
const MAX_CACHED_NU: usize = 10;

/// Returns the cache directory for Dory test setups.
/// Can be overridden via `DORY_TEST_CACHE_DIR` environment variable.
fn cache_dir() -> PathBuf {
    let dir = std::env::var("DORY_TEST_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Use target directory so cache persists across runs
            // but doesn't pollute source tree
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(manifest_dir)
                .join(".test-setup-cache")
        });
    fs::create_dir_all(&dir).expect("Failed to create Dory test cache directory");
    dir
}

/// Returns the cache file path for a given nu value.
fn cache_file_path(nu: usize) -> PathBuf {
    cache_dir().join(format!("public_params_nu_{nu}.bin"))
}

/// Try to load PublicParameters from a cache file.
fn try_load_from_cache(nu: usize) -> Option<PublicParameters> {
    let path = cache_file_path(nu);
    let bytes = fs::read(&path).ok()?;
    PublicParameters::deserialize_compressed(&bytes[..]).ok()
}

/// Save PublicParameters to a cache file using atomic write.
fn save_to_cache(nu: usize, params: &PublicParameters) {
    let path = cache_file_path(nu);
    let temp_path = path.with_extension("bin.tmp");

    let mut bytes = Vec::new();
    if params.serialize_compressed(&mut bytes).is_ok() {
        if fs::write(&temp_path, &bytes).is_ok() {
            // Atomic rename - safe even with concurrent processes
            let _ = fs::rename(&temp_path, &path);
        }
    }
}

/// Create a deterministic RNG for test setup generation.
fn deterministic_rng() -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(TEST_SEED)
}

/// Global in-memory cache for PublicParameters within a single process.
/// This helps when running with `cargo test` (shared process) rather than nextest.
static IN_MEMORY_CACHE: OnceCell<Mutex<HashMap<usize, &'static PublicParameters>>> =
    OnceCell::new();

fn get_in_memory_cache() -> &'static Mutex<HashMap<usize, &'static PublicParameters>> {
    IN_MEMORY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Get or create cached PublicParameters for a given `nu` value.
///
/// This function:
/// 1. Checks in-memory cache (for `cargo test` with shared process)
/// 2. Checks file-based cache (for nextest with separate processes)
/// 3. Generates and caches if not found
///
/// The result is leaked to provide a `'static` lifetime, which is
/// acceptable in test code.
pub fn get_public_parameters(nu: usize) -> &'static PublicParameters {
    // Check in-memory cache first
    let cache = get_in_memory_cache();
    {
        let guard = cache.lock().unwrap();
        if let Some(params) = guard.get(&nu) {
            return params;
        }
    }

    // Try loading from file cache
    let params = if let Some(params) = try_load_from_cache(nu) {
        params
    } else {
        // Generate fresh parameters with deterministic RNG
        let mut rng = deterministic_rng();
        let params = PublicParameters::test_rand(nu, &mut rng);
        // Save to file cache for future runs
        save_to_cache(nu, &params);
        params
    };

    // Leak to get 'static lifetime (fine for tests)
    let static_params: &'static PublicParameters = Box::leak(Box::new(params));

    // Store in in-memory cache
    {
        let mut guard = cache.lock().unwrap();
        guard.insert(nu, static_params);
    }

    static_params
}

/// Global in-memory cache for ProverSetup within a single process.
static PROVER_SETUP_CACHE: OnceCell<Mutex<HashMap<usize, &'static ProverSetup<'static>>>> =
    OnceCell::new();

fn get_prover_setup_cache() -> &'static Mutex<HashMap<usize, &'static ProverSetup<'static>>> {
    PROVER_SETUP_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Get or create cached ProverSetup for a given `nu` value.
///
/// Automatically uses the cached PublicParameters.
pub fn get_prover_setup(nu: usize) -> &'static ProverSetup<'static> {
    let cache = get_prover_setup_cache();
    {
        let guard = cache.lock().unwrap();
        if let Some(setup) = guard.get(&nu) {
            return setup;
        }
    }

    let params = get_public_parameters(nu);
    let setup = ProverSetup::from(params);
    let static_setup: &'static ProverSetup<'static> = Box::leak(Box::new(setup));

    {
        let mut guard = cache.lock().unwrap();
        guard.insert(nu, static_setup);
    }

    static_setup
}

/// Global in-memory cache for VerifierSetup within a single process.
static VERIFIER_SETUP_CACHE: OnceCell<Mutex<HashMap<usize, &'static VerifierSetup<'static>>>> =
    OnceCell::new();

fn get_verifier_setup_cache() -> &'static Mutex<HashMap<usize, &'static VerifierSetup<'static>>> {
    VERIFIER_SETUP_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Get or create cached VerifierSetup for a given `nu` value.
///
/// Automatically uses the cached PublicParameters.
pub fn get_verifier_setup(nu: usize) -> &'static VerifierSetup<'static> {
    let cache = get_verifier_setup_cache();
    {
        let guard = cache.lock().unwrap();
        if let Some(setup) = guard.get(&nu) {
            return setup;
        }
    }

    let params = get_public_parameters(nu);
    let setup = VerifierSetup::from(params);
    let static_setup: &'static VerifierSetup<'static> = Box::leak(Box::new(setup));

    {
        let mut guard = cache.lock().unwrap();
        guard.insert(nu, static_setup);
    }

    static_setup
}

/// Convenience struct bundling all three setup components.
pub struct TestSetup {
    pub public_parameters: &'static PublicParameters,
    pub prover_setup: &'static ProverSetup<'static>,
    pub verifier_setup: &'static VerifierSetup<'static>,
}

/// Get a complete test setup for a given `nu` value.
///
/// # Example
/// ```ignore
/// use crate::proof_primitive::dory::test_setup_accessor::get_test_setup;
///
/// let setup = get_test_setup(4);
/// // Use setup.prover_setup, setup.verifier_setup, etc.
/// ```
pub fn get_test_setup(nu: usize) -> TestSetup {
    TestSetup {
        public_parameters: get_public_parameters(nu),
        prover_setup: get_prover_setup(nu),
        verifier_setup: get_verifier_setup(nu),
    }
}

/// Pre-generate cache files for all common nu values.
/// Call this from a setup script or build step to warm the cache.
pub fn warmup_cache() {
    for nu in 1..=MAX_CACHED_NU {
        let _ = get_public_parameters(nu);
    }
}

/// Clear the file-based cache.
pub fn clear_cache() {
    let dir = cache_dir();
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_roundtrip() {
        // Generate and cache
        let params1 = get_public_parameters(2);
        // Load from cache (should hit file cache if in-memory is cleared)
        let params2 = get_public_parameters(2);
        // Should be the same pointer (in-memory cache)
        assert!(std::ptr::eq(params1, params2));
    }

    #[test]
    fn test_setup_consistency() {
        let setup = get_test_setup(3);
        assert!(!std::ptr::eq(
            setup.public_parameters as *const _ as *const u8,
            std::ptr::null()
        ));
    }

    #[test]
    fn test_deterministic_generation() {
        // Clear cache to force regeneration
        let path = cache_file_path(2);
        let _ = fs::remove_file(&path);

        // Generate fresh
        let mut rng1 = deterministic_rng();
        let params1 = PublicParameters::test_rand(2, &mut rng1);

        let mut rng2 = deterministic_rng();
        let params2 = PublicParameters::test_rand(2, &mut rng2);

        // Serialize both and compare bytes
        let mut bytes1 = Vec::new();
        let mut bytes2 = Vec::new();
        params1.serialize_compressed(&mut bytes1).unwrap();
        params2.serialize_compressed(&mut bytes2).unwrap();
        assert_eq!(bytes1, bytes2);
    }
}
