/// Extra tests for decimal precision and scale handling
#[cfg(test)]
mod tests {
    use crate::base::math::decimal::{DecimalError, Precision};

    #[test]
    fn test_precision_new_valid() {
        let p = Precision::new(1).unwrap();
        assert_eq!(p.value(), 1);

        let p = Precision::new(75).unwrap();
        assert_eq!(p.value(), 75);
    }

    #[test]
    fn test_precision_new_zero_fails() {
        assert!(Precision::new(0).is_err());
    }

    #[test]
    fn test_precision_new_too_large_fails() {
        // Max precision for Decimal75 is 75
        assert!(Precision::new(76).is_err());
    }

    #[test]
    fn test_decimal_error_display() {
        let err = DecimalError::InvalidPrecision {
            error: "precision must be between 1 and 75".to_string(),
        };
        let s = err.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_precision_default() {
        // Default precision should be 1 (minimum valid)
        let p = Precision::default();
        assert!(p.value() >= 1);
    }
}
