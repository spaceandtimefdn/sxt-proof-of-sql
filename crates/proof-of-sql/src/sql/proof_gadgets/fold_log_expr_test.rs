//! Tests for FoldLogExpr.

#[cfg(test)]
mod fold_log_expr_test {
    use crate::sql::proof_gadgets::FoldLogExpr;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_fold_log_expr_new() {
        let expr = FoldLogExpr::<TestScalar>::new(TestScalar::ONE, TestScalar::ZERO);
        assert_eq!(expr.alpha, TestScalar::ONE);
        assert_eq!(expr.beta, TestScalar::ZERO);
    }

    #[test]
    fn test_fold_log_expr_debug() {
        let expr = FoldLogExpr::<TestScalar>::new(TestScalar::ONE, TestScalar::ZERO);
        let debug_str = format!("{:?}", expr);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_fold_log_expr_clone() {
        let expr = FoldLogExpr::<TestScalar>::new(TestScalar::ONE, TestScalar::ZERO);
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_fold_log_expr_partial_eq() {
        let expr1 = FoldLogExpr::<TestScalar>::new(TestScalar::ONE, TestScalar::ZERO);
        let expr2 = FoldLogExpr::<TestScalar>::new(TestScalar::ONE, TestScalar::ZERO);
        let expr3 = FoldLogExpr::<TestScalar>::new(TestScalar::ZERO, TestScalar::ONE);
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }

    #[test]
    fn test_fold_log_expr_serialize() {
        let expr = FoldLogExpr::<TestScalar>::new(TestScalar::ONE, TestScalar::ZERO);
        let serialized = serde_json::to_string(&expr).unwrap();
        assert!(!serialized.is_empty());
    }
}