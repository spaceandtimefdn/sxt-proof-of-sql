//! Tests for column_operation_error.rs and table_operation_error.rs
//! These error types are used throughout the codebase but had no dedicated tests.
use crate::base::database::{
    ColumnField, ColumnOperationError, ColumnType, TableOperationError,
};
use crate::base::math::decimal::DecimalError;
use sqlparser::ast::Ident;

// === ColumnOperationError tests ===

#[test]
fn different_column_length_error() {
    let err = ColumnOperationError::DifferentColumnLength {
        len_a: 3,
        len_b: 5,
    };
    let msg = format!("{err}");
    assert!(msg.contains("3"));
    assert!(msg.contains("5"));
}

#[test]
fn different_column_length_equality() {
    let err1 = ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 };
    let err2 = ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 };
    assert_eq!(err1, err2);
}

#[test]
fn binary_operation_invalid_column_type_error() {
    let err = ColumnOperationError::BinaryOperationInvalidColumnType {
        operator: "add".to_string(),
        left_type: ColumnType::Boolean,
        right_type: ColumnType::BigInt,
    };
    let msg = format!("{err}");
    assert!(msg.contains("add"));
    assert!(msg.contains("Boolean"));
    assert!(msg.contains("BigInt"));
}

#[test]
fn unary_operation_invalid_column_type_error() {
    let err = ColumnOperationError::UnaryOperationInvalidColumnType {
        operator: "not".to_string(),
        operand_type: ColumnType::Int,
    };
    let msg = format!("{err}");
    assert!(msg.contains("not"));
    assert!(msg.contains("Int"));
}

#[test]
fn integer_overflow_error() {
    let err = ColumnOperationError::IntegerOverflow {
        error: "i64 overflow".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("i64 overflow"));
}

#[test]
fn division_by_zero_error() {
    let err = ColumnOperationError::DivisionByZero;
    let msg = format!("{err}");
    assert!(msg.to_lowercase().contains("zero"));
}

#[test]
fn index_out_of_bounds_error() {
    let err = ColumnOperationError::IndexOutOfBounds { index: 5, len: 3 };
    let msg = format!("{err}");
    assert!(msg.contains("5"));
    assert!(msg.contains("3"));
}

#[test]
fn union_different_types_error() {
    let err = ColumnOperationError::UnionDifferentTypes {
        correct_type: ColumnType::BigInt,
        actual_type: ColumnType::Int,
    };
    let msg = format!("{err}");
    assert!(msg.contains("BigInt"));
    assert!(msg.contains("Int"));
}

#[test]
fn signed_casting_error() {
    let err = ColumnOperationError::SignedCastingError {
        left_type: ColumnType::BigInt,
        right_type: ColumnType::Uint8,
    };
    let msg = format!("{err}");
    assert!(msg.contains("BigInt"));
    assert!(msg.contains("Uint8"));
}

#[test]
fn casting_error() {
    let err = ColumnOperationError::CastingError {
        left_type: ColumnType::Int128,
        right_type: ColumnType::SmallInt,
    };
    let msg = format!("{err}");
    assert!(msg.contains("Int128"));
    assert!(msg.contains("SmallInt"));
}

#[test]
fn scale_casting_error() {
    let err = ColumnOperationError::ScaleCastingError {
        left_type: ColumnType::BigInt,
        right_type: ColumnType::Int,
    };
    assert!(matches!(err, ColumnOperationError::ScaleCastingError { .. }));
}

// === TableOperationError tests ===

#[test]
fn union_incompatible_schemas_error() {
    let schema1 = vec![ColumnField::new(Ident::new("a"), ColumnType::BigInt)];
    let schema2 = vec![ColumnField::new(Ident::new("a"), ColumnType::Int)];
    let err = TableOperationError::UnionIncompatibleSchemas {
        correct_schema: schema1.clone(),
        actual_schema: schema2.clone(),
    };
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn union_not_enough_tables_error() {
    let err = TableOperationError::UnionNotEnoughTables;
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

#[test]
fn join_with_different_number_of_columns_error() {
    let err = TableOperationError::JoinWithDifferentNumberOfColumns {
        left_num_columns: 3,
        right_num_columns: 5,
    };
    let msg = format!("{err}");
    assert!(msg.contains("3"));
    assert!(msg.contains("5"));
}

#[test]
fn join_incompatible_types_error() {
    let err = TableOperationError::JoinIncompatibleTypes {
        left_type: ColumnType::Boolean,
        right_type: ColumnType::VarChar,
    };
    let msg = format!("{err}");
    assert!(msg.contains("Boolean"));
    assert!(msg.contains("VarChar"));
}

#[test]
fn column_does_not_exist_error() {
    let err = TableOperationError::ColumnDoesNotExist {
        column_ident: Ident::new("missing_col"),
    };
    let msg = format!("{err}");
    assert!(msg.contains("missing_col"));
}

#[test]
fn duplicate_column_error() {
    let err = TableOperationError::DuplicateColumn;
    assert!(matches!(err, TableOperationError::DuplicateColumn));
}

#[test]
fn column_index_out_of_bounds_error() {
    let err = TableOperationError::ColumnIndexOutOfBounds { column_index: 99 };
    let msg = format!("{err}");
    assert!(msg.contains("99"));
}

#[test]
fn table_operation_error_from_column_operation_error() {
    let col_err = ColumnOperationError::DivisionByZero;
    let table_err = TableOperationError::ColumnOperationError { source: col_err };
    assert!(matches!(table_err, TableOperationError::ColumnOperationError { .. }));
}
