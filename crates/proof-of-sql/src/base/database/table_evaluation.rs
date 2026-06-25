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

    #[test]
    fn new_creates_struct_with_correct_evals() {
        let evals = alloc::vec![ts(1), ts(2)];
        let te = TableEvaluation::new(evals.clone(), (ts(3), 5));
        assert_eq!(te.column_evals(), &[ts(1), ts(2)]);
    }

    #[test]
    fn chi_eval_returns_correct_scalar() {
        let te = TableEvaluation::<TestScalar>::new(alloc::vec![], (ts(7), 10));
        assert_eq!(te.chi_eval(), ts(7));
    }

    #[test]
    fn chi_returns_tuple() {
        let te = TableEvaluation::<TestScalar>::new(alloc::vec![], (ts(4), 8));
        assert_eq!(te.chi(), (ts(4), 8));
    }

    #[test]
    fn column_evals_is_empty_for_empty_input() {
        let te = TableEvaluation::<TestScalar>::new(alloc::vec![], (ts(0), 0));
        assert!(te.column_evals().is_empty());
    }

    #[test]
    fn equality_holds_for_same_values() {
        let a = TableEvaluation::new(alloc::vec![ts(1)], (ts(2), 3));
        let b = TableEvaluation::new(alloc::vec![ts(1)], (ts(2), 3));
        assert_eq!(a, b);
    }

    #[test]
    fn clone_produces_equal_value() {
        let te = TableEvaluation::new(alloc::vec![ts(5)], (ts(6), 7));
        assert_eq!(te.clone(), te);
    }

    #[test]
    fn debug_contains_struct_name() {
        let te = TableEvaluation::<TestScalar>::new(alloc::vec![], (ts(0), 0));
        assert!(alloc::format!("{te:?}").contains("TableEvaluation"));
    }
}
