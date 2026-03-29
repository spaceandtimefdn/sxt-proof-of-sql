/// Tests for Decimal / Precision / Scale helpers
#[cfg(test)]
mod tests {
    use crate::base::math::decimal::{
        DecimalError, Precision, MAX_SUPPORTED_PRECISION,
    };

    // -----------------------------------------------------------------------
    // Precision
    // -----------------------------------------------------------------------

    #[test]
    fn test_precision_new_valid_range() {
        for p in 1u8..=MAX_SUPPORTED_PRECISION {
            let prec = Precision::new(p);
            assert!(prec.is_ok(), "Precision::new({p}) should succeed");
            assert_eq!(prec.unwrap().value(), p);
        }
    }

    #[test]
    fn test_precision_new_zero_fails() {
        assert!(Precision::new(0).is_err());
    }

    #[test]
    fn test_precision_new_exceeds_max_fails() {
        let too_large = MAX_SUPPORTED_PRECISION + 1;
        assert!(Precision::new(too_large).is_err());
    }

    #[test]
    fn test_precision_display() {
        let p = Precision::new(10).unwrap();
        assert!(format!("{p}").contains("10"));
    }

    #[test]
    fn test_precision_debug() {
        let p = Precision::new(5).unwrap();
        let dbg = format!("{p:?}");
        assert!(!dbg.is_empty());
    }

    #[test]
    fn test_precision_equality() {
        let a = Precision::new(7).unwrap();
        let b = Precision::new(7).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_precision_inequality() {
        let a = Precision::new(5).unwrap();
        let b = Precision::new(6).unwrap();
        assert_ne!(a, b);
    }

    // -----------------------------------------------------------------------
    // DecimalError variants
    // -----------------------------------------------------------------------

    #[test]
    fn test_decimal_error_invalid_precision_is_error() {
        let e = DecimalError::InvalidPrecision {
            error: "precision out of range".to_string(),
        };
        let msg = format!("{e}");
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_decimal_error_rounding_is_error() {
        let e = DecimalError::RoundingError {
            error: "rounding required".to_string(),
        };
        let msg = format!("{e}");
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_decimal_error_unsupported_operation() {
        let e = DecimalError::UnsupportedOperation {
            error: "operation not allowed".to_string(),
        };
        let msg = format!("{e}");
        assert!(!msg.is_empty());
    }

    // -----------------------------------------------------------------------
    // MAX_SUPPORTED_PRECISION constant
    // -----------------------------------------------------------------------

    #[test]
    fn test_max_supported_precision_is_reasonable() {
        // The crate targets 75-decimal-digit precision for 256-bit numbers.
        assert!(MAX_SUPPORTED_PRECISION >= 38);
        assert!(MAX_SUPPORTED_PRECISION <= 75);
    }
}
