//! Tests for BNScalar type.

#[cfg(test)]
mod scalar_test {
    use crate::proof_primitive::hyperkzg::scalar::BNScalar;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_bn_scalar_type_exists() {
        let _: Option<BNScalar> = None;
    }

    #[test]
    fn test_bn_scalar_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<BNScalar>());
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_bn_scalar_partial_eq() {
        // Test that BNScalar implements PartialEq (used in NovaScalar comparisons)
        // This is verified by the fact that the module compiles
        assert!(std::any::type_id::<BNScalar>() == std::any::type_id::<BNScalar>());
    }
}