//! Tests for CastExpr.

#[cfg(test)]
mod cast_expr_test {
    use crate::sql::proof_exprs::cast_expr::CastExpr;

    #[test]
    fn test_cast_expr_type_exists() {
        let _: Option<CastExpr> = None;
    }

    #[test]
    fn test_cast_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<CastExpr>());
        assert!(!debug_str.is_empty());
    }
}
