//! Tests for LiteralExpr.

#[cfg(test)]
mod literal_expr_test {
    use crate::sql::proof_exprs::literal_expr::LiteralExpr;
    use crate::base::database::LiteralValue;

    #[test]
    fn test_literal_expr_new() {
        let value = LiteralValue::Boolean(true);
        let expr = LiteralExpr::new(value.clone());
        assert_eq!(expr.value(), &value);
    }

    #[test]
    fn test_literal_expr_type_exists() {
        let _: Option<LiteralExpr> = None;
    }

    #[test]
    fn test_literal_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<LiteralExpr>());
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_literal_expr_partial_eq() {
        let expr1 = LiteralExpr::new(LiteralValue::Boolean(true));
        let expr2 = LiteralExpr::new(LiteralValue::Boolean(true));
        let expr3 = LiteralExpr::new(LiteralValue::BigInt(123));
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }
}
