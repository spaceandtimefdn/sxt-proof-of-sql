//! Tests for DynProofExpr.

#[cfg(test)]
mod dyn_proof_expr_test {
    use crate::sql::proof_exprs::{DynProofExpr, LiteralExpr};
    use crate::base::database::{ColumnType, LiteralValue};
    use alloc::string::ToString;

    #[test]
    fn test_dyn_proof_expr_literal_debug() {
        let expr = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(42)));
        let debug_str = format!("{:?}", expr);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_dyn_proof_expr_literal_clone() {
        let expr = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(42)));
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_dyn_proof_expr_literal_serialize() {
        let expr = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(42)));
        let serialized = serde_json::to_string(&expr).unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_dyn_proof_expr_literal_data_type() {
        let expr = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(42)));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_dyn_proof_expr_literal_partial_eq() {
        let expr1 = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(42)));
        let expr2 = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(42)));
        let expr3 = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(100)));
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }

    #[test]
    fn test_dyn_proof_expr_placeholder() {
        let expr = DynProofExpr::Placeholder(crate::sql::proof_exprs::PlaceholderExpr);
        let debug_str = format!("{:?}", expr);
        assert!(!debug_str.is_empty());
    }
}