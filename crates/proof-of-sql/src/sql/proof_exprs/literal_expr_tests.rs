/// Tests for [`LiteralExpr`] — covering construction, `data_type()`, and the
/// conversion from `LiteralValue` that is partially missed by higher-level
/// tests.
#[cfg(test)]
mod tests {
    use crate::base::database::ColumnType;
    use crate::sql::proof_exprs::LiteralExpr;
    use proof_of_sql_parser::intermediate_ast::LiteralValue;

    // -----------------------------------------------------------------------
    // Construction and data_type()
    // -----------------------------------------------------------------------

    #[test]
    fn test_boolean_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(true));
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn test_boolean_false_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(false));
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn test_bigint_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(42));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_bigint_negative_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(-1));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_int128_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::Int128(i128::MAX));
        assert_eq!(expr.data_type(), ColumnType::Int128);
    }

    #[test]
    fn test_varchar_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::VarChar("hello".to_string()));
        assert_eq!(expr.data_type(), ColumnType::VarChar);
    }

    #[test]
    fn test_varchar_empty_literal_data_type() {
        let expr = LiteralExpr::new(LiteralValue::VarChar(String::new()));
        assert_eq!(expr.data_type(), ColumnType::VarChar);
    }

    // -----------------------------------------------------------------------
    // Debug / Clone
    // -----------------------------------------------------------------------

    #[test]
    fn test_literal_expr_debug_is_non_empty() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(0));
        assert!(!format!("{:?}", expr).is_empty());
    }

    #[test]
    fn test_literal_expr_clone() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(true));
        let cloned = expr.clone();
        assert_eq!(expr.data_type(), cloned.data_type());
    }

    // -----------------------------------------------------------------------
    // PartialEq
    // -----------------------------------------------------------------------

    #[test]
    fn test_equal_literal_exprs_compare_equal() {
        let a = LiteralExpr::new(LiteralValue::BigInt(7));
        let b = LiteralExpr::new(LiteralValue::BigInt(7));
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_literal_exprs_are_not_equal() {
        let a = LiteralExpr::new(LiteralValue::BigInt(1));
        let b = LiteralExpr::new(LiteralValue::BigInt(2));
        assert_ne!(a, b);
    }

    #[test]
    fn test_different_type_literal_exprs_are_not_equal() {
        let a = LiteralExpr::new(LiteralValue::Boolean(true));
        let b = LiteralExpr::new(LiteralValue::BigInt(1));
        assert_ne!(a, b);
    }
}
