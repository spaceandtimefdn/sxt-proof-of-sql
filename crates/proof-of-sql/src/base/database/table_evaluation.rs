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
    fn we_can_create_and_access_table_evaluation() {
        let column_evals = vec![TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)];
        let chi = (TestScalar::from(42), 5);
        let eval = TableEvaluation::new(column_evals.clone(), chi);

        assert_eq!(eval.column_evals(), &column_evals);
        assert_eq!(eval.chi_eval(), TestScalar::from(42));
        assert_eq!(eval.chi(), (TestScalar::from(42), 5));
    }

    #[test]
    fn we_can_create_table_evaluation_with_empty_columns() {
        let eval = TableEvaluation::<TestScalar>::new(vec![], (TestScalar::ZERO, 0));

        assert_eq!(eval.column_evals(), &[] as &[TestScalar]);
        assert_eq!(eval.chi_eval(), TestScalar::ZERO);
        assert_eq!(eval.chi(), (TestScalar::ZERO, 0));
    }

    #[test]
    fn table_evaluations_with_same_data_are_equal() {
        let eval_a = TableEvaluation::new(
            vec![TestScalar::from(10)],
            (TestScalar::from(5), 3),
        );
        let eval_b = TableEvaluation::new(
            vec![TestScalar::from(10)],
            (TestScalar::from(5), 3),
        );
        assert_eq!(eval_a, eval_b);
    }

    #[test]
    fn table_evaluations_with_different_data_are_not_equal() {
        let eval_a = TableEvaluation::new(
            vec![TestScalar::from(10)],
            (TestScalar::from(5), 3),
        );
        let eval_b = TableEvaluation::new(
            vec![TestScalar::from(20)],
            (TestScalar::from(5), 3),
        );
        assert_ne!(eval_a, eval_b);
    }

    #[test]
    fn table_evaluations_with_different_chi_are_not_equal() {
        let eval_a = TableEvaluation::new(
            vec![TestScalar::from(10)],
            (TestScalar::from(5), 3),
        );
        let eval_b = TableEvaluation::new(
            vec![TestScalar::from(10)],
            (TestScalar::from(7), 3),
        );
        assert_ne!(eval_a, eval_b);
    }

    #[test]
    fn we_can_clone_table_evaluation() {
        let eval = TableEvaluation::new(
            vec![TestScalar::from(1), TestScalar::from(2)],
            (TestScalar::from(99), 10),
        );
        let cloned = eval.clone();
        assert_eq!(eval, cloned);
    }
}
