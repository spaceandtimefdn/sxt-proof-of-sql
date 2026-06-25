use crate::base::{polynomial::compute_evaluation_vector, scalar::Scalar};
use alloc::{vec, vec::Vec};

/// Accessor for the random scalars used to form the sumcheck polynomial of a query proof
pub struct SumcheckRandomScalars<'a, S: Scalar> {
    pub entrywise_point: &'a [S],
    pub subpolynomial_multipliers: &'a [S],
    pub table_length: usize,
}

impl<'a, S: Scalar> SumcheckRandomScalars<'a, S> {
    pub fn new(scalars: &'a [S], table_length: usize, num_sumcheck_variables: usize) -> Self {
        let num_subpolynomial_multipliers = scalars.len() - num_sumcheck_variables;
        let (subpolynomial_multipliers, entrywise_point) =
            scalars.split_at(num_subpolynomial_multipliers);
        Self {
            entrywise_point,
            subpolynomial_multipliers,
            table_length,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn compute_entrywise_multipliers(&self) -> Vec<S> {
        let mut v = vec![S::ZERO; self.table_length];
        compute_evaluation_vector(&mut v, self.entrywise_point);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::SumcheckRandomScalars;
    use crate::base::scalar::test_scalar::TestScalar;

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn new_splits_scalars_correctly() {
        let scalars = alloc::vec![ts(10), ts(1), ts(2)];
        let s = SumcheckRandomScalars::new(&scalars, 2, 2);
        assert_eq!(s.subpolynomial_multipliers, &[ts(10)]);
        assert_eq!(s.entrywise_point, &[ts(1), ts(2)]);
    }

    #[test]
    fn table_length_is_stored() {
        let scalars = alloc::vec![ts(1), ts(2)];
        let s = SumcheckRandomScalars::new(&scalars, 4, 1);
        assert_eq!(s.table_length, 4);
    }

    #[test]
    fn compute_entrywise_multipliers_returns_vec_of_table_length() {
        let scalars = alloc::vec![ts(1)]; // 1 scalar, 1 var, 0 subpoly
        let s = SumcheckRandomScalars::new(&scalars, 2, 1);
        let result = s.compute_entrywise_multipliers();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn compute_entrywise_multipliers_at_zero_gives_all_ones_and_zeros() {
        // entrywise_point = [0], table_length = 2
        // eval vector at 0 should be [1, 0] (1-t, t) evaluated at t=0
        let scalars = alloc::vec![ts(0)];
        let s = SumcheckRandomScalars::new(&scalars, 2, 1);
        let result = s.compute_entrywise_multipliers();
        assert_eq!(result[0], ts(1));
        assert_eq!(result[1], ts(0));
    }
}
