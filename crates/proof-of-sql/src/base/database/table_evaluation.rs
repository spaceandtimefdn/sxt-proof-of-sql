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
    fn new_preserves_column_and_chi_evaluations() {
        let column_evals = vec![TestScalar::from(11), TestScalar::from(29)];
        let chi = (TestScalar::from(7), 4);

        let table_evaluation = TableEvaluation::new(column_evals.clone(), chi);

        assert_eq!(table_evaluation.column_evals(), column_evals.as_slice());
        assert_eq!(table_evaluation.chi_eval(), chi.0);
        assert_eq!(table_evaluation.chi(), chi);
    }

    #[test]
    fn new_supports_tables_without_column_evaluations() {
        let chi = (TestScalar::from(13), 0);

        let table_evaluation = TableEvaluation::new(vec![], chi);

        assert!(table_evaluation.column_evals().is_empty());
        assert_eq!(table_evaluation.chi_eval(), chi.0);
        assert_eq!(table_evaluation.chi(), chi);
    }
}
