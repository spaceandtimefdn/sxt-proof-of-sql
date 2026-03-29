#[cfg(test)]
mod tests {
    use crate::base::math::decimal::{DecimalError, Precision};

    // -----------------------------------------------------------------------
    // Precision
    // -----------------------------------------------------------------------

    #[test]
    fn test_precision_new_valid_values() {
        assert!(Precision::new(1).is_ok());
        assert!(Precision::new(38).is_ok());
    }

    #[test]
    fn test_precision_new_zero_is_invalid() {
        assert!(matches!(
            Precision::new(0),
            Err(DecimalError::InvalidPrecision { .. })
        ));
    }

    #[test]
    fn test_precision_new_exceeds_max_is_invalid() {
        // Precision > 75 (the max supported) should be rejected.
        assert!(matches!(
            Precision::new(76),
            Err(DecimalError::InvalidPrecision { .. })
        ));
    }

    #[test]
    fn test_precision_value_roundtrips() {
        let p = Precision::new(10).expect("valid precision");
        assert_eq!(p.value(), 10);
    }

    // -----------------------------------------------------------------------
    // DecimalError display / variants
    // -----------------------------------------------------------------------

    #[test]
    fn test_decimal_error_invalid_precision_contains_value() {
        let err = Precision::new(0).unwrap_err();
        let msg = format!("{err}");
        // The error message should mention the invalid precision value.
        assert!(msg.contains('0') || msg.to_lowercase().contains("precision"));
    }

    #[test]
    fn test_decimal_error_unsupported_operation_variant() {
        let err = DecimalError::UnsupportedOperation {
            error: "test op".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("test op"));
    }
}
