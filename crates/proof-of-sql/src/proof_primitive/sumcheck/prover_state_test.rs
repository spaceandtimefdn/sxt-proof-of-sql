//! Tests for prover state.

#[cfg(test)]
mod prover_state_test {
    use crate::proof_primitive::sumcheck::prover_state::ProverState;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_prover_state_new() {
        let list_of_products: Vec<(TestScalar, Vec<usize>)> = vec![];
        let flattened_ml_extensions: Vec<Vec<TestScalar>> = vec![];

        let state = ProverState::new(
            list_of_products,
            flattened_ml_extensions,
            2,
            4,
        );

        assert_eq!(state.num_vars, 2);
        assert_eq!(state.max_multiplicands, 4);
        assert_eq!(state.round, 0);
    }

    #[test]
    fn test_prover_state_debug() {
        let list_of_products: Vec<(TestScalar, Vec<usize>)> = vec![];
        let flattened_ml_extensions: Vec<Vec<TestScalar>> = vec![];

        let state = ProverState::new(
            list_of_products,
            flattened_ml_extensions,
            2,
            4,
        );

        let debug_str = format!("{:?}", state);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_prover_state_with_data() {
        let list_of_products = vec![
            (TestScalar::ONE, vec![0, 1]),
            (TestScalar::TWO, vec![2]),
        ];
        let flattened_ml_extensions = vec![
            vec![TestScalar::ONE, TestScalar::TWO],
            vec![TestScalar::ZERO, TestScalar::ONE],
            vec![TestScalar::ONE, TestScalar::ONE, TestScalar::ONE, TestScalar::ONE],
        ];

        let state = ProverState::new(
            list_of_products,
            flattened_ml_extensions,
            2,
            2,
        );

        assert_eq!(state.num_vars, 2);
        assert_eq!(state.max_multiplicands, 2);
        assert_eq!(state.round, 0);
        assert_eq!(state.list_of_products.len(), 2);
        assert_eq!(state.flattened_ml_extensions.len(), 3);
    }
}