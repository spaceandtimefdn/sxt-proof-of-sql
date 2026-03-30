/// Test utilities for the Dory proof system.
///
/// These utilities are only compiled when `#[cfg(test)]` or the `test-utils`
/// feature is active so they never appear in production binaries.
#[cfg(any(test, feature = "test"))]
pub mod dory_setup_cache;

#[cfg(any(test, feature = "test"))]
pub use dory_setup_cache::{prover_setup, public_parameters, verifier_setup, MAX_NU};
