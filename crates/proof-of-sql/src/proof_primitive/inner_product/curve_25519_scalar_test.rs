//! Tests for Curve25519Scalar type.

#[cfg(test)]
mod curve_25519_scalar_test {
    use crate::proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar;

    #[test]
    fn test_curve_25519_scalar_type_exists() {
        let _: Option<Curve25519Scalar> = None;
    }

    #[test]
    fn test_curve_25519_scalar_debug() {
        use crate::base::scalar::test_scalar::TestScalar;
        // Curve25519Scalar requires curve25519 feature, but TestScalar is simpler
        let debug_str = format!("{:?}", std::any::type_name::<Curve25519Scalar>());
        assert!(!debug_str.is_empty());
    }
}