use super::ScalarConversionError;

#[test]
fn scalar_conversion_error_display_messages() {
    let err = ScalarConversionError::Overflow {
        error: "value too large".to_string(),
    };
    assert_eq!(format!("{}"), "Overflow error: value too large");
    
    let err2 = ScalarConversionError::Overflow {
        error: "overflow in multiplication".to_string(),
    };
    assert_eq!(format!("{}"), "Overflow error: overflow in multiplication");
}

#[test]
fn scalar_conversion_error_debug_formatting() {
    let err = ScalarConversionError::Overflow {
        error: "test".to_string(),
    };
    assert_eq!(format!("{:?}"), "Overflow { error: "test" }");
}

#[test]
fn scalar_conversion_error_equality() {
    let err1 = ScalarConversionError::Overflow {
        error: "same".to_string(),
    };
    let err2 = ScalarConversionError::Overflow {
        error: "same".to_string(),
    };
    let err3 = ScalarConversionError::Overflow {
        error: "different".to_string(),
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
