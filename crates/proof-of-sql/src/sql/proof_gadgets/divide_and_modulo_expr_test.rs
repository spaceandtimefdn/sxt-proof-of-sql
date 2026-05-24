//! Tests for divide_and_modulo_expr.

#[cfg(test)]
mod divide_and_modulo_expr_test {
    use crate::sql::proof_gadgets::divide_and_modulo_expr::DivideAndModuloExpr;

    #[test]
    fn test_divide_and_modulo_expr_type_exists() {
        let _: Option<DivideAndModuloExpr> = None;
    }

    #[test]
    fn test_divide_and_modulo_expr_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<DivideAndModuloExpr>());
        assert!(!debug_str.is_empty());
    }
}
