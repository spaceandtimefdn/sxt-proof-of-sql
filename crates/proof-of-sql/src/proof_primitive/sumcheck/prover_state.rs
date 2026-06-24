use crate::base::polynomial::CompositePolynomial;
/*
 * Adapted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::{base::scalar::Scalar, utils::log};
use alloc::vec::Vec;

#[derive(Debug)]
pub struct ProverState<S: Scalar> {
    /// Stores the list of products that is meant to be added together. Each multiplicand is represented by
    /// the index in `flattened_ml_extensions`
    pub list_of_products: Vec<(S, Vec<usize>)>,
    /// Stores a list of multilinear extensions in which `self.list_of_products` points to
    pub flattened_ml_extensions: Vec<Vec<S>>,
    pub num_vars: usize,
    pub max_multiplicands: usize,
    pub round: usize,
}

impl<S: Scalar> ProverState<S> {
    pub fn new(
        list_of_products: Vec<(S, Vec<usize>)>,
        flattened_ml_extensions: Vec<Vec<S>>,
        num_vars: usize,
        max_multiplicands: usize,
    ) -> Self {
        ProverState {
            list_of_products,
            flattened_ml_extensions,
            num_vars,
            max_multiplicands,
            round: 0,
        }
    }

    #[tracing::instrument(name = "ProverState::create", level = "debug", skip_all)]
    #[cfg_attr(not(test), expect(dead_code))]
    pub fn create(polynomial: &CompositePolynomial<S>) -> Self {
        log::log_memory_usage("Start");

        assert!(
            polynomial.num_variables != 0,
            "Attempt to prove a constant."
        );

        // create a deep copy of all unique MLExtensions
        let flattened_ml_extensions = polynomial
            .flattened_ml_extensions
            .iter()
            .map(|x| x.as_ref().clone())
            .collect();

        log::log_memory_usage("End");

        ProverState::new(
            polynomial.products.clone(),
            flattened_ml_extensions,
            polynomial.num_variables,
            polynomial.max_multiplicands,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ProverState;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::vec;

    #[test]
    fn new_sets_list_of_products() {
        let products = vec![(TestScalar::from(1u64), vec![0usize, 1])];
        let exts = vec![vec![TestScalar::from(1u64)], vec![TestScalar::from(2u64)]];
        let state = ProverState::new(products.clone(), exts, 2, 2);
        assert_eq!(state.list_of_products.len(), 1);
    }

    #[test]
    fn new_sets_flattened_ml_extensions() {
        let exts = vec![
            vec![TestScalar::from(1u64), TestScalar::from(2u64)],
            vec![TestScalar::from(3u64), TestScalar::from(4u64)],
        ];
        let state = ProverState::<TestScalar>::new(vec![], exts.clone(), 1, 0);
        assert_eq!(state.flattened_ml_extensions.len(), 2);
        assert_eq!(state.flattened_ml_extensions[0][0], TestScalar::from(1u64));
    }

    #[test]
    fn new_sets_num_vars() {
        let state = ProverState::<TestScalar>::new(vec![], vec![], 5, 0);
        assert_eq!(state.num_vars, 5);
    }

    #[test]
    fn new_sets_max_multiplicands() {
        let state = ProverState::<TestScalar>::new(vec![], vec![], 1, 3);
        assert_eq!(state.max_multiplicands, 3);
    }

    #[test]
    fn new_initializes_round_to_zero() {
        let state = ProverState::<TestScalar>::new(vec![], vec![], 2, 1);
        assert_eq!(state.round, 0);
    }

    #[test]
    fn new_empty_products_and_extensions() {
        let state = ProverState::<TestScalar>::new(vec![], vec![], 0, 0);
        assert!(state.list_of_products.is_empty());
        assert!(state.flattened_ml_extensions.is_empty());
    }

    #[test]
    fn new_preserves_coefficient_in_products() {
        let coeff = TestScalar::from(42u64);
        let products = vec![(coeff, vec![0usize])];
        let exts = vec![vec![TestScalar::from(1u64)]];
        let state = ProverState::new(products, exts, 1, 1);
        assert_eq!(state.list_of_products[0].0, coeff);
    }

    #[test]
    fn new_preserves_indices_in_products() {
        let products = vec![(TestScalar::from(1u64), vec![0usize, 2, 4])];
        let state = ProverState::new(products, vec![], 3, 3);
        assert_eq!(state.list_of_products[0].1, vec![0, 2, 4]);
    }

    #[test]
    fn new_multiple_products() {
        let products = vec![
            (TestScalar::from(1u64), vec![0usize]),
            (TestScalar::from(2u64), vec![1usize]),
        ];
        let state = ProverState::new(products, vec![], 1, 1);
        assert_eq!(state.list_of_products.len(), 2);
    }
}
