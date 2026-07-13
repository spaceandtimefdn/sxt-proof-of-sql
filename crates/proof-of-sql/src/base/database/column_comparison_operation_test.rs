use crate::base::{
    database::{
        column_comparison_operation::{ComparisonOp, EqualOp, GreaterThanOp, LessThanOp},
        ColumnOperationError, OwnedColumn,
    },
    scalar::test_scalar::TestScalar,
};

// ========== EqualOp tests ==========

#[test]
fn equal_op_on_booleans() {
    let lhs = OwnedColumn::Boolean(vec![true, false, true]);
    let rhs = OwnedColumn::Boolean(vec![true, true, false]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn equal_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::BigInt(vec![1, 5, 3]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
}

#[test]
fn equal_op_on_varchar() {
    let lhs = OwnedColumn::VarChar(vec!["hello".to_string(), "world".to_string()]);
    let rhs = OwnedColumn::VarChar(vec!["hello".to_string(), "foo".to_string()]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false]));
}

#[test]
fn equal_op_on_uint8() {
    let lhs = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::Uint8(vec![1u8, 5, 3]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
}

#[test]
fn equal_op_on_tinyint() {
    let lhs = OwnedColumn::TinyInt(vec![1i8, -2, 3]);
    let rhs = OwnedColumn::TinyInt(vec![1i8, 2, 3]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
}

#[test]
fn equal_op_on_smallint() {
    let lhs = OwnedColumn::SmallInt(vec![100i16, 200, 300]);
    let rhs = OwnedColumn::SmallInt(vec![100i16, 999, 300]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
}

#[test]
fn equal_op_on_int() {
    let lhs = OwnedColumn::Int(vec![100000i32, 200000, 300000]);
    let rhs = OwnedColumn::Int(vec![100000i32, 999999, 300000]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
}

#[test]
fn equal_op_on_int128() {
    let lhs = OwnedColumn::Int128(vec![1i128, 2, 3]);
    let rhs = OwnedColumn::Int128(vec![1i128, 5, 3]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
}

#[test]
fn equal_op_different_length_errors() {
    let lhs = OwnedColumn::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::BigInt(vec![1, 2]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::DifferentColumnLength { .. })
    ));
}

#[test]
fn equal_op_incompatible_types_errors() {
    let lhs = OwnedColumn::BigInt(vec![1]);
    let rhs = OwnedColumn::Boolean(vec![true]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn equal_op_uint8_tinyint_errors() {
    let lhs = OwnedColumn::Uint8(vec![1u8]);
    let rhs = OwnedColumn::TinyInt(vec![1i8]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::SignedCastingError { .. })
    ));
}

// ========== GreaterThanOp tests ==========

#[test]
fn greater_than_op_on_booleans() {
    let lhs = OwnedColumn::Boolean(vec![true, false, true]);
    let rhs = OwnedColumn::Boolean(vec![false, true, true]);
    let result =
        GreaterThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn greater_than_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![5, 1, 3]);
    let rhs = OwnedColumn::BigInt(vec![3, 2, 3]);
    let result =
        GreaterThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn greater_than_op_varchar_errors() {
    let lhs = OwnedColumn::VarChar(vec!["a".to_string()]);
    let rhs = OwnedColumn::VarChar(vec!["b".to_string()]);
    let result = GreaterThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn greater_than_op_on_int128() {
    let lhs = OwnedColumn::Int128(vec![100i128, 50, 75]);
    let rhs = OwnedColumn::Int128(vec![99i128, 50, 100]);
    let result =
        GreaterThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

// ========== LessThanOp tests ==========

#[test]
fn less_than_op_on_booleans() {
    let lhs = OwnedColumn::Boolean(vec![true, false, true]);
    let rhs = OwnedColumn::Boolean(vec![false, true, true]);
    let result =
        LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![false, true, false]));
}

#[test]
fn less_than_op_on_bigint() {
    let lhs = OwnedColumn::BigInt(vec![1, 5, 3]);
    let rhs = OwnedColumn::BigInt(vec![3, 2, 3]);
    let result =
        LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn less_than_op_varchar_errors() {
    let lhs = OwnedColumn::VarChar(vec!["a".to_string()]);
    let rhs = OwnedColumn::VarChar(vec!["b".to_string()]);
    let result = LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs);
    assert!(matches!(
        result,
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn less_than_op_on_tinyint() {
    let lhs = OwnedColumn::TinyInt(vec![1i8, 5, 3]);
    let rhs = OwnedColumn::TinyInt(vec![3i8, 2, 3]);
    let result =
        LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn less_than_op_on_smallint() {
    let lhs = OwnedColumn::SmallInt(vec![10i16, 50, 30]);
    let rhs = OwnedColumn::SmallInt(vec![30i16, 20, 30]);
    let result =
        LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn less_than_op_on_int() {
    let lhs = OwnedColumn::Int(vec![1i32, 500, 300]);
    let rhs = OwnedColumn::Int(vec![3i32, 200, 300]);
    let result =
        LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
}

#[test]
fn less_than_op_empty_columns() {
    let lhs = OwnedColumn::BigInt(vec![]);
    let rhs = OwnedColumn::BigInt(vec![]);
    let result =
        LessThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![]));
}

#[test]
fn equal_op_empty_columns() {
    let lhs = OwnedColumn::BigInt(vec![]);
    let rhs = OwnedColumn::BigInt(vec![]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![]));
}

// ========== Mixed type comparisons (upcasting) ==========

#[test]
fn equal_op_uint8_with_smallint() {
    let lhs = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::SmallInt(vec![1i16, 2, 3]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, true, true]));
}

#[test]
fn equal_op_smallint_with_bigint() {
    let lhs = OwnedColumn::SmallInt(vec![100i16, 200]);
    let rhs = OwnedColumn::BigInt(vec![100i64, 999]);
    let result = EqualOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false]));
}

#[test]
fn greater_than_op_bigint_with_int128() {
    let lhs = OwnedColumn::BigInt(vec![100i64, 50]);
    let rhs = OwnedColumn::Int128(vec![99i128, 50]);
    let result =
        GreaterThanOp::owned_column_element_wise_comparison::<TestScalar>(&lhs, &rhs).unwrap();
    assert_eq!(result, OwnedColumn::Boolean(vec![true, false]));
}
