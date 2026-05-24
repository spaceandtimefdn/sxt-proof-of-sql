use crate::base::{
    database::{
        column_arithmetic_operation::{AddOp, ArithmeticOp, DivOp, MulOp, SubOp},
        ColumnOperationError, OwnedColumn,
    },
    scalar::test_scalar::TestScalar,
};

// ========== AddOp tests ==========

#[test]
fn add_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::BigInt(vec![10, 20, 30]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![11, 22, 33]));
}

#[test]
fn add_op_on_uint8() {
    let lhs = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::Uint8(vec![10u8, 20, 30]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Uint8(vec![11u8, 22, 33]));
}

#[test]
fn add_op_on_tinyint() {
    let lhs = OwnedColumn::TinyInt(vec![1i8, -2, 3]);
    let rhs = OwnedColumn::TinyInt(vec![10i8, 20, -30]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::TinyInt(vec![11i8, 18, -27]));
}

#[test]
fn add_op_on_smallint() {
    let lhs = OwnedColumn::SmallInt(vec![100i16, 200]);
    let rhs = OwnedColumn::SmallInt(vec![200i16, 300]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::SmallInt(vec![300i16, 500]));
}

#[test]
fn add_op_on_int() {
    let lhs = OwnedColumn::Int(vec![1000i32, 2000]);
    let rhs = OwnedColumn::Int(vec![2000i32, 3000]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Int(vec![3000i32, 5000]));
}

#[test]
fn add_op_on_int128() {
    let lhs = OwnedColumn::Int128(vec![100i128, 200]);
    let rhs = OwnedColumn::Int128(vec![200i128, 300]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Int128(vec![300i128, 500]));
}

#[test]
fn add_op_bigint_overflow_returns_error() {
    let lhs = OwnedColumn::BigInt(vec![i64::MAX]);
    let rhs = OwnedColumn::BigInt(vec![1]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::IntegerOverflow { .. })
    ));
}

#[test]
fn add_op_different_length_errors() {
    let lhs = OwnedColumn::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::BigInt(vec![1, 2]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::DifferentColumnLength { .. })
    ));
}

#[test]
fn add_op_uint8_tinyint_errors() {
    let lhs = OwnedColumn::Uint8(vec![1u8]);
    let rhs = OwnedColumn::TinyInt(vec![1i8]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::SignedCastingError { .. })
    ));
}

#[test]
fn add_op_incompatible_types_errors() {
    let lhs = OwnedColumn::BigInt(vec![1]);
    let rhs = OwnedColumn::Boolean(vec![true]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn add_op_empty_columns() {
    let lhs = OwnedColumn::BigInt(vec![]);
    let rhs = OwnedColumn::BigInt(vec![]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![]));
}

// ========== SubOp tests ==========

#[test]
fn sub_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![10, 20, 30]);
    let rhs = OwnedColumn::BigInt(vec![1, 2, 3]);
    let result = SubOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![9, 18, 27]));
}

#[test]
fn sub_op_on_int128() {
    let lhs = OwnedColumn::Int128(vec![100i128, 200]);
    let rhs = OwnedColumn::Int128(vec![50i128, 100]);
    let result = SubOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Int128(vec![50i128, 100]));
}

#[test]
fn sub_op_bigint_underflow_returns_error() {
    let lhs = OwnedColumn::BigInt(vec![i64::MIN]);
    let rhs = OwnedColumn::BigInt(vec![1]);
    let result = SubOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::IntegerOverflow { .. })
    ));
}

#[test]
fn sub_op_empty_columns() {
    let lhs = OwnedColumn::BigInt(vec![]);
    let rhs = OwnedColumn::BigInt(vec![]);
    let result = SubOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![]));
}

// ========== MulOp tests ==========

#[test]
fn mul_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![2, 3, 4]);
    let rhs = OwnedColumn::BigInt(vec![10, 20, 30]);
    let result = MulOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![20, 60, 120]));
}

#[test]
fn mul_op_on_int128() {
    let lhs = OwnedColumn::Int128(vec![5i128, 10]);
    let rhs = OwnedColumn::Int128(vec![20i128, 30]);
    let result = MulOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Int128(vec![100i128, 300]));
}

#[test]
fn mul_op_bigint_overflow_returns_error() {
    let lhs = OwnedColumn::BigInt(vec![i64::MAX]);
    let rhs = OwnedColumn::BigInt(vec![2]);
    let result = MulOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::IntegerOverflow { .. })
    ));
}

#[test]
fn mul_op_empty_columns() {
    let lhs = OwnedColumn::BigInt(vec![]);
    let rhs = OwnedColumn::BigInt(vec![]);
    let result = MulOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![]));
}

// ========== DivOp tests ==========

#[test]
fn div_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![10, 20, 30]);
    let rhs = OwnedColumn::BigInt(vec![2, 4, 5]);
    let result = DivOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![5, 5, 6]));
}

#[test]
fn div_op_on_int128() {
    let lhs = OwnedColumn::Int128(vec![100i128, 200]);
    let rhs = OwnedColumn::Int128(vec![10i128, 20]);
    let result = DivOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Int128(vec![10i128, 10]));
}

#[test]
fn div_op_by_zero_returns_error() {
    let lhs = OwnedColumn::BigInt(vec![10]);
    let rhs = OwnedColumn::BigInt(vec![0]);
    let result = DivOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::DivisionByZero { .. })
    ));
}

#[test]
fn div_op_empty_columns() {
    let lhs = OwnedColumn::BigInt(vec![]);
    let rhs = OwnedColumn::BigInt(vec![]);
    let result = DivOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![]));
}

// ========== Mixed type arithmetic (upcasting) ==========

#[test]
fn add_op_uint8_with_smallint() {
    let lhs = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::SmallInt(vec![100i16, 200, 300]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::SmallInt(vec![101i16, 202, 303]));
}

#[test]
fn add_op_smallint_with_bigint() {
    let lhs = OwnedColumn::SmallInt(vec![100i16, 200]);
    let rhs = OwnedColumn::BigInt(vec![1000i64, 2000]);
    let result = AddOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![1100i64, 2200]));
}

#[test]
fn sub_op_uint8_with_int() {
    let lhs = OwnedColumn::Uint8(vec![10u8, 20]);
    let rhs = OwnedColumn::Int(vec![5i32, 10]);
    let result = SubOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Int(vec![5i32, 10]));
}

#[test]
fn mul_op_tinyint_with_bigint() {
    let lhs = OwnedColumn::TinyInt(vec![5i8, 10]);
    let rhs = OwnedColumn::BigInt(vec![100i64, 200]);
    let result = MulOp::owned_column_element_wise_arithmetic::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::BigInt(vec![500i64, 2000]));
}
