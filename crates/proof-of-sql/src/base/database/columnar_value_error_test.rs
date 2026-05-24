use crate::base::database::ColumnarValueError;

#[test]
fn columnar_value_error_display_messages() {
    let err = ColumnarValueError::ColumnLengthMismatch {
        columnar_value_length: 10,
        attempt_to_convert_length: 5,
    };
    assert!(format!("{}").contains("10") && format!("{}").contains("5"));
}

#[test]
fn columnar_value_error_debug_formatting() {
    let err = ColumnarValueError::ColumnLengthMismatch {
        columnar_value_length: 8,
        attempt_to_convert_length: 3,
    };
    assert!(format!("{:?}").contains("ColumnLengthMismatch"));
}

#[test]
fn columnar_value_error_equality() {
    let err1 = ColumnarValueError::ColumnLengthMismatch {
        columnar_value_length: 5,
        attempt_to_convert_length: 5,
    };
    let err2 = ColumnarValueError::ColumnLengthMismatch {
        columnar_value_length: 5,
        attempt_to_convert_length: 5,
    };
    let err3 = ColumnarValueError::ColumnLengthMismatch {
        columnar_value_length: 10,
        attempt_to_convert_length: 5,
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
