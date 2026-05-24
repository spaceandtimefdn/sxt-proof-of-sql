use super::{TableOperationError, TableOperationResult};
use crate::base::database::{ColumnField, ColumnType};
use sqlparser::ast::Ident;

#[test]
fn table_operation_error_display_messages() {
    // UnionNotEnoughTables
    let err = TableOperationError::UnionNotEnoughTables;
    assert_eq!(format!("{}"), "Cannot union fewer than 2 tables");
    
    // DuplicateColumn
    let err = TableOperationError::DuplicateColumn;
    assert_eq!(format!("{}"), "Some column is duplicated in table");
}

#[test]
fn table_operation_error_debug_formatting() {
    let err = TableOperationError::UnionNotEnoughTables;
    assert!(format!("{:?}").contains("UnionNotEnoughTables"));
    
    let err2 = TableOperationError::DuplicateColumn;
    assert!(format!("{:?}").contains("DuplicateColumn"));
}

#[test]
fn table_operation_error_equality() {
    let err1 = TableOperationError::UnionNotEnoughTables;
    let err2 = TableOperationError::UnionNotEnoughTables;
    let err3 = TableOperationError::DuplicateColumn;
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn table_operation_result_error_propagation() -> TableOperationResult<i32> {
    Err(TableOperationError::DuplicateColumn)?;
    Ok(42)
}

#[test]
fn table_operation_result_success() -> TableOperationResult<i32> {
    Ok(42)
}
