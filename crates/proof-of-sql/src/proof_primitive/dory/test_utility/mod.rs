/// Utility helpers for Dory tests.
///
/// The main export of this module is a set of *cached* setup objects that are
/// computed once per test-binary invocation.  This avoids the repeated
/// (10-15 s each) `PublicParameters::test_rand` / `ProverSetup::from` /
/// `VerifierSetup::from` calls that previously dominated the overall test
/// suite runtime.
mod cached_setup;

pub use cached_setup::{
    prover_setup_for_testing, public_parameters_for_testing, verifier_setup_for_testing,
    TEST_SETUP_MAX_NU,
};
