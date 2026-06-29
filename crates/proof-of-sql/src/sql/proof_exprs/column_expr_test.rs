//! Tests for ColumnExpr.

#[cfg(test)]
mod column_expr_test {
    use crate::sql::proof_exprs::column_expr::ColumnExpr;

    #[test]
    fn test_column_expr_type_exists() {
        let _: Option<ColumnExpr> = None;
    }

    #[test]
    fn test_column_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<ColumnExpr>());
        assert!(!debug_str.is_empty());
    }
}
