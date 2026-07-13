use super::{DecimalError, IntermediateDecimalError};

#[test]
fn decimal_error_display_messages() {
    // InvalidDecimal
    let err = DecimalError::InvalidDecimal {
        error: "invalid format".to_string(),
    };
    assert_eq!(format!("{}"), "Invalid decimal format or value: invalid format");
    
    // InvalidPrecision
    let err = DecimalError::InvalidPrecision {
        error: "precision too high".to_string(),
    };
    assert_eq!(format!("{}"), "Decimal precision is not valid: precision too high");
    
    // InvalidScale
    let err = DecimalError::InvalidScale {
        scale: "-1".to_string(),
    };
    assert_eq!(format!("{}"), "Decimal scale is not valid: -1");
    
    // RoundingError
    let err = DecimalError::RoundingError {
        error: "loss of precision".to_string(),
    };
    assert_eq!(format!("{}"), "Unsupported operation: cannot round decimal: loss of precision");
}

#[test]
fn decimal_error_debug_formatting() {
    let err = DecimalError::OutOfRange;
    assert!(format!("{:?}").contains("OutOfRange"));
}

#[test]
fn decimal_error_equality() {
    let err1 = DecimalError::InvalidDecimal {
        error: "same".to_string(),
    };
    let err2 = DecimalError::InvalidDecimal {
        error: "same".to_string(),
    };
    let err3 = DecimalError::InvalidDecimal {
        error: "different".to_string(),
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn intermediate_decimal_error_display_messages() {
    // OutOfRange
    let err = IntermediateDecimalError::OutOfRange;
    assert_eq!(format!("{}"), "Value out of range for target type");
    
    // LossyCast
    let err = IntermediateDecimalError::LossyCast;
    assert_eq!(format!("{}"), "Fractional part of decimal is non-zero");
    
    // ConversionFailure
    let err = IntermediateDecimalError::ConversionFailure;
    assert_eq!(format!("{}"), "Conversion to integer failed");
}

#[test]
fn intermediate_decimal_error_debug_formatting() {
    let err = IntermediateDecimalError::OutOfRange;
    assert!(format!("{:?}").contains("OutOfRange"));
}

#[test]
fn intermediate_decimal_error_equality() {
    let err1 = IntermediateDecimalError::OutOfRange;
    let err2 = IntermediateDecimalError::OutOfRange;
    let err3 = IntermediateDecimalError::LossyCast;
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
