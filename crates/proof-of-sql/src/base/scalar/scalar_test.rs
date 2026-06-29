//! Tests for Scalar trait.

#[cfg(test)]
mod scalar_test {
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_scalar_type_exists() {
        let _: Option<TestScalar> = None;
    }

    #[test]
    fn test_scalar_constants() {
        assert_eq!(TestScalar::ZERO, TestScalar::from(0u64));
        assert_eq!(TestScalar::ONE, TestScalar::from(1u64));
        assert_eq!(TestScalar::TWO, TestScalar::from(2u64));
        assert_eq!(TestScalar::TEN, TestScalar::from(10u64));
    }
}
