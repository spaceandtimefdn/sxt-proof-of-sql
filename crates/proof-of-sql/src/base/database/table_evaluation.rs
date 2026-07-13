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

    #[test]
    fn test_new_and_accessors() {
        let evals: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
        let chi = (TestScalar::from(3u64), 4usize);
        let te = TableEvaluation::new(evals.clone(), chi);
        assert_eq!(te.column_evals(), evals.as_slice());
        assert_eq!(te.chi_eval(), TestScalar::from(3u64));
        assert_eq!(te.chi(), chi);
    }

    #[test]
    fn test_equality() {
        let a = TableEvaluation::new(
            vec![TestScalar::from(1u64)],
            (TestScalar::from(2u64), 3usize),
        );
        let b = TableEvaluation::new(
            vec![TestScalar::from(1u64)],
            (TestScalar::from(2u64), 3usize),
        );
        let c = TableEvaluation::new(
            vec![TestScalar::from(9u64)],
            (TestScalar::from(2u64), 3usize),
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_clone() {
        let original = TableEvaluation::new(
            vec![TestScalar::from(5u64)],
            (TestScalar::from(6u64), 7usize),
        );
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_empty_column_evals() {
        let te: TableEvaluation<TestScalar> =
            TableEvaluation::new(vec![], (TestScalar::from(0u64), 0usize));
        assert!(te.column_evals().is_empty());
        assert_eq!(te.chi_eval(), TestScalar::from(0u64));
    }
}
