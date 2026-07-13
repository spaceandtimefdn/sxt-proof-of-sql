#[cfg(test)]
mod precision_tests {
    use crate::base::math::decimal::{DecimalError, Precision};
    use alloc::string::{String, ToString};
    use serde_json;

    #[test]
    fn we_can_deserialize_valid_precision() {
        let json = "50"; // A valid value within the range
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_ok());
        assert_eq!(precision.unwrap().value(), 50);
    }

    #[test]
    fn we_can_deserialize_valid_precision_inclusive() {
        let json = "75"; // A valid value within the range
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_ok());
        assert_eq!(precision.unwrap().value(), 75);
    }

    #[test]
    fn we_cannot_deserialize_invalid_precision() {
        let json = "76"; // An invalid value outside the range
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_err());
    }

    // Test deserialization of a non-numeric value
    #[test]
    fn we_cannot_deserialize_non_numeric_precision() {
        let json = "\"not a number\"";
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_err());
    }

    #[test]
    fn we_can_try_convert_u64_to_precision() {
        assert_eq!(Precision::try_from(75_u64).unwrap().value(), 75);
        assert!(matches!(
            Precision::try_from(76_u64),
            Err(DecimalError::InvalidPrecision { .. })
        ));
        assert!(matches!(
            Precision::try_from(u64::MAX),
            Err(DecimalError::InvalidPrecision { .. })
        ));
    }

    #[test]
    fn we_can_convert_decimal_errors_to_strings() {
        assert_eq!(
            String::from(DecimalError::InvalidPrecision {
                error: 0_u8.to_string()
            }),
            "Decimal precision is not valid: 0"
        );
    }
}
