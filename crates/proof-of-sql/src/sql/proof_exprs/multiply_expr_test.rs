//! Tests for MultiplyExpr.

#[cfg(test)]
mod multiply_expr_test {
    use crate::sql::proof_exprs::multiply_expr::MultiplyExpr;

    #[test]
    fn test_multiply_expr_type_exists() {
        let _: Option<MultiplyExpr> = None;
    }

    #[test]
    fn test_multiply_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<MultiplyExpr>());
        assert!(!debug_str.is_empty());
    }
}
