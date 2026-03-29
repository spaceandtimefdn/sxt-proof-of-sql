#[cfg(test)]
mod precision_tests {
    use crate::base::math::decimal::Precision;
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
}

#[cfg(test)]
mod precision_new_tests {
    use crate::base::math::decimal::{DecimalError, Precision};

    #[test]
    fn we_can_create_precision_with_value_one() {
        let p = Precision::new(1);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().value(), 1);
    }

    #[test]
    fn we_can_create_precision_with_max_value() {
        let p = Precision::new(75);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().value(), 75);
    }

    #[test]
    fn we_can_create_precision_with_midrange_value() {
        let p = Precision::new(38);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().value(), 38);
    }

    #[test]
    fn we_cannot_create_precision_with_zero() {
        let p = Precision::new(0);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { error } if error == "0"
        ));
    }

    #[test]
    fn we_cannot_create_precision_exceeding_max() {
        let p = Precision::new(76);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { error } if error == "76"
        ));
    }

    #[test]
    fn we_cannot_create_precision_with_255() {
        let p = Precision::new(255);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { error } if error == "255"
        ));
    }

    #[test]
    fn precision_value_returns_inner_u8() {
        for v in [1u8, 10, 50, 75] {
            let p = Precision::new(v).unwrap();
            assert_eq!(p.value(), v);
        }
    }

    #[test]
    fn precision_equality_holds() {
        let p1 = Precision::new(42).unwrap();
        let p2 = Precision::new(42).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn precision_inequality_holds() {
        let p1 = Precision::new(10).unwrap();
        let p2 = Precision::new(20).unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn precision_clone_works() {
        let p = Precision::new(30).unwrap();
        let p_clone = p;
        assert_eq!(p, p_clone);
    }

    #[test]
    fn precision_debug_format_works() {
        let p = Precision::new(25).unwrap();
        let debug_str = format!("{p:?}");
        assert!(debug_str.contains("25"));
    }
}

#[cfg(test)]
mod precision_try_from_u64_tests {
    use crate::base::math::decimal::{DecimalError, Precision};

    #[test]
    fn we_can_create_precision_from_valid_u64() {
        let p = Precision::try_from(1u64);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().value(), 1);
    }

    #[test]
    fn we_can_create_precision_from_u64_max_valid() {
        let p = Precision::try_from(75u64);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().value(), 75);
    }

    #[test]
    fn we_can_create_precision_from_u64_midrange() {
        let p = Precision::try_from(50u64);
        assert!(p.is_ok());
        assert_eq!(p.unwrap().value(), 50);
    }

    #[test]
    fn we_cannot_create_precision_from_u64_zero() {
        let p = Precision::try_from(0u64);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { .. }
        ));
    }

    #[test]
    fn we_cannot_create_precision_from_u64_exceeding_max() {
        let p = Precision::try_from(76u64);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { .. }
        ));
    }

    #[test]
    fn we_cannot_create_precision_from_u64_very_large() {
        let p = Precision::try_from(u64::MAX);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { .. }
        ));
    }

    #[test]
    fn we_cannot_create_precision_from_u64_256() {
        // 256 overflows u8
        let p = Precision::try_from(256u64);
        assert!(p.is_err());
        assert!(matches!(
            p.unwrap_err(),
            DecimalError::InvalidPrecision { .. }
        ));
    }
}

#[cfg(test)]
mod decimal_error_display_tests {
    use crate::base::math::decimal::{DecimalError, IntermediateDecimalError};
    use alloc::string::{String, ToString};

    #[test]
    fn decimal_error_invalid_decimal_displays_correctly() {
        let err = DecimalError::InvalidDecimal {
            error: "bad_value".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("bad_value"));
        assert!(s.contains("Invalid decimal format or value"));
    }

    #[test]
    fn decimal_error_invalid_precision_displays_correctly() {
        let err = DecimalError::InvalidPrecision {
            error: "100".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("100"));
        assert!(s.contains("Decimal precision is not valid"));
    }

    #[test]
    fn decimal_error_invalid_scale_displays_correctly() {
        let err = DecimalError::InvalidScale {
            scale: "-999".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("-999"));
        assert!(s.contains("Decimal scale is not valid"));
    }

    #[test]
    fn decimal_error_rounding_error_displays_correctly() {
        let err = DecimalError::RoundingError {
            error: "cannot round".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("cannot round"));
        assert!(s.contains("Unsupported operation"));
    }

    #[test]
    fn decimal_error_converts_to_string_via_from() {
        let err = DecimalError::InvalidDecimal {
            error: "test_error".to_string(),
        };
        let s: String = err.into();
        assert!(s.contains("test_error"));
    }

    #[test]
    fn decimal_error_converts_to_string_via_from_invalid_precision() {
        let err = DecimalError::InvalidPrecision {
            error: "42".to_string(),
        };
        let s: String = err.into();
        assert!(!s.is_empty());
    }

    #[test]
    fn intermediate_decimal_error_out_of_range_displays() {
        let err = IntermediateDecimalError::OutOfRange;
        let s = err.to_string();
        assert!(s.contains("Value out of range for target type"));
    }

    #[test]
    fn intermediate_decimal_error_lossy_cast_displays() {
        let err = IntermediateDecimalError::LossyCast;
        let s = err.to_string();
        assert!(s.contains("Fractional part of decimal is non-zero"));
    }

    #[test]
    fn intermediate_decimal_error_conversion_failure_displays() {
        let err = IntermediateDecimalError::ConversionFailure;
        let s = err.to_string();
        assert!(s.contains("Conversion to integer failed"));
    }

    #[test]
    fn decimal_error_intermediate_conversion_error_wraps_correctly() {
        let intermediate = IntermediateDecimalError::OutOfRange;
        let err = DecimalError::IntermediateDecimalConversionError { source: intermediate };
        let s = err.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn decimal_error_equality_holds() {
        let e1 = DecimalError::InvalidScale {
            scale: "bad".to_string(),
        };
        let e2 = DecimalError::InvalidScale {
            scale: "bad".to_string(),
        };
        assert_eq!(e1, e2);
    }

    #[test]
    fn intermediate_decimal_error_equality_holds() {
        assert_eq!(
            IntermediateDecimalError::OutOfRange,
            IntermediateDecimalError::OutOfRange
        );
        assert_eq!(
            IntermediateDecimalError::LossyCast,
            IntermediateDecimalError::LossyCast
        );
        assert_eq!(
            IntermediateDecimalError::ConversionFailure,
            IntermediateDecimalError::ConversionFailure
        );
    }

    #[test]
    fn intermediate_decimal_error_inequality_holds() {
        assert_ne!(
            IntermediateDecimalError::OutOfRange,
            IntermediateDecimalError::LossyCast
        );
        assert_ne!(
            IntermediateDecimalError::LossyCast,
            IntermediateDecimalError::ConversionFailure
        );
    }
}

#[cfg(test)]
mod precision_hash_tests {
    use crate::base::math::decimal::Precision;
    use std::collections::HashSet;

    #[test]
    fn precision_can_be_used_in_hash_set() {
        let mut set = HashSet::new();
        let p1 = Precision::new(10).unwrap();
        let p2 = Precision::new(20).unwrap();
        let p3 = Precision::new(10).unwrap();
        set.insert(p1);
        set.insert(p2);
        set.insert(p3); // duplicate of p1
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn precision_hash_consistent_with_equality() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let p1 = Precision::new(42).unwrap();
        let p2 = Precision::new(42).unwrap();

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        p1.hash(&mut h1);
        p2.hash(&mut h2);

        assert_eq!(h1.finish(), h2.finish());
    }
}

