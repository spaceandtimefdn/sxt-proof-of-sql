use super::{try_add_column_types, try_multiply_column_types};
use crate::base::database::ColumnType;

#[test]
fn test_add_bigint_bigint() {
    let result = try_add_column_types(ColumnType::BigInt, ColumnType::BigInt);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ColumnType::BigInt);
}

#[test]
fn test_add_int128_int128() {
    let result = try_add_column_types(ColumnType::Int128, ColumnType::Int128);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ColumnType::Int128);
}

#[test]
fn test_add_bigint_int128_upcast() {
    // BigInt + Int128 should upcast to Int128
    let result = try_add_column_types(ColumnType::BigInt, ColumnType::Int128);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ColumnType::Int128);
}

#[test]
fn test_add_incompatible_types_returns_error() {
    // Adding a boolean to a numeric should fail
    let result = try_add_column_types(ColumnType::Boolean, ColumnType::BigInt);
    assert!(result.is_err());
}

#[test]
fn test_multiply_bigint_bigint() {
    let result = try_multiply_column_types(ColumnType::BigInt, ColumnType::BigInt);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ColumnType::BigInt);
}

#[test]
fn test_multiply_int128_int128() {
    let result = try_multiply_column_types(ColumnType::Int128, ColumnType::Int128);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ColumnType::Int128);
}

#[test]
fn test_multiply_bigint_int128() {
    let result = try_multiply_column_types(ColumnType::BigInt, ColumnType::Int128);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ColumnType::Int128);
}

#[test]
fn test_multiply_incompatible_types_returns_error() {
    let result = try_multiply_column_types(ColumnType::Boolean, ColumnType::BigInt);
    assert!(result.is_err());
}

#[test]
fn test_add_varchar_varchar_returns_error() {
    // String concatenation is not a supported column type operation here
    let result = try_add_column_types(ColumnType::VarChar, ColumnType::VarChar);
    assert!(result.is_err());
}

#[test]
fn test_multiply_varchar_returns_error() {
    let result = try_multiply_column_types(ColumnType::VarChar, ColumnType::VarChar);
    assert!(result.is_err());
}
