//! Tests for ScalingCastExpr.

#[cfg(test)]
mod scaling_cast_expr_test {
    use crate::sql::proof_exprs::scaling_cast_expr::ScalingCastExpr;

    #[test]
    fn test_scaling_cast_expr_type_exists() {
        let _: Option<ScalingCastExpr> = None;
    }

    #[test]
    fn test_scaling_cast_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<ScalingCastExpr>());
        assert!(!debug_str.is_empty());
    }
}
