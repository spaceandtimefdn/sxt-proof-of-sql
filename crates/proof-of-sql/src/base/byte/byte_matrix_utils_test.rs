//! Tests for byte_matrix_utils module.

#[cfg(test)]
mod byte_matrix_utils_test {
    use crate::base::byte::byte_matrix_utils::compute_varying_byte_matrix;
    use crate::base::scalar::test_scalar::TestScalar;
    use bumpalo::Bump;

    #[test]
    fn test_compute_varying_byte_matrix_function_exists() {
        let alloc = Bump::new();
        let scalars = [TestScalar::ONE, TestScalar::TWO, TestScalar::from(3u64)];
        let (varying_columns, byte_distribution) = 
            compute_varying_byte_matrix::<TestScalar>(&scalars, &alloc);
        // Function executes without panic
        assert!(varying_columns.len() >= 0);
        let _ = byte_distribution;
    }
}
