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

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn new_stores_list_of_products() {
        let state = ProverState::new(
            alloc::vec![(ts(1), alloc::vec![0usize])],
            alloc::vec![alloc::vec![ts(1), ts(2)]],
            1,
            1,
        );
        assert_eq!(state.list_of_products.len(), 1);
    }

    #[test]
    fn new_stores_flattened_extensions() {
        let state = ProverState::new(
            alloc::vec![],
            alloc::vec![alloc::vec![ts(3), ts(4)]],
            1,
            0,
        );
        assert_eq!(state.flattened_ml_extensions.len(), 1);
        assert_eq!(state.flattened_ml_extensions[0][0], ts(3));
    }

    #[test]
    fn new_stores_num_vars() {
        let state = ProverState::<TestScalar>::new(alloc::vec![], alloc::vec![], 3, 0);
        assert_eq!(state.num_vars, 3);
    }

    #[test]
    fn new_stores_max_multiplicands() {
        let state = ProverState::<TestScalar>::new(alloc::vec![], alloc::vec![], 1, 4);
        assert_eq!(state.max_multiplicands, 4);
    }

    #[test]
    fn new_starts_round_at_zero() {
        let state = ProverState::<TestScalar>::new(alloc::vec![], alloc::vec![], 1, 0);
        assert_eq!(state.round, 0);
    }
}
