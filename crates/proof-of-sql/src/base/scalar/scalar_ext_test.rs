#[cfg(test)]
mod tests {
    use crate::base::scalar::{Curve25519Scalar, ScalarExt};

    // -----------------------------------------------------------------------
    // ScalarExt::from_str
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_str_zero() {
        let s: Curve25519Scalar = ScalarExt::from_str("0").expect("parse 0");
        assert_eq!(s, Curve25519Scalar::from(0_i64));
    }

    #[test]
    fn test_from_str_positive_integer() {
        let s: Curve25519Scalar = ScalarExt::from_str("42").expect("parse 42");
        assert_eq!(s, Curve25519Scalar::from(42_i64));
    }

    #[test]
    fn test_from_str_negative_integer() {
        let s: Curve25519Scalar = ScalarExt::from_str("-7").expect("parse -7");
        assert_eq!(s, Curve25519Scalar::from(-7_i64));
    }

    #[test]
    fn test_from_str_large_positive() {
        let s: Curve25519Scalar = ScalarExt::from_str("1000000000").expect("parse large");
        assert_eq!(s, Curve25519Scalar::from(1_000_000_000_i64));
    }

    #[test]
    fn test_from_str_invalid_returns_error() {
        let result: Result<Curve25519Scalar, _> = ScalarExt::from_str("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_empty_string_returns_error() {
        let result: Result<Curve25519Scalar, _> = ScalarExt::from_str("");
        assert!(result.is_err());
    }
}
