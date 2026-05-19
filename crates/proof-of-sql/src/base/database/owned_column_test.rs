//! Tests for owned_column.rs — covering len, is_empty, slice, column_type, iterators, try_from_scalars
use crate::base::{
    database::{ColumnType, OwnedColumn, OwnedColumnError},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};

// === len and is_empty tests ===

#[test]
fn owned_column_len_and_is_empty() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 2, 3]);
    assert_eq!(col.len(), 3);
    assert!(!col.is_empty());

    let empty = OwnedColumn::<TestScalar>::BigInt(vec![]);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}

// === column_type for all variants ===

#[test]
fn owned_column_type_for_all_variants() {
    assert_eq!(OwnedColumn::<TestScalar>::Boolean(vec![true]).column_type(), ColumnType::Boolean);
    assert_eq!(OwnedColumn::<TestScalar>::Uint8(vec![1u8]).column_type(), ColumnType::Uint8);
    assert_eq!(OwnedColumn::<TestScalar>::TinyInt(vec![1i8]).column_type(), ColumnType::TinyInt);
    assert_eq!(OwnedColumn::<TestScalar>::SmallInt(vec![1i16]).column_type(), ColumnType::SmallInt);
    assert_eq!(OwnedColumn::<TestScalar>::Int(vec![1]).column_type(), ColumnType::Int);
    assert_eq!(OwnedColumn::<TestScalar>::BigInt(vec![1i64]).column_type(), ColumnType::BigInt);
    assert_eq!(OwnedColumn::<TestScalar>::Int128(vec![1i128]).column_type(), ColumnType::Int128);
    assert_eq!(OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string()]).column_type(), ColumnType::VarChar);
    assert_eq!(OwnedColumn::<TestScalar>::VarBinary(vec![vec![1u8]]).column_type(), ColumnType::VarBinary);
    assert_eq!(
        OwnedColumn::<TestScalar>::Scalar(vec![TestScalar::from(1)]).column_type(),
        ColumnType::Scalar
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::Decimal75(Precision::new(12).unwrap(), 2, vec![TestScalar::from(1)]).column_type(),
        ColumnType::Decimal75(Precision::new(12).unwrap(), 2)
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![1000i64]).column_type(),
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc())
    );
}

// === slice tests ===

#[test]
fn owned_column_slice_bigint() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30, 40, 50]);
    let sliced: OwnedColumn<TestScalar> = col.slice(1, 4);
    assert_eq!(sliced, OwnedColumn::BigInt(vec![20, 30, 40]));
}

#[test]
fn owned_column_slice_boolean() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, false, true, false]);
    let sliced: OwnedColumn<TestScalar> = col.slice(0, 2);
    assert_eq!(sliced, OwnedColumn::Boolean(vec![true, false]));
}

#[test]
fn owned_column_slice_varchar() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    let sliced: OwnedColumn<TestScalar> = col.slice(1, 3);
    assert_eq!(sliced, OwnedColumn::VarChar(vec!["b".to_string(), "c".to_string()]));
}

#[test]
fn owned_column_slice_full() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 3]);
    let sliced: OwnedColumn<TestScalar> = col.slice(0, 3);
    assert_eq!(sliced, OwnedColumn::Int(vec![1, 2, 3]));
}

// === iterator tests ===

#[test]
fn owned_column_i64_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
    let values: Vec<i64> = col.i64_iter().copied().collect();
    assert_eq!(values, vec![10, 20, 30]);
}

#[test]
fn owned_column_i32_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 3]);
    let values: Vec<i32> = col.i32_iter().copied().collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn owned_column_bool_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, false, true]);
    let values: Vec<bool> = col.bool_iter().copied().collect();
    assert_eq!(values, vec![true, false, true]);
}

#[test]
fn owned_column_string_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["hello".to_string(), "world".to_string()]);
    let values: Vec<&String> = col.string_iter().collect();
    assert_eq!(values[0], "hello");
    assert_eq!(values[1], "world");
}

#[test]
fn owned_column_i128_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1i128, 2, 3]);
    let values: Vec<i128> = col.i128_iter().copied().collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn owned_column_u8_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let values: Vec<u8> = col.u8_iter().copied().collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn owned_column_i8_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![-1i8, 0, 1]);
    let values: Vec<i8> = col.i8_iter().copied().collect();
    assert_eq!(values, vec![-1, 0, 1]);
}

#[test]
fn owned_column_i16_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![100i16, 200]);
    let values: Vec<i16> = col.i16_iter().copied().collect();
    assert_eq!(values, vec![100, 200]);
}

#[test]
fn owned_column_scalar_iter() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(vec![TestScalar::from(1), TestScalar::from(2)]);
    let values: Vec<&TestScalar> = col.scalar_iter().collect();
    assert_eq!(values.len(), 2);
}

// === try_from_scalars tests ===

#[test]
fn try_from_scalars_bigint() {
    let scalars = [TestScalar::from(42), TestScalar::from(99)];
    let col: OwnedColumn<TestScalar> = OwnedColumn::<TestScalar>::try_from_scalars(&scalars, ColumnType::BigInt).unwrap();
    assert_eq!(col, OwnedColumn::BigInt(vec![42, 99]));
}

#[test]
fn try_from_scalars_boolean() {
    let scalars = [TestScalar::from(0), TestScalar::from(1)];
    let col: OwnedColumn<TestScalar> = OwnedColumn::<TestScalar>::try_from_scalars(&scalars, ColumnType::Boolean).unwrap();
    assert_eq!(col, OwnedColumn::Boolean(vec![false, true]));
}

#[test]
fn try_from_scalars_int() {
    let scalars = [TestScalar::from(10)];
    let col: OwnedColumn<TestScalar> = OwnedColumn::<TestScalar>::try_from_scalars(&scalars, ColumnType::Int).unwrap();
    assert_eq!(col, OwnedColumn::Int(vec![10]));
}

// === Equality and Clone ===

#[test]
fn owned_column_equality() {
    let col1: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 3]);
    let col2: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 3]);
    assert_eq!(col1, col2);
}

#[test]
fn owned_column_inequality() {
    let col1: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 3]);
    let col2: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 4]);
    assert_ne!(col1, col2);
}

#[test]
fn owned_column_clone() {
    let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![100, 200]);
    let cloned = col.clone();
    assert_eq!(col, cloned);
}
