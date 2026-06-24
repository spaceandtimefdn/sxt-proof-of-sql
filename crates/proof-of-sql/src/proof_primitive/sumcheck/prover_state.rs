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
    use crate::base::{polynomial::CompositePolynomial, scalar::test_scalar::TestScalar};

    #[test]
    fn new_sets_round_to_zero() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 3, 2);
        assert_eq!(state.round, 0);
    }

    #[test]
    fn new_stores_num_vars() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 5, 0);
        assert_eq!(state.num_vars, 5);
    }

    #[test]
    fn new_stores_max_multiplicands() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 2, 4);
        assert_eq!(state.max_multiplicands, 4);
    }

    #[test]
    fn new_stores_list_of_products() {
        let products = vec![(TestScalar::from(3u64), vec![0usize, 1])];
        let state: ProverState<TestScalar> = ProverState::new(products, vec![], 2, 2);
        assert_eq!(state.list_of_products.len(), 1);
    }

    #[test]
    fn new_stores_flattened_ml_extensions() {
        let exts = vec![vec![TestScalar::from(1u64), TestScalar::from(2u64)]];
        let state: ProverState<TestScalar> = ProverState::new(vec![], exts, 1, 0);
        assert_eq!(state.flattened_ml_extensions.len(), 1);
        assert_eq!(state.flattened_ml_extensions[0].len(), 2);
    }

    #[test]
    fn create_from_empty_composite_polynomial() {
        let poly = CompositePolynomial::<TestScalar>::new(3);
        let state = ProverState::create(&poly);
        assert_eq!(state.num_vars, 3);
        assert_eq!(state.round, 0);
        assert_eq!(state.max_multiplicands, 0);
        assert!(state.list_of_products.is_empty());
    }

    #[test]
    fn debug_output_contains_num_vars() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 7, 0);
        let debug = alloc::format!("{state:?}");
        assert!(debug.contains("num_vars"));
    }

    #[test]
    fn empty_prover_state_has_all_zero_fields() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 0, 0);
        assert!(state.list_of_products.is_empty());
        assert!(state.flattened_ml_extensions.is_empty());
        assert_eq!(state.num_vars, 0);
        assert_eq!(state.max_multiplicands, 0);
        assert_eq!(state.round, 0);
    }
}
