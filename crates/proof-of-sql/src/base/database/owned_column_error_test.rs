//! Tests for owned_column_error.rs
use crate::base::database::{ColumnType, OwnedColumnError};

#[test]
fn type_cast_error_display() {
    let err = OwnedColumnError::TypeCastError {
        from_type: ColumnType::Int,
        to_type: ColumnType::Boolean,
    };
    let msg = format!("{err}");
    assert!(msg.contains("Int"));
    assert!(msg.contains("Boolean"));
}

#[test]
fn type_cast_error_equality() {
    let err1 = OwnedColumnError::TypeCastError {
        from_type: ColumnType::BigInt,
        to_type: ColumnType::Int,
    };
    let err2 = OwnedColumnError::TypeCastError {
        from_type: ColumnType::BigInt,
        to_type: ColumnType::Int,
    };
    assert_eq!(err1, err2);
}

#[test]
fn scalar_conversion_error() {
    let err = OwnedColumnError::ScalarConversionError {
        error: "invalid scalar".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("invalid scalar"));
}

#[test]
fn unsupported_error() {
    let err = OwnedColumnError::Unsupported {
        error: "feature not available".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("feature not available"));
}

#[test]
fn owned_column_result_ok() {
    let result: OwnedColumnResult<i32> = Ok(42);
    assert!(result.is_ok());
}

#[test]
fn owned_column_result_err() {
    let result: OwnedColumnResult<i32> = Err(OwnedColumnError::Unsupported {
        error: "test".to_string(),
    });
    assert!(result.is_err());
}

use crate::base::database::OwnedColumnResult;
