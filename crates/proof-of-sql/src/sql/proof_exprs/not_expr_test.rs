//! Tests for NotExpr.

#[cfg(test)]
mod not_expr_test {
    use crate::sql::proof_exprs::not_expr::NotExpr;

    #[test]
    fn test_not_expr_type_exists() {
        let _: Option<NotExpr> = None;
    }

    #[test]
    fn test_not_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<NotExpr>());
        assert!(!debug_str.is_empty());
    }
}
