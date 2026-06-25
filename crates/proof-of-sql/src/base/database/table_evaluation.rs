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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::vec;

    #[test]
    fn table_evaluation_accessors_return_constructor_values() {
        let evaluation = TableEvaluation::new(
            vec![TestScalar::from(3), TestScalar::from(5)],
            (TestScalar::from(8), 4),
        );

        assert_eq!(
            evaluation.column_evals(),
            &[TestScalar::from(3), TestScalar::from(5)]
        );
        assert_eq!(evaluation.chi_eval(), TestScalar::from(8));
        assert_eq!(evaluation.chi(), (TestScalar::from(8), 4));
    }

    #[test]
    fn table_evaluation_clone_preserves_values() {
        let evaluation = TableEvaluation::new(vec![TestScalar::from(1)], (TestScalar::from(2), 3));

        assert_eq!(evaluation.clone(), evaluation);
    }
}
