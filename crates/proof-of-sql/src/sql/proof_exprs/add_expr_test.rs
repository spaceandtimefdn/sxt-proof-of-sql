//! Tests for AddExpr.

#[cfg(test)]
mod add_expr_test {
    use crate::sql::proof_exprs::{AddExpr, DynProofExpr};
    use crate::base::database::{ColumnType, LiteralValue};
    use alloc::string::ToString;

    #[test]
    fn test_add_expr_debug() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(1));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(2));
        let add_result = AddExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(add_expr) = add_result {
            let debug_str = format!("{:?}", add_expr);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_add_expr_clone() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(1));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(2));
        let add_result = AddExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(add_expr) = add_result {
            let cloned = add_expr.clone();
            assert_eq!(add_expr, cloned);
        }
    }

    #[test]
    fn test_add_expr_serialize() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(1));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(2));
        let add_result = AddExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(add_expr) = add_result {
            let serialized = serde_json::to_string(&add_expr).unwrap();
            assert!(!serialized.is_empty());
        }
    }

    #[test]
    fn test_add_expr_data_type() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(1));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(2));
        let add_result = AddExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(add_expr) = add_result {
            assert_eq!(add_expr.data_type(), ColumnType::BigInt);
        }
    }

    #[test]
    fn test_add_expr_lhs_rhs() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(1));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(2));
        let add_result = AddExpr::try_new(Box::new(expr1.clone()), Box::new(expr2.clone()));
        if let Ok(add_expr) = add_result {
            let lhs = add_expr.lhs();
            let rhs = add_expr.rhs();
            assert_eq!(lhs.data_type(), ColumnType::BigInt);
            assert_eq!(rhs.data_type(), ColumnType::BigInt);
        }
    }
}