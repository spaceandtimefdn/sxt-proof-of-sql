//! Tests for ScalarProduct.

#[cfg(test)]
mod scalar_product_test {
    use crate::proof_primitive::dory::scalar_product;

    #[test]
    fn test_scalar_product_functions_exist() {
        let _ = scalar_product::scalar_product_prove::<()>;
        let _ = scalar_product::scalar_product_verify::<()>;
    }
}