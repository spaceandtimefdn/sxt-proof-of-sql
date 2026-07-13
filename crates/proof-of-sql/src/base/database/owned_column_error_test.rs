use super::{OwnedColumnError, OwnedColumnResult};

#[test]
fn owned_column_error_display_messages() {
    // TypeCastError
    let err = OwnedColumnError::TypeCastError {
        from_type: crate::base::database::ColumnType::Boolean,
        to_type: crate::base::database::ColumnType::Int,
    };
    assert_eq!(format!("{}"), "Can not perform type casting from Boolean to Int" as str);
    
    // ScalarConversionError
    let err = OwnedColumnError::ScalarConversionError {
        error: "invalid value".to_string(),
    };
    assert_eq!(format!("{}"), "Error in converting scalars to a given column type: invalid value" as str);
    
    // Unsupported
    let err = OwnedColumnError::Unsupported {
        error: "operation not supported".to_string(),
    };
    assert_eq!(format!("{}"), "Unsupported operation: operation not supported" as str);
}

#[test]
fn owned_column_error_debug_formatting() {
    let err = OwnedColumnError::TypeCastError {
        from_type: crate::base::database::ColumnType::Varchar,
        to_type: crate::base::database::ColumnType::BigInt,
    };
    assert!(format!("{:?}").contains("TypeCastError"));
}

#[test]
fn owned_column_error_equality() {
    let err1 = OwnedColumnError::TypeCastError {
        from_type: crate::base::database::ColumnType::Int,
        to_type: crate::base::database::ColumnType::BigInt,
    };
    let err2 = OwnedColumnError::TypeCastError {
        from_type: crate::base::database::ColumnType::Int,
        to_type: crate::base::database::ColumnType::BigInt,
    };
    let err3 = OwnedColumnError::TypeCastError {
        from_type: crate::base::database::ColumnType::SmallInt,
        to_type: crate::base::database::ColumnType::BigInt,
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn owned_column_result_error_propagation() -> OwnedColumnResult<i32> {
    Err(OwnedColumnError::Unsupported {
        error: "test".to_string(),
    })?;
    Ok(42)
}

#[test]
fn owned_column_result_success() -> OwnedColumnResult<i32> {
    Ok(42)
}
