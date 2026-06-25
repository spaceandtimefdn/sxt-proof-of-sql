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

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    fn make_eval() -> TableEvaluation<TestScalar> {
        TableEvaluation::new(alloc::vec![ts(1), ts(2), ts(3)], (ts(10), 4))
    }

    #[test]
    fn column_evals_returns_correct_values() {
        let e = make_eval();
        assert_eq!(e.column_evals(), &[ts(1), ts(2), ts(3)]);
    }

    #[test]
    fn chi_eval_returns_scalar_part() {
        let e = make_eval();
        assert_eq!(e.chi_eval(), ts(10));
    }

    #[test]
    fn chi_returns_both_parts() {
        let e = make_eval();
        assert_eq!(e.chi(), (ts(10), 4));
    }

    #[test]
    fn equality_on_identical_values() {
        assert_eq!(make_eval(), make_eval());
    }

    #[test]
    fn inequality_on_different_column_evals() {
        let a = TableEvaluation::new(alloc::vec![ts(1)], (ts(5), 1));
        let b = TableEvaluation::new(alloc::vec![ts(2)], (ts(5), 1));
        assert_ne!(a, b);
    }

    #[test]
    fn inequality_on_different_chi() {
        let a = TableEvaluation::new(alloc::vec![ts(1)], (ts(5), 1));
        let b = TableEvaluation::new(alloc::vec![ts(1)], (ts(6), 1));
        assert_ne!(a, b);
    }

    #[test]
    fn clone_produces_equal_value() {
        let e = make_eval();
        assert_eq!(e.clone(), e);
    }

    #[test]
    fn debug_formatting_works() {
        let e = make_eval();
        let s = alloc::format!("{e:?}");
        assert!(s.contains("TableEvaluation"));
    }

    #[test]
    fn empty_column_evals_is_valid() {
        let e = TableEvaluation::<TestScalar>::new(alloc::vec![], (ts(0), 0));
        assert_eq!(e.column_evals(), &[]);
        assert_eq!(e.chi_eval(), ts(0));
    }
}
