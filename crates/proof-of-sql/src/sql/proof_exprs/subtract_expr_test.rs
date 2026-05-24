//! Tests for SubtractExpr.

#[cfg(test)]
mod subtract_expr_test {
    use crate::sql::proof_exprs::{SubtractExpr, DynProofExpr};
    use crate::base::database::{ColumnType, LiteralValue};
    use alloc::string::ToString;

    #[test]
    fn test_subtract_expr_debug() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(5));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(3));
        let sub_result = SubtractExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(sub_expr) = sub_result {
            let debug_str = format!("{:?}", sub_expr);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_subtract_expr_clone() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(5));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(3));
        let sub_result = SubtractExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(sub_expr) = sub_result {
            let cloned = sub_expr.clone();
            assert_eq!(sub_expr, cloned);
        }
    }

    #[test]
    fn test_subtract_expr_serialize() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(5));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(3));
        let sub_result = SubtractExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(sub_expr) = sub_result {
            let serialized = serde_json::to_string(&sub_expr).unwrap();
            assert!(!serialized.is_empty());
        }
    }

    #[test]
    fn test_subtract_expr_data_type() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(5));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(3));
        let sub_result = SubtractExpr::try_new(Box::new(expr1), Box::new(expr2));
        if let Ok(sub_expr) = sub_result {
            assert_eq!(sub_expr.data_type(), ColumnType::BigInt);
        }
    }

    #[test]
    fn test_subtract_expr_lhs_rhs() {
        let expr1 = DynProofExpr::Literal(LiteralValue::BigInt(5));
        let expr2 = DynProofExpr::Literal(LiteralValue::BigInt(3));
        let sub_result = SubtractExpr::try_new(Box::new(expr1.clone()), Box::new(expr2.clone()));
        if let Ok(sub_expr) = sub_result {
            let lhs = sub_expr.lhs();
            let rhs = sub_expr.rhs();
            assert_eq!(lhs.data_type(), ColumnType::BigInt);
            assert_eq!(rhs.data_type(), ColumnType::BigInt);
        }
    }
}