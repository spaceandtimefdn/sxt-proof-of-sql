use super::{
    can_and_or_types, can_not_type, try_add_subtract_column_types,
    try_add_subtract_column_types_with_scaling, try_cast_types, try_divide_column_types,
    try_equals_types, try_equals_types_with_scaling, try_inequality_types,
    try_inequality_types_with_scaling, try_multiply_column_types, try_neg_type,
    try_scale_cast_types, ColumnOperationError, ColumnType,
};
use crate::base::math::decimal::Precision;

// ============================================================================
// try_add_subtract_column_types tests
// ============================================================================

#[test]
fn add_subtract_int_and_bigint_returns_decimal() {
    let result = try_add_subtract_column_types(ColumnType::Int, ColumnType::BigInt);
    assert!(result.is_ok());
    let ct = result.unwrap();
    assert!(matches!(ct, ColumnType::Decimal75(_, _)));
}

#[test]
fn add_subtract_scalar_with_any_numeric_returns_scalar() {
    let result = try_add_subtract_column_types(ColumnType::Scalar, ColumnType::Int);
    assert_eq!(result.unwrap(), ColumnType::Scalar);

    let result = try_add_subtract_column_types(ColumnType::BigInt, ColumnType::Scalar);
    assert_eq!(result.unwrap(), ColumnType::Scalar);
}

#[test]
fn add_subtract_non_numeric_returns_error() {
    let result = try_add_subtract_column_types(ColumnType::Boolean, ColumnType::Int);
    assert!(result.is_err());

    let result = try_add_subtract_column_types(ColumnType::VarChar, ColumnType::BigInt);
    assert!(result.is_err());
}

#[test]
fn add_subtract_decimal_with_same_scale() {
    let precision = Precision::new(10).unwrap();
    let ct = ColumnType::Decimal75(precision, 2);
    let result = try_add_subtract_column_types(ct, ct);
    assert!(result.is_ok());
}

#[test]
fn add_subtract_decimal_with_different_scale_returns_error() {
    let p1 = Precision::new(10).unwrap();
    let p2 = Precision::new(12).unwrap();
    let lhs = ColumnType::Decimal75(p1, 2);
    let rhs = ColumnType::Decimal75(p2, 3);
    let result = try_add_subtract_column_types(lhs, rhs);
    assert!(result.is_err());
}

// ============================================================================
// try_add_subtract_column_types_with_scaling tests
// ============================================================================

#[test]
fn add_subtract_with_scaling_different_scale_succeeds() {
    let p1 = Precision::new(10).unwrap();
    let p2 = Precision::new(12).unwrap();
    let lhs = ColumnType::Decimal75(p1, 2);
    let rhs = ColumnType::Decimal75(p2, 3);
    let result = try_add_subtract_column_types_with_scaling(lhs, rhs);
    assert!(result.is_ok());
}

#[test]
fn add_subtract_with_scaling_int_and_bigint() {
    let result = try_add_subtract_column_types_with_scaling(ColumnType::Int, ColumnType::BigInt);
    assert!(result.is_ok());
}

// ============================================================================
// try_multiply_column_types tests
// ============================================================================

#[test]
fn multiply_int_and_bigint_returns_decimal() {
    let result = try_multiply_column_types(ColumnType::Int, ColumnType::BigInt);
    assert!(result.is_ok());
}

#[test]
fn multiply_scalar_with_numeric_returns_scalar() {
    let result = try_multiply_column_types(ColumnType::Scalar, ColumnType::Int);
    assert_eq!(result.unwrap(), ColumnType::Scalar);
}

#[test]
fn multiply_non_numeric_returns_error() {
    let result = try_multiply_column_types(ColumnType::Boolean, ColumnType::Int);
    assert!(result.is_err());
}

// ============================================================================
// try_divide_column_types tests
// ============================================================================

#[test]
fn divide_int_and_bigint_returns_decimal() {
    let result = try_divide_column_types(ColumnType::Int, ColumnType::BigInt);
    assert!(result.is_ok());
}

#[test]
fn divide_non_numeric_returns_error() {
    let result = try_divide_column_types(ColumnType::VarChar, ColumnType::Int);
    assert!(result.is_err());
}

// ============================================================================
// try_cast_types tests
// ============================================================================

#[test]
fn cast_same_type_succeeds() {
    assert!(try_cast_types(ColumnType::Int, ColumnType::Int).is_ok());
    assert!(try_cast_types(ColumnType::BigInt, ColumnType::BigInt).is_ok());
    assert!(try_cast_types(ColumnType::Boolean, ColumnType::Boolean).is_ok());
}

#[test]
fn cast_int_to_bigint_succeeds() {
    assert!(try_cast_types(ColumnType::Int, ColumnType::BigInt).is_ok());
}

#[test]
fn cast_int_to_smallint_returns_error() {
    assert!(try_cast_types(ColumnType::Int, ColumnType::SmallInt).is_err());
}

#[test]
fn cast_boolean_to_int_returns_error() {
    assert!(try_cast_types(ColumnType::Boolean, ColumnType::Int).is_err());
}

#[test]
fn cast_varchar_to_int_returns_error() {
    assert!(try_cast_types(ColumnType::VarChar, ColumnType::Int).is_err());
}

// ============================================================================
// try_scale_cast_types tests
// ============================================================================

#[test]
fn scale_cast_same_type_succeeds() {
    assert!(try_scale_cast_types(ColumnType::Int, ColumnType::Int).is_ok());
}

#[test]
fn scale_cast_int_to_decimal_succeeds() {
    let p = Precision::new(10).unwrap();
    assert!(try_scale_cast_types(ColumnType::Int, ColumnType::Decimal75(p, 0)).is_ok());
}

// ============================================================================
// try_equals_types tests
// ============================================================================

#[test]
fn equals_same_numeric_types_succeeds() {
    assert!(try_equals_types(ColumnType::Int, ColumnType::Int).is_ok());
    assert!(try_equals_types(ColumnType::BigInt, ColumnType::BigInt).is_ok());
    assert!(try_equals_types(ColumnType::SmallInt, ColumnType::SmallInt).is_ok());
}

#[test]
fn equals_boolean_columns_succeeds() {
    assert!(try_equals_types(ColumnType::Boolean, ColumnType::Boolean).is_ok());
}

#[test]
fn equals_varchar_columns_succeeds() {
    assert!(try_equals_types(ColumnType::VarChar, ColumnType::VarChar).is_ok());
}

#[test]
fn equals_incompatible_types_returns_error() {
    assert!(try_equals_types(ColumnType::Boolean, ColumnType::Int).is_err());
    assert!(try_equals_types(ColumnType::VarChar, ColumnType::Int).is_err());
}

// ============================================================================
// try_inequality_types tests
// ============================================================================

#[test]
fn inequality_same_numeric_types_succeeds() {
    assert!(try_inequality_types(ColumnType::Int, ColumnType::Int).is_ok());
    assert!(try_inequality_types(ColumnType::BigInt, ColumnType::BigInt).is_ok());
}

#[test]
fn inequality_varchar_returns_error() {
    assert!(try_inequality_types(ColumnType::VarChar, ColumnType::VarChar).is_err());
}

#[test]
fn inequality_boolean_returns_error() {
    assert!(try_inequality_types(ColumnType::Boolean, ColumnType::Boolean).is_err());
}

// ============================================================================
// try_neg_type tests
// ============================================================================

#[test]
fn neg_numeric_type_returns_same_type() {
    assert_eq!(try_neg_type(ColumnType::Int).unwrap(), ColumnType::Int);
    assert_eq!(try_neg_type(ColumnType::BigInt).unwrap(), ColumnType::BigInt);
    assert_eq!(try_neg_type(ColumnType::SmallInt).unwrap(), ColumnType::SmallInt);
}

#[test]
fn neg_non_numeric_returns_error() {
    assert!(try_neg_type(ColumnType::Boolean).is_err());
    assert!(try_neg_type(ColumnType::VarChar).is_err());
}

// ============================================================================
// can_and_or_types tests
// ============================================================================

#[test]
fn and_or_boolean_columns() {
    assert!(can_and_or_types(ColumnType::Boolean, ColumnType::Boolean));
}

#[test]
fn and_or_non_boolean_returns_false() {
    assert!(!can_and_or_types(ColumnType::Int, ColumnType::Int));
    assert!(!can_and_or_types(ColumnType::Boolean, ColumnType::Int));
}

// ============================================================================
// can_not_type tests
// ============================================================================

#[test]
fn not_boolean_returns_true() {
    assert!(can_not_type(ColumnType::Boolean));
}

#[test]
fn not_non_boolean_returns_false() {
    assert!(!can_not_type(ColumnType::Int));
    assert!(!can_not_type(ColumnType::VarChar));
}

// ============================================================================
// Cross-type equality and inequality with scaling
// ============================================================================

#[test]
fn equals_types_with_scaling_different_scale_succeeds() {
    let p1 = Precision::new(10).unwrap();
    let p2 = Precision::new(12).unwrap();
    let lhs = ColumnType::Decimal75(p1, 2);
    let rhs = ColumnType::Decimal75(p2, 3);
    assert!(try_equals_types_with_scaling(lhs, rhs).is_ok());
}

#[test]
fn inequality_types_with_scaling_different_scale_succeeds() {
    let p1 = Precision::new(10).unwrap();
    let p2 = Precision::new(12).unwrap();
    let lhs = ColumnType::Decimal75(p1, 2);
    let rhs = ColumnType::Decimal75(p2, 3);
    assert!(try_inequality_types_with_scaling(lhs, rhs).is_ok());
}
