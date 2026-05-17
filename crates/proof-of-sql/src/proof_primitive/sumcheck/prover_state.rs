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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::{rc::Rc, vec, vec::Vec};

    fn s(x: i64) -> TestScalar {
        TestScalar::from(x)
    }

    #[test]
    fn new_with_empty_inputs_initializes_round_to_zero() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 3, 2);
        assert_eq!(state.round, 0);
        assert_eq!(state.num_vars, 3);
        assert_eq!(state.max_multiplicands, 2);
        assert!(state.list_of_products.is_empty());
        assert!(state.flattened_ml_extensions.is_empty());
    }

    #[test]
    fn new_preserves_field_values_verbatim() {
        let products: Vec<(TestScalar, Vec<usize>)> =
            vec![(s(2), vec![0, 1]), (s(3), vec![1, 2])];
        let extensions: Vec<Vec<TestScalar>> =
            vec![vec![s(1), s(2)], vec![s(3), s(4)], vec![s(5), s(6)]];
        let state = ProverState::new(products.clone(), extensions.clone(), 4, 2);
        assert_eq!(state.list_of_products, products);
        assert_eq!(state.flattened_ml_extensions, extensions);
        assert_eq!(state.num_vars, 4);
        assert_eq!(state.max_multiplicands, 2);
        assert_eq!(state.round, 0);
    }

    #[test]
    fn new_always_sets_round_to_zero_regardless_of_other_fields() {
        let state: ProverState<TestScalar> = ProverState::new(vec![], vec![], 0, 0);
        assert_eq!(state.round, 0);
        let state2: ProverState<TestScalar> = ProverState::new(vec![], vec![], 100, 50);
        assert_eq!(state2.round, 0);
    }

    #[test]
    fn create_copies_fields_from_composite_polynomial() {
        let mut poly: CompositePolynomial<TestScalar> = CompositePolynomial::new(3);
        let ext1 = Rc::new(vec![s(1), s(2), s(3), s(4), s(5), s(6), s(7), s(8)]);
        let ext2 = Rc::new(vec![s(2), s(4), s(6), s(8), s(10), s(12), s(14), s(16)]);
        poly.add_product([ext1, ext2], s(5));

        let state = ProverState::create(&poly);
        assert_eq!(state.num_vars, poly.num_variables);
        assert_eq!(state.max_multiplicands, poly.max_multiplicands);
        assert_eq!(state.list_of_products, poly.products);
        assert_eq!(state.round, 0);
        assert_eq!(state.flattened_ml_extensions.len(), 2);
    }

    #[test]
    fn create_performs_a_deep_copy_of_ml_extensions() {
        let mut poly: CompositePolynomial<TestScalar> = CompositePolynomial::new(1);
        let original = vec![s(11), s(22)];
        let shared = Rc::new(original.clone());
        poly.add_product([shared], s(1));

        let state = ProverState::create(&poly);
        // The state owns Vec<Vec<S>> — not Rc — so its data is independent of any
        // outstanding Rc references on the original polynomial.
        assert_eq!(state.flattened_ml_extensions, vec![original]);
    }

    #[test]
    #[should_panic(expected = "Attempt to prove a constant.")]
    fn create_panics_when_polynomial_has_zero_variables() {
        let poly: CompositePolynomial<TestScalar> = CompositePolynomial::new(0);
        let _ = ProverState::create(&poly);
    }

    #[test]
    fn create_from_polynomial_with_no_products_yields_empty_state_lists() {
        // A non-zero num_variables polynomial with no products added still satisfies
        // the create() precondition; we should get empty product/extension lists.
        let poly: CompositePolynomial<TestScalar> = CompositePolynomial::new(1);
        let state = ProverState::create(&poly);
        assert_eq!(state.num_vars, 1);
        assert_eq!(state.max_multiplicands, 0);
        assert!(state.list_of_products.is_empty());
        assert!(state.flattened_ml_extensions.is_empty());
        assert_eq!(state.round, 0);
    }
}
