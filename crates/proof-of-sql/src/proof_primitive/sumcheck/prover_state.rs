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
    use crate::base::{polynomial::CompositePolynomial, scalar::test_scalar::TestScalar};
    use alloc::{rc::Rc, vec};

    #[test]
    fn prover_state_new_initializes_round_state() {
        let state = ProverState::new(
            vec![(TestScalar::from(7), vec![0, 1])],
            vec![vec![TestScalar::from(1), TestScalar::from(2)]],
            3,
            2,
        );

        assert_eq!(
            state.list_of_products,
            vec![(TestScalar::from(7), vec![0, 1])]
        );
        assert_eq!(
            state.flattened_ml_extensions,
            vec![vec![TestScalar::from(1), TestScalar::from(2)]]
        );
        assert_eq!(state.num_vars, 3);
        assert_eq!(state.max_multiplicands, 2);
        assert_eq!(state.round, 0);
    }

    #[test]
    fn prover_state_create_copies_composite_polynomial_terms() {
        let mle_a = Rc::new(vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(4),
        ]);
        let mle_b = Rc::new(vec![
            TestScalar::from(5),
            TestScalar::from(6),
            TestScalar::from(7),
            TestScalar::from(8),
        ]);

        let mut polynomial = CompositePolynomial::new(2);
        polynomial.add_product([mle_a.clone(), mle_b.clone()], TestScalar::from(9));
        polynomial.add_product([mle_a.clone()], TestScalar::from(10));

        let state = ProverState::create(&polynomial);

        assert_eq!(
            state.list_of_products,
            vec![
                (TestScalar::from(9), vec![0, 1]),
                (TestScalar::from(10), vec![0])
            ]
        );
        assert_eq!(
            state.flattened_ml_extensions,
            vec![(*mle_a).clone(), (*mle_b).clone()]
        );
        assert_eq!(state.num_vars, 2);
        assert_eq!(state.max_multiplicands, 2);
        assert_eq!(state.round, 0);
    }

    #[test]
    #[should_panic(expected = "Attempt to prove a constant.")]
    fn prover_state_create_rejects_constant_polynomials() {
        let polynomial = CompositePolynomial::<TestScalar>::new(0);

        ProverState::create(&polynomial);
    }
}
