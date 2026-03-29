/// Utility helpers shared across Dory unit-tests.
///
/// The primary purpose of this module is to provide **cached** setup objects so
/// that `PublicParameters::test_rand`, `ProverSetup::from`, and
/// `VerifierSetup::from` are each executed **at most once per test-binary run**
/// rather than once per test function.  Before this change those three calls
/// accounted for the majority of the total test-suite wall time (~10-15 s each
/// time they ran).
///
/// See [`cached_setup`] for details and usage examples.
mod cached_setup;

pub use cached_setup::{
    prover_setup_for_testing, public_parameters_for_testing, verifier_setup_for_testing,
    TEST_SETUP_MAX_NU,
};
