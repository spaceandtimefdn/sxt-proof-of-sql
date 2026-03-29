/// Utilities for Dory tests that cache expensive setup objects so they are only
/// computed once per test-binary run, dramatically reducing overall test time.
mod cached_setup;
pub use cached_setup::{
    blitzar_handle_for_testing, prover_setup_for_testing, public_parameters_for_testing,
    verifier_setup_for_testing,
};
