use super::ArrowArrayToColumnConversionError;

#[test]
fn arrow_conversion_error_display_messages() {
    // ArrayContainsNulls
    let err = ArrowArrayToColumnConversionError::ArrayContainsNulls;
    assert_eq!(format!("{}"), "arrow array must not contain nulls");
    
    // IndexOutOfBounds
    let err = ArrowArrayToColumnConversionError::IndexOutOfBounds {
        len: 10,
        index: 100,
    };
    assert_eq!(format!("{}"), "index out of bounds: the len is 10 but the index is 100");
}

#[test]
fn arrow_conversion_error_debug_formatting() {
    let err = ArrowArrayToColumnConversionError::ArrayContainsNulls;
    assert!(format!("{:?}").contains("ArrayContainsNulls"));
    
    let err2 = ArrowArrayToColumnConversionError::IndexOutOfBounds {
        len: 5,
        index: 10,
    };
    assert!(format!("{:?}").contains("IndexOutOfBounds"));
}

#[test]
fn arrow_conversion_error_equality() {
    let err1 = ArrowArrayToColumnConversionError::ArrayContainsNulls;
    let err2 = ArrowArrayToColumnConversionError::ArrayContainsNulls;
    let err3 = ArrowArrayToColumnConversionError::IndexOutOfBounds {
        len: 10,
        index: 20,
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
