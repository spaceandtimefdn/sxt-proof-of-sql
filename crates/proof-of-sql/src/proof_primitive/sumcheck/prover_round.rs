/*
 * Adapted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::{
    base::{if_rayon, scalar::Scalar},
    proof_primitive::sumcheck::ProverState,
    utils::log,
};
use alloc::{vec, vec::Vec};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[expect(clippy::ref_option, clippy::missing_panics_doc)]
#[tracing::instrument(level = "debug", skip_all)]
pub fn prove_round<S: Scalar>(prover_state: &mut ProverState<S>, r_maybe: &Option<S>) -> Vec<S> {
    log::log_memory_usage("Start");

    if let Some(r) = r_maybe {
        assert!(
            prover_state.round != 0,
            "first round should be prover first."
        );

        // fix argument
        if_rayon!(
            prover_state.flattened_ml_extensions.par_iter_mut(),
            prover_state.flattened_ml_extensions.iter_mut()
        )
        .for_each(|multiplicand| {
            in_place_fix_variable(multiplicand, *r, prover_state.num_vars - prover_state.round);
        });
    } else if prover_state.round > 0 {
        panic!("verifier message is empty");
    }

    prover_state.round += 1;

    assert!(
        prover_state.round <= prover_state.num_vars,
        "Prover is not active"
    );

    let degree = prover_state.max_multiplicands; // the degree of univariate polynomial sent by prover at this round
    let round_length = 1usize << (prover_state.num_vars - prover_state.round);

    // The pseudocode of what this is trying to do is:

    // foreach t in 0..=degree compute
    //   sum over row in 0..round_length:
    //     sum over product in list_of_products:
    //       product over multiplicand in product:
    //         table = the mle of the multiplicand
    //         table[2b] * (1-t) + table[2b+1] * t
    // This gives a vector of length degree + 1

    // The order of these loops is changed for the purpose of efficiency.

    // The outer loop is the loop over all products in the list_of_products
    let sums_iter = if_rayon!(
        prover_state.list_of_products.par_iter(),
        prover_state.list_of_products.iter()
    )
    .map(|(coefficient, multiplicand_indices)| {
        // The second loop is the loop over the row (b) in 0..round_length
        let products_iter =
            if_rayon!((0..round_length).into_par_iter(), 0..round_length).map(|b| {
                // We add a vector of products, which takes a bit of extra memory. The reason for this is for the efficient modification described below
                let mut products = vec![*coefficient; degree + 1];

                // The third loop is the loop over the factors/multiplicand in the product term.
                for &multiplicand_index in multiplicand_indices {
                    let table = &prover_state.flattened_ml_extensions[multiplicand_index];

                    // This third+final loop give an efficient way of computing
                    // products[t] *= table[b << 1] * (S::one() - t_as_field) + table[(b << 1) + 1] * t_as_field;
                    // It requires only 1 addition (plus the cumulative multiplication) to accomplish the same task.
                    // It relies on the fact that
                    // table[b << 1] * (S::one() - t_as_field) + table[(b << 1) + 1] * t_as_field == table[b << 1] + t * diff
                    let mut start = table[b << 1];
                    let step = table[(b << 1) + 1] - start;

                    // The innermost loop loops over the values (t) that we are evaluating at.
                    products.iter_mut().take(degree).for_each(|product| {
                        *product *= start;
                        start += step;
                    });
                    products[degree] *= start;
                }
                products
            });
        if_rayon!(
            products_iter.reduce(|| vec![S::zero(); degree + 1], vec_elementwise_add),
            products_iter.fold(vec![S::zero(); degree + 1], vec_elementwise_add)
        )
    });
    let res = if_rayon!(
        sums_iter.reduce(|| vec![S::zero(); degree + 1], vec_elementwise_add),
        sums_iter.fold(vec![S::zero(); degree + 1], vec_elementwise_add)
    );

    log::log_memory_usage("End");

    res
}

/// This is equivalent to
/// *multiplicand = Vec<S> {
///                    `ark_impl`: `multiplicand.ark_impl.fix_variables(&[r_as_field])`,
///                };
/// Only it does it in place
/// # Panics
/// Panics if `num_vars` is less than or equal to 0, indicating an invalid size of the partial point.
fn in_place_fix_variable<S: Scalar>(multiplicand: &mut [S], r_as_field: S, num_vars: usize) {
    assert!(num_vars > 0, "invalid size of partial point");
    for b in 0..(1 << num_vars) {
        let left: S = multiplicand[b << 1];
        let right: S = multiplicand[(b << 1) + 1];
        multiplicand[b] = left + r_as_field * (right - left);
    }
}

fn vec_elementwise_add<S: Scalar>(a: Vec<S>, b: Vec<S>) -> Vec<S> {
    a.into_iter().zip(b).map(|(x, y)| x + y).collect::<Vec<S>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn vec_elementwise_add_basic() {
        let a = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let b = vec![
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];
        let result = vec_elementwise_add(a, b);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], TestScalar::from(11));
        assert_eq!(result[1], TestScalar::from(22));
        assert_eq!(result[2], TestScalar::from(33));
    }

    #[test]
    fn vec_elementwise_add_empty() {
        let a: Vec<TestScalar> = vec![];
        let b: Vec<TestScalar> = vec![];
        let result = vec_elementwise_add(a, b);
        assert!(result.is_empty());
    }

    #[test]
    fn in_place_fix_variable_simple() {
        // num_vars=1: iterates b in 0..2, reads multiplicand[2*b] and multiplicand[2*b+1]
        // So data needs to be at least 2 * (1 << 1) = 4 entries long
        let mut data = vec![
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
            TestScalar::from(40),
        ];
        let r = TestScalar::from(0); // r=0 means we get left
        in_place_fix_variable(&mut data, r, 1);
        // b=0: left=10, right=20, result = 10 + 0*(20-10) = 10
        // b=1: left=30, right=40, result = 30 + 0*(40-30) = 30
        assert_eq!(data[0], TestScalar::from(10));
        assert_eq!(data[1], TestScalar::from(30));
    }

    #[test]
    fn in_place_fix_variable_r_equals_one() {
        // r=1 means we get right
        let mut data = vec![
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
            TestScalar::from(40),
        ];
        let r = TestScalar::from(1);
        in_place_fix_variable(&mut data, r, 1);
        // b=0: 10 + 1*(20-10) = 20
        // b=1: 30 + 1*(40-30) = 40
        assert_eq!(data[0], TestScalar::from(20));
        assert_eq!(data[1], TestScalar::from(40));
    }

    #[test]
    fn in_place_fix_variable_two_vars() {
        // num_vars=2: iterates b in 0..4, reads multiplicand[2*b] and multiplicand[2*b+1]
        // So data needs to be at least 2 * (1 << 2) = 8 entries long
        let mut data = vec![
            TestScalar::from(0),
            TestScalar::from(4),
            TestScalar::from(10),
            TestScalar::from(14),
            TestScalar::from(20),
            TestScalar::from(24),
            TestScalar::from(30),
            TestScalar::from(34),
        ];
        let r = TestScalar::from(2);
        in_place_fix_variable(&mut data, r, 2);
        // b=0: 0 + 2*(4-0) = 8
        // b=1: 10 + 2*(14-10) = 18
        // b=2: 20 + 2*(24-20) = 28
        // b=3: 30 + 2*(34-30) = 38
        assert_eq!(data[0], TestScalar::from(8));
        assert_eq!(data[1], TestScalar::from(18));
        assert_eq!(data[2], TestScalar::from(28));
        assert_eq!(data[3], TestScalar::from(38));
    }

    #[test]
    fn in_place_fix_variable_halves_length() {
        let mut data = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(4),
            TestScalar::from(5),
            TestScalar::from(6),
            TestScalar::from(7),
            TestScalar::from(8),
        ];
        in_place_fix_variable(&mut data, TestScalar::from(0), 2);
        // r=0: just takes left values
        assert_eq!(data[0], TestScalar::from(1));
        assert_eq!(data[1], TestScalar::from(3));
        assert_eq!(data[2], TestScalar::from(5));
        assert_eq!(data[3], TestScalar::from(7));
    }

    #[test]
    #[should_panic(expected = "invalid size of partial point")]
    fn in_place_fix_variable_zero_vars_panics() {
        let mut data = vec![TestScalar::from(1)];
        in_place_fix_variable(&mut data, TestScalar::from(0), 0);
    }
}
