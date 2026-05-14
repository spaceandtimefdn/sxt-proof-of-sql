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
    fn table_evaluation_returns_column_and_chi_values() {
        let evaluation = TableEvaluation::new(
            vec![TestScalar::from(3_u64), TestScalar::from(5_u64)],
            (TestScalar::from(11_u64), 2),
        );

        assert_eq!(
            evaluation.column_evals(),
            &[TestScalar::from(3_u64), TestScalar::from(5_u64)]
        );
        assert_eq!(evaluation.chi_eval(), TestScalar::from(11_u64));
        assert_eq!(evaluation.chi(), (TestScalar::from(11_u64), 2));
    }
}
