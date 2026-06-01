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
    use crate::proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar;

    #[test]
    fn we_can_retrieve_column_and_chi_evaluations() {
        let column_evals = vec![Curve25519Scalar::from(3), Curve25519Scalar::from(5)];
        let chi = (Curve25519Scalar::from(7), 2);

        let evaluation = TableEvaluation::new(column_evals.clone(), chi);

        assert_eq!(evaluation.column_evals(), column_evals.as_slice());
        assert_eq!(evaluation.chi_eval(), chi.0);
        assert_eq!(evaluation.chi(), chi);
    }

    #[test]
    fn we_can_clone_table_evaluations() {
        let evaluation = TableEvaluation::new(
            vec![Curve25519Scalar::from(11), Curve25519Scalar::from(13)],
            (Curve25519Scalar::from(17), 3),
        );

        assert_eq!(evaluation.clone(), evaluation);
    }
}
