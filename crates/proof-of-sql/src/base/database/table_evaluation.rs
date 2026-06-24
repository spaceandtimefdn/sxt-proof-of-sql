use crate::base::scalar::Scalar;
use alloc::vec::Vec;

/// The result of evaluating a table
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TableEvaluation<S: Scalar> {
    /// Evaluation of each column in the table
    column_evals: Vec<S>,
    /// Evaluation of an all-one column with the same length as the table
    chi: (S, usize),
}

impl<S: Scalar> TableEvaluation<S> {
    /// Creates a new [`TableEvaluation`].
    #[must_use]
    pub fn new(column_evals: Vec<S>, chi: (S, usize)) -> Self {
        Self { column_evals, chi }
    }

    /// Returns the evaluation of each column in the table.
    #[must_use]
    pub fn column_evals(&self) -> &[S] {
        &self.column_evals
    }

    /// Returns the evaluation of an all-one column with the same length as the table.
    #[must_use]
    pub fn chi_eval(&self) -> S {
        self.chi.0
    }

    /// Returns the evaluation of an all-one column with the same length as the table.
    #[must_use]
    pub fn chi(&self) -> (S, usize) {
        self.chi
    }
}

#[cfg(test)]
mod tests {
    use super::TableEvaluation;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::vec;

    #[test]
    fn new_stores_column_evals_and_chi() {
        let evals = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
        let chi = (TestScalar::from(3u64), 4usize);
        let te = TableEvaluation::new(evals.clone(), chi);
        assert_eq!(te.column_evals(), &evals[..]);
    }

    #[test]
    fn chi_eval_returns_correct_scalar() {
        let evals = vec![TestScalar::from(0u64)];
        let chi_scalar = TestScalar::from(7u64);
        let te = TableEvaluation::new(evals, (chi_scalar, 5));
        assert_eq!(te.chi_eval(), chi_scalar);
    }

    #[test]
    fn chi_returns_both_scalar_and_length() {
        let te = TableEvaluation::<TestScalar>::new(vec![], (TestScalar::from(42u64), 10));
        assert_eq!(te.chi(), (TestScalar::from(42u64), 10));
    }

    #[test]
    fn column_evals_empty_slice() {
        let te = TableEvaluation::<TestScalar>::new(vec![], (TestScalar::from(0u64), 0));
        assert_eq!(te.column_evals().len(), 0);
    }

    #[test]
    fn column_evals_multiple_values() {
        let evals = vec![
            TestScalar::from(10u64),
            TestScalar::from(20u64),
            TestScalar::from(30u64),
        ];
        let te = TableEvaluation::new(evals, (TestScalar::from(0u64), 3));
        assert_eq!(te.column_evals().len(), 3);
        assert_eq!(te.column_evals()[1], TestScalar::from(20u64));
    }

    #[test]
    fn equality_same_data() {
        let evals = vec![TestScalar::from(1u64)];
        let te1 = TableEvaluation::new(evals.clone(), (TestScalar::from(2u64), 5));
        let te2 = TableEvaluation::new(evals, (TestScalar::from(2u64), 5));
        assert_eq!(te1, te2);
    }

    #[test]
    fn inequality_different_chi() {
        let evals = vec![TestScalar::from(1u64)];
        let te1 = TableEvaluation::new(evals.clone(), (TestScalar::from(2u64), 5));
        let te2 = TableEvaluation::new(evals, (TestScalar::from(3u64), 5));
        assert_ne!(te1, te2);
    }

    #[test]
    fn inequality_different_column_evals() {
        let te1 = TableEvaluation::new(vec![TestScalar::from(1u64)], (TestScalar::from(0u64), 1));
        let te2 = TableEvaluation::new(vec![TestScalar::from(2u64)], (TestScalar::from(0u64), 1));
        assert_ne!(te1, te2);
    }

    #[test]
    fn clone_equals_original() {
        let te = TableEvaluation::new(
            vec![TestScalar::from(5u64)],
            (TestScalar::from(1u64), 2),
        );
        assert_eq!(te.clone(), te);
    }

    #[test]
    fn chi_length_accessible() {
        let te = TableEvaluation::<TestScalar>::new(vec![], (TestScalar::from(0u64), 42));
        assert_eq!(te.chi().1, 42);
    }
}
