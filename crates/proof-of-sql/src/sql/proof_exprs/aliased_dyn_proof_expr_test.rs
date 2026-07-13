//! Tests for AliasedDynProofExpr.

#[cfg(test)]
mod aliased_dyn_proof_expr_test {
    use crate::sql::proof_exprs::aliased_dyn_proof_expr::AliasedDynProofExpr;

    #[test]
    fn test_aliased_dyn_proof_expr_type_exists() {
        let _: Option<AliasedDynProofExpr> = None;
    }

    #[test]
    fn test_aliased_dyn_proof_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<AliasedDynProofExpr>());
        assert!(!debug_str.is_empty());
    }
}
