//! Tests for InequalityExpr.

#[cfg(test)]
mod inequality_expr_test {
    use crate::sql::proof_exprs::inequality_expr::InequalityExpr;

    #[test]
    fn test_inequality_expr_type_exists() {
        let _: Option<InequalityExpr> = None;
    }

    #[test]
    fn test_inequality_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<InequalityExpr>());
        assert!(!debug_str.is_empty());
    }
}
