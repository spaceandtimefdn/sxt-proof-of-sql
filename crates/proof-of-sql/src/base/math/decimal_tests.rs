#[cfg(test)]
mod precision_tests {
    use crate::base::math::decimal::{DecimalError, Precision};
    use serde_json;

    #[test]
    fn we_can_construct_valid_precision() {
        let precision = Precision::new(1).unwrap();

        assert_eq!(precision.value(), 1);
    }

    #[test]
    fn we_cannot_construct_zero_precision() {
        assert_eq!(
            Precision::new(0),
            Err(DecimalError::InvalidPrecision {
                error: "0".to_string()
            })
        );
    }

    #[test]
    fn we_cannot_construct_precision_from_oversized_u64() {
        assert_eq!(
            Precision::try_from(u64::MAX),
            Err(DecimalError::InvalidPrecision {
                error: u64::MAX.to_string()
            })
        );
    }

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
}
