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
    use alloc::{rc::Rc, vec};

    #[test]
    fn new_preserves_state_fields_and_starts_at_round_zero() {
        let list_of_products = vec![
            (TestScalar::from(7u64), vec![0]),
            (-TestScalar::from(3u64), vec![0, 1]),
        ];
        let flattened_ml_extensions = vec![
            vec![TestScalar::from(1u64), TestScalar::from(2u64)],
            vec![TestScalar::from(3u64), TestScalar::from(4u64)],
        ];

        let state = ProverState::new(
            list_of_products.clone(),
            flattened_ml_extensions.clone(),
            3,
            2,
        );

        assert_eq!(state.list_of_products, list_of_products);
        assert_eq!(state.flattened_ml_extensions, flattened_ml_extensions);
        assert_eq!(state.num_vars, 3);
        assert_eq!(state.max_multiplicands, 2);
        assert_eq!(state.round, 0);
    }

    #[test]
    fn create_copies_composite_polynomial_structure() {
        let shared_extension = Rc::new(vec![
            TestScalar::from(1u64),
            TestScalar::from(2u64),
            TestScalar::from(3u64),
            TestScalar::from(4u64),
        ]);
        let second_extension = Rc::new(vec![
            TestScalar::from(5u64),
            TestScalar::from(6u64),
            TestScalar::from(7u64),
            TestScalar::from(8u64),
        ]);
        let mut polynomial = CompositePolynomial::new(2);
        polynomial.add_product([shared_extension.clone()], TestScalar::from(11u64));
        polynomial.add_product(
            [shared_extension.clone(), second_extension.clone()],
            TestScalar::from(13u64),
        );

        let state = ProverState::create(&polynomial);

        assert_eq!(state.list_of_products, polynomial.products);
        assert_eq!(
            state.flattened_ml_extensions,
            vec![(*shared_extension).clone(), (*second_extension).clone()]
        );
        assert_eq!(state.num_vars, polynomial.num_variables);
        assert_eq!(state.max_multiplicands, 2);
        assert_eq!(state.round, 0);
    }

    #[test]
    #[should_panic(expected = "Attempt to prove a constant.")]
    fn create_rejects_constant_polynomial() {
        let polynomial = CompositePolynomial::<TestScalar>::new(0);

        let _ = ProverState::create(&polynomial);
    }
}
