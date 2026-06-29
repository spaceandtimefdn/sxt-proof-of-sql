//! Tests for TableEvaluation.

#[cfg(test)]
mod table_evaluation_test {
    use crate::base::database::TableEvaluation;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_table_evaluation_new() {
        let eval = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ONE, TestScalar::ZERO],
            (TestScalar::ONE, 2),
        );
        assert_eq!(eval.column_evals().len(), 2);
        assert_eq!(eval.chi(), (TestScalar::ONE, 2));
    }

    #[test]
    fn test_table_evaluation_column_evals() {
        let eval = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ONE, TestScalar::TWO, TestScalar::ZERO],
            (TestScalar::ONE, 3),
        );
        assert_eq!(eval.column_evals()[0], TestScalar::ONE);
        assert_eq!(eval.column_evals()[1], TestScalar::TWO);
        assert_eq!(eval.column_evals()[2], TestScalar::ZERO);
    }

    #[test]
    fn test_table_evaluation_chi_eval() {
        let eval = TableEvaluation::<TestScalar>::new(
            vec![],
            (TestScalar::from(5i64), 10),
        );
        assert_eq!(eval.chi_eval(), TestScalar::from(5i64));
    }

    #[test]
    fn test_table_evaluation_clone() {
        let eval = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ONE],
            (TestScalar::ONE, 1),
        );
        let cloned = eval.clone();
        assert_eq!(eval, cloned);
    }

    #[test]
    fn test_table_evaluation_partial_eq() {
        let eval1 = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ONE],
            (TestScalar::ONE, 1),
        );
        let eval2 = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ONE],
            (TestScalar::ONE, 1),
        );
        let eval3 = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ZERO],
            (TestScalar::ONE, 1),
        );
        assert_eq!(eval1, eval2);
        assert_ne!(eval1, eval3);
    }

    #[test]
    fn test_table_evaluation_debug() {
        let eval = TableEvaluation::<TestScalar>::new(
            vec![TestScalar::ONE],
            (TestScalar::ONE, 1),
        );
        let debug_str = format!("{:?}", eval);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_table_evaluation_empty() {
        let eval = TableEvaluation::<TestScalar>::new(
            vec![],
            (TestScalar::ZERO, 0),
        );
        assert!(eval.column_evals().is_empty());
        assert_eq!(eval.chi(), (TestScalar::ZERO, 0));
    }
}