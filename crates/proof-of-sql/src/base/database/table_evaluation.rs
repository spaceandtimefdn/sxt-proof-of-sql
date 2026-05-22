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
    fn stores_column_evaluations_and_chi_metadata() {
        let table_evaluation = TableEvaluation::new(
            vec![
                TestScalar::from(7u64),
                TestScalar::from(11u64),
                TestScalar::from(13u64),
            ],
            (TestScalar::from(5u64), 8),
        );

        assert_eq!(
            table_evaluation.column_evals(),
            &[
                TestScalar::from(7u64),
                TestScalar::from(11u64),
                TestScalar::from(13u64),
            ],
        );
        assert_eq!(table_evaluation.chi_eval(), TestScalar::from(5u64));
        assert_eq!(table_evaluation.chi(), (TestScalar::from(5u64), 8));
    }

    #[test]
    fn supports_empty_column_evaluation_results() {
        let table_evaluation = TableEvaluation::new(vec![], (TestScalar::from(3u64), 0));

        assert!(table_evaluation.column_evals().is_empty());
        assert_eq!(table_evaluation.chi_eval(), TestScalar::from(3u64));
        assert_eq!(table_evaluation.chi(), (TestScalar::from(3u64), 0));
    }

    #[test]
    fn clone_and_equality_preserve_all_fields() {
        let table_evaluation = TableEvaluation::new(
            vec![TestScalar::from(17u64), TestScalar::from(19u64)],
            (TestScalar::from(23u64), 2),
        );

        let cloned = table_evaluation.clone();

        assert_eq!(cloned, table_evaluation);
        assert_eq!(cloned.column_evals(), table_evaluation.column_evals());
        assert_eq!(cloned.chi(), table_evaluation.chi());
    }
}
