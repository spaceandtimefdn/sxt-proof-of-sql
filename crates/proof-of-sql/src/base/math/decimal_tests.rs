#[cfg(test)]
mod precision_tests {
    use crate::base::math::decimal::{DecimalError, Precision};
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
    fn we_can_convert_decimal_errors_to_strings() {
        let error = DecimalError::InvalidScale {
            scale: "76".to_string(),
        };

        let message: String = error.into();

        assert_eq!(message, "Decimal scale is not valid: 76");
    }

    #[test]
    fn we_can_create_precision_from_u64() {
        let precision = Precision::try_from(75_u64).unwrap();

        assert_eq!(precision.value(), 75);
    }

    #[test]
    fn we_cannot_create_precision_from_large_u64() {
        let precision = Precision::try_from(u64::MAX);

        assert_eq!(
            precision,
            Err(DecimalError::InvalidPrecision {
                error: u64::MAX.to_string(),
            })
        );
    }
}
