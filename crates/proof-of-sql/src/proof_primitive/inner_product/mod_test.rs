#[cfg(test)]
mod inner_product_mod_test {
    #[test]
    fn test_module_items_exist() {
        use crate::proof_primitive::inner_product::{curve_25519_scalar, ristretto_point};
        let _ = curve_25519_scalar::Curve25519Scalar::default();
        let _ = ristretto_point::RistrettoPoint::default();
    }
}
