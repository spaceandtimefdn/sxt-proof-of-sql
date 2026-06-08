//! Example showing how to migrate existing tests to use cached setups.
//!
//! This file demonstrates the before/after pattern for each type of test
//! that creates Dory setups.

// =============================================================================
// Pattern 1: Simple test with ProverSetup + VerifierSetup
// =============================================================================

// BEFORE:
// ```rust
// #[test]
// fn test_something() {
//     let mut rng = StdRng::seed_from_u64(0);
//     let public_parameters = PublicParameters::test_rand(4, &mut rng);
//     let prover_setup = ProverSetup::from(&public_parameters);
//     let verifier_setup = VerifierSetup::from(&public_parameters);
//     // ... test logic using prover_setup, verifier_setup
// }
// ```

// AFTER:
// ```rust
// #[test]
// fn test_something() {
//     let setup = test_setup_accessor::get_test_setup(4);
//     let prover_setup = setup.prover_setup;
//     let verifier_setup = setup.verifier_setup;
//     // ... test logic using prover_setup, verifier_setup (unchanged)
// }
// ```

// =============================================================================
// Pattern 2: Test that also needs PublicParameters directly
// =============================================================================

// BEFORE:
// ```rust
// #[test]
// fn test_setup_sizes() {
//     let mut rng = StdRng::seed_from_u64(0);
//     let public_parameters = PublicParameters::test_rand(6, &mut rng);
//     // ... test logic using public_parameters
// }
// ```

// AFTER:
// ```rust
// #[test]
// fn test_setup_sizes() {
//     let public_parameters = test_setup_accessor::get_public_parameters(6);
//     // ... test logic using public_parameters (unchanged)
// }
// ```

// =============================================================================
// Pattern 3: Test with loop over multiple nu values
// =============================================================================

// BEFORE:
// ```rust
// #[test]
// fn test_various_lengths() {
//     for nu in 1..=8 {
//         let mut rng = StdRng::seed_from_u64(0);
//         let public_parameters = PublicParameters::test_rand(nu, &mut rng);
//         let prover_setup = ProverSetup::from(&public_parameters);
//         let verifier_setup = VerifierSetup::from(&public_parameters);
//         // ... test at this nu
//     }
// }
// ```

// AFTER:
// ```rust
// #[test]
// fn test_various_lengths() {
//     for nu in 1..=8 {
//         let setup = test_setup_accessor::get_test_setup(nu);
//         // ... test at this nu (unchanged)
//     }
// }
// ```

// =============================================================================
// Pattern 4: Using the macro for even more concise code
// =============================================================================

// ```rust
// use crate::proof_primitive::dory::test_macro::cached_dory_setup;
//
// #[test]
// fn test_something() {
//     cached_dory_setup!(nu = 4, prover_setup, verifier_setup);
//     // prover_setup and verifier_setup are directly available
// }
// ```

fn main() {
    println!("This is a documentation example - see source for migration patterns.");
}
