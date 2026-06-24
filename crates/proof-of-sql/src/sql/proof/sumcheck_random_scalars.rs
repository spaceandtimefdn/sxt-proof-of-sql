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
    use alloc::vec;

    #[test]
    fn new_splits_scalars_correctly() {
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(1u64), // subpolynomial multiplier
            TestScalar::from(2u64), // subpolynomial multiplier
            TestScalar::from(3u64), // entrywise point (1 sumcheck variable)
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 4, 1);
        assert_eq!(srs.subpolynomial_multipliers.len(), 2);
        assert_eq!(srs.entrywise_point.len(), 1);
    }

    #[test]
    fn entrywise_point_is_last_num_vars_scalars() {
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(10u64),
            TestScalar::from(20u64),
            TestScalar::from(30u64),
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 2, 2);
        assert_eq!(srs.entrywise_point[0], TestScalar::from(20u64));
        assert_eq!(srs.entrywise_point[1], TestScalar::from(30u64));
    }

    #[test]
    fn subpolynomial_multipliers_is_first_elements() {
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(100u64),
            TestScalar::from(200u64),
            TestScalar::from(300u64),
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 2, 1);
        assert_eq!(srs.subpolynomial_multipliers[0], TestScalar::from(100u64));
        assert_eq!(srs.subpolynomial_multipliers[1], TestScalar::from(200u64));
    }

    #[test]
    fn table_length_stored_correctly() {
        let scalars: alloc::vec::Vec<TestScalar> = vec![TestScalar::from(0u64), TestScalar::from(1u64)];
        let srs = SumcheckRandomScalars::new(&scalars, 7, 1);
        assert_eq!(srs.table_length, 7);
    }

    #[test]
    fn compute_entrywise_multipliers_returns_correct_length() {
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(1u64),
            TestScalar::from(0u64), // entrywise point
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 2, 1);
        let multipliers = srs.compute_entrywise_multipliers();
        assert_eq!(multipliers.len(), 2);
    }

    #[test]
    fn compute_entrywise_multipliers_with_zero_point_first_element_is_one() {
        // entrywise_point = [0]: evaluation_vector = [1, 0]
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(5u64), // subpolynomial multiplier
            TestScalar::from(0u64), // entrywise point = 0
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 2, 1);
        let multipliers = srs.compute_entrywise_multipliers();
        assert_eq!(multipliers[0], TestScalar::from(1u64));
        assert_eq!(multipliers[1], TestScalar::from(0u64));
    }

    #[test]
    fn compute_entrywise_multipliers_with_one_point_second_element_is_one() {
        // entrywise_point = [1]: evaluation_vector = [0, 1]
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(5u64), // subpolynomial multiplier
            TestScalar::from(1u64), // entrywise point = 1
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 2, 1);
        let multipliers = srs.compute_entrywise_multipliers();
        assert_eq!(multipliers[0], TestScalar::from(0u64));
        assert_eq!(multipliers[1], TestScalar::from(1u64));
    }

    #[test]
    fn all_scalars_are_entrywise_point_when_no_subpolynomials() {
        let scalars: alloc::vec::Vec<TestScalar> = vec![
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        ];
        let srs = SumcheckRandomScalars::new(&scalars, 4, 2);
        assert_eq!(srs.subpolynomial_multipliers.len(), 0);
        assert_eq!(srs.entrywise_point.len(), 2);
    }
}
