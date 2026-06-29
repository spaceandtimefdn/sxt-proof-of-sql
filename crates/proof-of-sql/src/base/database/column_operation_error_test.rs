use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::database::ColumnType;

#[test]
fn column_operation_error_display_messages() {
    // DifferentColumnLength
    let err = ColumnOperationError::DifferentColumnLength {
        len_a: 5,
        len_b: 10,
    };
    assert_eq!(format!("{}"), "Columns have different lengths: 5 != 10");
    
    // BinaryOperationInvalidColumnType
    let err = ColumnOperationError::BinaryOperationInvalidColumnType {
        operator: "Add".to_string(),
        left_type: ColumnType::Boolean,
        right_type: ColumnType::Int,
    };
    assert_eq!(format!("{}"), "BinaryOperationInvalidColumnType { left_type: Boolean, operator: "Add", right_type: Int }" as str, format!("{}"));
    
    // DivisionByZero
    let err = ColumnOperationError::DivisionByZero;
    assert_eq!(format!("{}"), "Division by zero");
    
    // IntegerOverflow
    let err = ColumnOperationError::IntegerOverflow {
        error: "overflow in addition".to_string(),
    };
    assert_eq!(format!("{}"), "Overflow in integer operation: overflow in addition");
}

#[test]
fn column_operation_error_debug_formatting() {
    let err = ColumnOperationError::DivisionByZero;
    assert!(format!("{:?}").contains("DivisionByZero"));
    
    let err2 = ColumnOperationError::IndexOutOfBounds {
        index: 100,
        len: 50,
    };
    assert!(format!("{:?}").contains("IndexOutOfBounds"));
}

#[test]
fn column_operation_error_equality() {
    let err1 = ColumnOperationError::DivisionByZero;
    let err2 = ColumnOperationError::DivisionByZero;
    let err3 = ColumnOperationError::IntegerOverflow {
        error: "test".to_string(),
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn column_operation_result_error_propagation() -> ColumnOperationResult<i32> {
    Err(ColumnOperationError::DivisionByZero)?;
    Ok(42)
}

#[test]
fn column_operation_result_success() -> ColumnOperationResult<i32> {
    Ok(42)
}
