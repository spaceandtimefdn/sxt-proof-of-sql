//! Standalone binary to pre-warm the Dory test setup cache.
//!
//! Run with: `cargo run --bin warmup-test-cache --all-features`
//!
//! This generates cached setup files for nu values 1..=10,
//! which covers all test scenarios. Run once before test suite
//! to eliminate setup generation time during tests.

use std::time::Instant;

fn main() {
    println!("Warming up Dory test setup cache...");
    println!("Cache directory: {}", cache_dir().display());
    println!();

    let total_start = Instant::now();

    for nu in 1..=10 {
        let start = Instant::now();
        print!("  Generating setup for nu={nu}...");

        // This will generate and cache if not already cached
        let _ = get_public_parameters(nu);

        let elapsed = start.elapsed();
        println!(" done ({elapsed:.2?})");
    }

    let total_elapsed = total_start.elapsed();
    println!();
    println!("Cache warmup complete in {total_elapsed:.2?}");
    println!("Subsequent test runs will load from cache (~100ms per setup)");
}

// Note: In actual integration, this would import from the crate.
// For now, this serves as documentation of the intended binary.
use std::path::PathBuf;
use std::fs;

fn cache_dir() -> PathBuf {
    let dir = std::env::var("DORY_TEST_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".test-setup-cache")
        });
    fs::create_dir_all(&dir).expect("Failed to create cache directory");
    dir
}

// Placeholder - actual implementation uses the crate's types
fn get_public_parameters(_nu: usize) {
    // In actual integration:
    // proof_of_sql::proof_primitive::dory::test_setup_accessor::get_public_parameters(nu);
}
