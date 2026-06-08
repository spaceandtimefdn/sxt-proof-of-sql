// =============================================================================
// PATCH: Add to crates/proof-of-sql/src/proof_primitive/dory/mod.rs
// =============================================================================
//
// Add this line to the module declarations in mod.rs:
//
//   #[cfg(test)]
//   pub mod test_setup_accessor;
//
// =============================================================================
// EXAMPLE: How to update existing tests to use the cache
// =============================================================================
//
// BEFORE (in each test):
// ```
// let mut rng = StdRng::seed_from_u64(12345);
// let public_parameters = PublicParameters::test_rand(4, &mut rng);
// let prover_setup = ProverSetup::from(&public_parameters);
// let verifier_setup = VerifierSetup::from(&public_parameters);
// ```
//
// AFTER (using cached setup):
// ```
// use crate::proof_primitive::dory::test_setup_accessor::get_test_setup;
// let setup = get_test_setup(4);
// let prover_setup = setup.prover_setup;
// let verifier_setup = setup.verifier_setup;
// ```
//
// This change alone reduces each test's setup time from ~5-15s to ~100ms (cache hit)
// or ~0ms (in-memory hit for cargo test).
