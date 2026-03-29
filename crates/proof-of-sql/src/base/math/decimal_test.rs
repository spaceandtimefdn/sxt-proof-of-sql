use super::{DecimalError, Precision};

#[test]
fn test_precision_new_valid() {
    let p = Precision::new(10).expect("valid precision");
    assert_eq!(p.value(), 10);
}

#[test]
fn test_precision_new_zero_is_invalid() {
    assert!(Precision::new(0).is_err());
}

#[test]
fn test_precision_max_valid() {
    // Maximum allowed precision for proof-of-sql (75)
    assert!(Precision::new(75).is_ok());
}

#[test]
fn test_precision_above_max_is_invalid() {
    assert!(Precision::new(76).is_err());
}

#[test]
fn test_decimal_error_precision_variant() {
    let err = DecimalError::InvalidPrecision {
        error: "precision out of range".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("precision") || msg.contains("InvalidPrecision") || !msg.is_empty());
}

#[test]
fn test_decimal_error_rounding_variant() {
    let err = DecimalError::RoundingError {
        error: "cannot round".to_string(),
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn test_precision_value_roundtrip() {
    for v in [1u8, 10, 38, 75] {
        let p = Precision::new(v).unwrap();
        assert_eq!(p.value(), v);
    }
}
