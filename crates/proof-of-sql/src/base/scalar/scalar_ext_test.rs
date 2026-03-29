/// Tests for scalar extension trait implementations
#[cfg(test)]
mod tests {
    use crate::base::scalar::{test_scalar::TestScalar, Scalar, ScalarExt};

    #[test]
    fn test_from_str_valid_positive_integer() {
        let s: TestScalar = TestScalar::from_str("42").unwrap();
        assert_eq!(s, TestScalar::from(42u64));
    }

    #[test]
    fn test_from_str_valid_zero() {
        let s: TestScalar = TestScalar::from_str("0").unwrap();
        assert_eq!(s, TestScalar::ZERO);
    }

    #[test]
    fn test_from_str_valid_negative_integer() {
        let s: TestScalar = TestScalar::from_str("-1").unwrap();
        assert_eq!(s, -TestScalar::ONE);
    }

    #[test]
    fn test_from_str_invalid_returns_error() {
        let result: Result<TestScalar, _> = TestScalar::from_str("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_empty_returns_error() {
        let result: Result<TestScalar, _> = TestScalar::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_large_number() {
        // A large decimal that fits within the scalar field
        let s: TestScalar = TestScalar::from_str("1000000000").unwrap();
        assert_eq!(s, TestScalar::from(1_000_000_000u64));
    }
}
