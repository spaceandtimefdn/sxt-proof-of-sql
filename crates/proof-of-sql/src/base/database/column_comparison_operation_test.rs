use super::{
    ColumnOperationError, ColumnType, ComparisonOp, EqualOp, GreaterThanOp, LessThanOp,
    OwnedColumn,
};
use crate::base::{database::owned_table_utility::*, scalar::test_scalar::TestScalar};

// ============================================================================
// EqualOp tests
// ============================================================================

#[test]
fn equal_op_uint8_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3, 4]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 3, 3, 5]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_smallint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1i16, -2, 3, 0]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1i16, -2, 0, 0]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 3, 3]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_bigint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 2, 4]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, true, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_int128_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1i128, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1i128, 3, 3]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_boolean_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, false, true]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, true, false]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_varchar_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["a".to_string(), "x".to_string(), "c".to_string()]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_mixed_uint8_smallint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1i16, 3, 3]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_mixed_int_bigint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 3, 3]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

// ============================================================================
// GreaterThanOp tests
// ============================================================================

#[test]
fn greater_than_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![3i32, 1, 2]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 1, 5]);
    let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn greater_than_op_bigint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![5i64, 3, 1]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![3i64, 3, 5]);
    let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn greater_than_op_uint8_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![5u8, 3, 1]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![3u8, 3, 5]);
    let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn greater_than_op_varchar_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["b".to_string()]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["a".to_string()]);
    let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// LessThanOp tests
// ============================================================================

#[test]
fn less_than_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2, 5]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![3i32, 2, 3]);
    let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn less_than_op_smallint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1i16, 5, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![3i16, 2, 3]);
    let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn less_than_op_varchar_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["a".to_string()]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["b".to_string()]);
    let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn comparison_different_length_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2, 3]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::DifferentColumnLength { .. })));
}

#[test]
fn comparison_signed_casting_error_uint8_tinyint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1i8]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::SignedCastingError { .. })));
}

#[test]
fn comparison_signed_casting_error_tinyint_uint8() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1i8]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::SignedCastingError { .. })));
}

#[test]
fn comparison_incompatible_column_types_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })));
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn equal_op_empty_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert!(v.is_empty()),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_mixed_tinyint_smallint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1i8, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1i16, 3, 3]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn greater_than_mixed_smallint_bigint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![5i16, 1, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![3i64, 1, 5]);
    let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn less_than_mixed_int_int128() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 5, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![3i128, 2, 3]);
    let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, false]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn equal_op_negative_values() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![-1i16, -2, 0]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![-1i16, 2, 0]);
    let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![true, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}

#[test]
fn greater_than_with_negative_smallint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![-1i16, 0, 1]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![0i16, 0, -1]);
    let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Boolean(v) => assert_eq!(v, vec![false, false, true]),
        _ => panic!("Expected Boolean column"),
    }
}
