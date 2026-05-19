use super::{
    AddOp, ArithmeticOp, ColumnOperationError, DivOp, MulOp, OwnedColumn, SubOp,
};
use crate::base::scalar::test_scalar::TestScalar;

// ============================================================================
// AddOp tests
// ============================================================================

#[test]
fn add_op_uint8_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8, 2, 10]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![3u8, 4, 5]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Uint8(v) => assert_eq!(v, vec![4u8, 6, 15]),
        _ => panic!("Expected Uint8 column"),
    }
}

#[test]
fn add_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, -2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![4i32, 5, -1]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert_eq!(v, vec![5i32, 3, 2]),
        _ => panic!("Expected Int column"),
    }
}

#[test]
fn add_op_bigint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10i64, 20, 30]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::BigInt(v) => assert_eq!(v, vec![11i64, 22, 33]),
        _ => panic!("Expected BigInt column"),
    }
}

#[test]
fn add_op_mixed_int_bigint() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10i64, 20, 30]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::BigInt(v) => assert_eq!(v, vec![11i64, 22, 33]),
        _ => panic!("Expected BigInt column"),
    }
}

#[test]
fn add_op_smallint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1i16, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![4i16, 5, 6]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::SmallInt(v) => assert_eq!(v, vec![5i16, 7, 9]),
        _ => panic!("Expected SmallInt column"),
    }
}

#[test]
fn add_op_int128_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1i128, 2, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![10i128, 20, 30]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int128(v) => assert_eq!(v, vec![11i128, 22, 33]),
        _ => panic!("Expected Int128 column"),
    }
}

// ============================================================================
// SubOp tests
// ============================================================================

#[test]
fn sub_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![5i32, 10, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2i32, 3, 3]);
    let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert_eq!(v, vec![3i32, 7, 0]),
        _ => panic!("Expected Int column"),
    }
}

#[test]
fn sub_op_bigint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10i64, 20, 30]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 2, 3]);
    let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::BigInt(v) => assert_eq!(v, vec![9i64, 18, 27]),
        _ => panic!("Expected BigInt column"),
    }
}

#[test]
fn sub_op_negative_result() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![5i32, 10]);
    let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert_eq!(v, vec![-4i32, -8]),
        _ => panic!("Expected Int column"),
    }
}

// ============================================================================
// MulOp tests
// ============================================================================

#[test]
fn mul_op_uint8_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![2u8, 3, 4]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![5u8, 6, 7]);
    let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Uint8(v) => assert_eq!(v, vec![10u8, 18, 28]),
        _ => panic!("Expected Uint8 column"),
    }
}

#[test]
fn mul_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2i32, -3, 0]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![3i32, 4, 5]);
    let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert_eq!(v, vec![6i32, -12, 0]),
        _ => panic!("Expected Int column"),
    }
}

// ============================================================================
// DivOp tests
// ============================================================================

#[test]
fn div_op_int_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10i32, 20, 9]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2i32, 4, 3]);
    let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert_eq!(v, vec![5i32, 5, 3]),
        _ => panic!("Expected Int column"),
    }
}

#[test]
fn div_op_bigint_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10i64, 20, 30]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![2i64, 4, 5]);
    let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::BigInt(v) => assert_eq!(v, vec![5i64, 5, 6]),
        _ => panic!("Expected BigInt column"),
    }
}

// ============================================================================
// Error cases
// ============================================================================

#[test]
fn arithmetic_different_length_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32, 2, 3]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::DifferentColumnLength { .. })));
}

#[test]
fn arithmetic_signed_casting_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1i8]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::SignedCastingError { .. })));
}

#[test]
fn arithmetic_incompatible_types_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1i32]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs);
    assert!(matches!(result, Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })));
}

#[test]
fn div_by_zero_returns_error() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10i32]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![0i32]);
    let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs);
    assert!(result.is_err());
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn add_op_empty_columns() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![]);
    let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert!(v.is_empty()),
        _ => panic!("Expected Int column"),
    }
}

#[test]
fn mul_op_mixed_smallint_int() {
    let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![2i16, 3]);
    let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![5i32, 10]);
    let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
    match result {
        OwnedColumn::Int(v) => assert_eq!(v, vec![10i32, 30]),
        _ => panic!("Expected Int column"),
    }
}
