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

    #[test]
    fn new_stores_column_evals_and_chi() {
        let evals = vec![TestScalar::from(1), TestScalar::from(2)];
        let chi = (TestScalar::from(3), 5);
        let te = TableEvaluation::new(evals.clone(), chi);
        assert_eq!(te.column_evals(), evals.as_slice());
        assert_eq!(te.chi_eval(), TestScalar::from(3));
        assert_eq!(te.chi(), (TestScalar::from(3), 5));
    }

    #[test]
    fn column_evals_returns_correct_slice() {
        let evals = vec![TestScalar::from(10), TestScalar::from(20), TestScalar::from(30)];
        let te = TableEvaluation::new(evals.clone(), (TestScalar::from(0), 3));
        assert_eq!(te.column_evals().len(), 3);
        assert_eq!(te.column_evals()[1], TestScalar::from(20));
    }

    #[test]
    fn chi_eval_returns_first_element_of_chi() {
        let te = TableEvaluation::new(vec![], (TestScalar::from(42), 100));
        assert_eq!(te.chi_eval(), TestScalar::from(42));
    }

    #[test]
    fn chi_returns_full_tuple() {
        let te = TableEvaluation::new(vec![], (TestScalar::from(7), 99));
        assert_eq!(te.chi(), (TestScalar::from(7), 99));
    }

    #[test]
    fn empty_column_evals_is_valid() {
        let te = TableEvaluation::<TestScalar>::new(vec![], (TestScalar::from(0), 0));
        assert_eq!(te.column_evals().len(), 0);
    }

    #[test]
    fn table_evaluation_equality() {
        let te1 = TableEvaluation::new(
            vec![TestScalar::from(1)],
            (TestScalar::from(2), 5),
        );
        let te2 = TableEvaluation::new(
            vec![TestScalar::from(1)],
            (TestScalar::from(2), 5),
        );
        assert_eq!(te1, te2);
    }

    #[test]
    fn table_evaluation_inequality_different_evals() {
        let te1 = TableEvaluation::new(vec![TestScalar::from(1)], (TestScalar::from(0), 1));
        let te2 = TableEvaluation::new(vec![TestScalar::from(2)], (TestScalar::from(0), 1));
        assert_ne!(te1, te2);
    }

    #[test]
    fn table_evaluation_clone_equals_original() {
        let te = TableEvaluation::new(
            vec![TestScalar::from(5), TestScalar::from(6)],
            (TestScalar::from(1), 2),
        );
        assert_eq!(te.clone(), te);
    }
}
