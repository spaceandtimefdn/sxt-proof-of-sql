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
        let scalars = alloc::vec![ts(1), ts(2), ts(3)];
        let sr = SumcheckRandomScalars::new(&scalars, 4, 2);
        assert_eq!(sr.subpolynomial_multipliers, &scalars[..1]);
        assert_eq!(sr.entrywise_point, &scalars[1..]);
        assert_eq!(sr.table_length, 4);
    }

    #[test]
    fn new_all_scalars_as_entrywise_point() {
        let scalars = alloc::vec![ts(5), ts(6)];
        let sr = SumcheckRandomScalars::new(&scalars, 2, 2);
        assert_eq!(sr.subpolynomial_multipliers.len(), 0);
        assert_eq!(sr.entrywise_point.len(), 2);
    }

    #[test]
    fn compute_entrywise_multipliers_length_equals_table_length() {
        let scalars = alloc::vec![ts(1), ts(0)];
        let sr = SumcheckRandomScalars::new(&scalars, 2, 1);
        let mult = sr.compute_entrywise_multipliers();
        assert_eq!(mult.len(), 2);
    }

    #[test]
    fn compute_entrywise_multipliers_with_zero_point_is_all_ones() {
        let scalars = alloc::vec![ts(99), ts(0)];
        let sr = SumcheckRandomScalars::new(&scalars, 2, 1);
        let mult = sr.compute_entrywise_multipliers();
        // With point=[0]: evaluation vector = [1-0, 0] = [1, 0]
        assert_eq!(mult[0], ts(1));
        assert_eq!(mult[1], ts(0));
    }

    #[test]
    fn compute_entrywise_multipliers_with_one_point() {
        let scalars = alloc::vec![ts(99), ts(1)];
        let sr = SumcheckRandomScalars::new(&scalars, 2, 1);
        let mult = sr.compute_entrywise_multipliers();
        // With point=[1]: evaluation vector = [1-1, 1] = [0, 1]
        assert_eq!(mult[0], ts(0));
        assert_eq!(mult[1], ts(1));
    }
}
