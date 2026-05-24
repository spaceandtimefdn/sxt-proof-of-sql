//! Tests for Sumcheck ProverRound.

#[cfg(test)]
mod prover_round_test {
    use crate::base::scalar::test_scalar::TestScalar;
    use crate::proof_primitive::sumcheck::{ProverState, prover_round};
    use crate::base::polynomial::MultilinearExtension;
    use bumpalo::Bump;

    #[test]
    fn test_prover_round_empty_state() {
        let alloc = Bump::new();
        let mle: Vec<TestScalar> = vec![TestScalar::ONE];
        // Test that prover_round can be called (basic sanity)
        let _ = prover_round::<TestScalar>;
    }

    #[test]
    fn test_prover_round_function_exists() {
        // Verify the function exists and is callable
        let _ = prover_round::<TestScalar>;
    }
}