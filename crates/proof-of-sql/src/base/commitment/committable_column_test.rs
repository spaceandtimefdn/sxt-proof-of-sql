//! Tests for committable_column.rs — covering len, is_empty, column_type, From conversions
use crate::base::{
    commitment::CommittableColumn,
    database::{Column, ColumnType, OwnedColumn},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;

// === len and is_empty ===

#[test]
fn committable_column_len_boolean() {
    let cc = CommittableColumn::Boolean(&[true, false, true]);
    assert_eq!(cc.len(), 3);
    assert!(!cc.is_empty());
}

#[test]
fn committable_column_len_int() {
    let cc = CommittableColumn::Int(&[1, 2, 3, 4, 5]);
    assert_eq!(cc.len(), 5);
}

#[test]
fn committable_column_len_bigint() {
    let cc = CommittableColumn::BigInt(&[1i64, 2]);
    assert_eq!(cc.len(), 2);
}

#[test]
fn committable_column_len_int128() {
    let cc = CommittableColumn::Int128(&[1i128, 2, 3]);
    assert_eq!(cc.len(), 3);
}

#[test]
fn committable_column_len_decimal75() {
    let cc = CommittableColumn::Decimal75(Precision::new(12).unwrap(), 2, vec![[1u64; 4], [2u64; 4]]);
    assert_eq!(cc.len(), 2);
}

#[test]
fn committable_column_len_scalar() {
    let cc = CommittableColumn::Scalar(vec![[1u64; 4], [2u64; 4], [3u64; 4]]);
    assert_eq!(cc.len(), 3);
}

#[test]
fn committable_column_len_varchar() {
    let cc = CommittableColumn::VarChar(vec![[1u64; 4]]);
    assert_eq!(cc.len(), 1);
}

#[test]
fn committable_column_len_varbinary() {
    let cc = CommittableColumn::VarBinary(vec![[1u64; 4], [2u64; 4]]);
    assert_eq!(cc.len(), 2);
}

#[test]
fn committable_column_len_timestamptz() {
    let cc = CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[100i64, 200]);
    assert_eq!(cc.len(), 2);
}

#[test]
fn committable_column_empty() {
    let cc = CommittableColumn::Int(&[]);
    assert_eq!(cc.len(), 0);
    assert!(cc.is_empty());
}

#[test]
fn committable_column_len_uint8() {
    let cc = CommittableColumn::Uint8(&[1u8, 2, 3]);
    assert_eq!(cc.len(), 3);
}

#[test]
fn committable_column_len_tinyint() {
    let cc = CommittableColumn::TinyInt(&[-1i8, 0, 1]);
    assert_eq!(cc.len(), 3);
}

#[test]
fn committable_column_len_smallint() {
    let cc = CommittableColumn::SmallInt(&[100i16, 200]);
    assert_eq!(cc.len(), 2);
}

// === column_type via From ===

#[test]
fn committable_column_type_boolean() {
    let cc = CommittableColumn::Boolean(&[true]);
    assert_eq!(cc.column_type(), ColumnType::Boolean);
}

#[test]
fn committable_column_type_int() {
    let cc = CommittableColumn::Int(&[1]);
    assert_eq!(cc.column_type(), ColumnType::Int);
}

#[test]
fn committable_column_type_bigint() {
    let cc = CommittableColumn::BigInt(&[1i64]);
    assert_eq!(cc.column_type(), ColumnType::BigInt);
}

#[test]
fn committable_column_type_int128() {
    let cc = CommittableColumn::Int128(&[1i128]);
    assert_eq!(cc.column_type(), ColumnType::Int128);
}

#[test]
fn committable_column_type_scalar() {
    let cc = CommittableColumn::Scalar(vec![[0u64; 4]]);
    assert_eq!(cc.column_type(), ColumnType::Scalar);
}

#[test]
fn committable_column_type_varchar() {
    let cc = CommittableColumn::VarChar(vec![[0u64; 4]]);
    assert_eq!(cc.column_type(), ColumnType::VarChar);
}

#[test]
fn committable_column_type_varbinary() {
    let cc = CommittableColumn::VarBinary(vec![[0u64; 4]]);
    assert_eq!(cc.column_type(), ColumnType::VarBinary);
}

#[test]
fn committable_column_type_decimal75() {
    let cc = CommittableColumn::Decimal75(Precision::new(12).unwrap(), 3, vec![[0u64; 4]]);
    assert_eq!(cc.column_type(), ColumnType::Decimal75(Precision::new(12).unwrap(), 3));
}

#[test]
fn committable_column_type_timestamptz() {
    let cc = CommittableColumn::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc(), &[0]);
    assert_eq!(cc.column_type(), ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc()));
}

#[test]
fn committable_column_type_uint8() {
    let cc = CommittableColumn::Uint8(&[1u8]);
    assert_eq!(cc.column_type(), ColumnType::Uint8);
}

#[test]
fn committable_column_type_tinyint() {
    let cc = CommittableColumn::TinyInt(&[1i8]);
    assert_eq!(cc.column_type(), ColumnType::TinyInt);
}

#[test]
fn committable_column_type_smallint() {
    let cc = CommittableColumn::SmallInt(&[1i16]);
    assert_eq!(cc.column_type(), ColumnType::SmallInt);
}

// === From<&Column> for CommittableColumn ===

#[test]
fn from_column_boolean() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::Boolean(&[true, false]);
    let cc = CommittableColumn::from(&col);
    assert_eq!(cc, CommittableColumn::Boolean(&[true, false]));
}

#[test]
fn from_column_int() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::Int(&[1, 2, 3]);
    let cc = CommittableColumn::from(&col);
    assert_eq!(cc, CommittableColumn::Int(&[1, 2, 3]));
}

#[test]
fn from_column_bigint() {
    let col = Column::<TestScalar>::BigInt(&[10i64, 20]);
    let cc = CommittableColumn::from(&col);
    assert_eq!(cc, CommittableColumn::BigInt(&[10, 20]));
}

#[test]
fn from_column_int128() {
    let col = Column::<TestScalar>::Int128(&[1i128]);
    let cc = CommittableColumn::from(&col);
    assert_eq!(cc, CommittableColumn::Int128(&[1]));
}

#[test]
fn from_column_uint8() {
    let col = Column::<TestScalar>::Uint8(&[1u8, 2]);
    let cc = CommittableColumn::from(&col);
    assert_eq!(cc, CommittableColumn::Uint8(&[1, 2]));
}

#[test]
fn from_column_timestamptz() {
    let col = Column::<TestScalar>::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[100i64]);
    let cc = CommittableColumn::from(&col);
    assert_eq!(cc, CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[100]));
}

// === From<&OwnedColumn> for CommittableColumn ===

#[test]
fn from_owned_column_boolean() {
    let owned: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, false]);
    let cc = CommittableColumn::from(&owned);
    assert_eq!(cc, CommittableColumn::Boolean(&[true, false]));
}

#[test]
fn from_owned_column_uint8() {
    let owned: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1u8, 2, 3]);
    let cc = CommittableColumn::from(&owned);
    assert_eq!(cc, CommittableColumn::Uint8(&[1, 2, 3]));
}

#[test]
fn from_owned_column_int() {
    let owned: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10, 20]);
    let cc = CommittableColumn::from(&owned);
    assert_eq!(cc, CommittableColumn::Int(&[10, 20]));
}

#[test]
fn from_owned_column_bigint() {
    let owned: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1i64, 2]);
    let cc = CommittableColumn::from(&owned);
    assert_eq!(cc, CommittableColumn::BigInt(&[1, 2]));
}

// === From slice conversions ===

#[test]
fn from_u8_slice() {
    let data: &[u8] = &[1, 2, 3];
    let cc = CommittableColumn::from(data);
    assert_eq!(cc, CommittableColumn::Uint8(&[1, 2, 3]));
}

#[test]
fn from_i8_slice() {
    let data: &[i8] = &[-1, 0, 1];
    let cc = CommittableColumn::from(data);
    assert_eq!(cc, CommittableColumn::TinyInt(&[-1, 0, 1]));
}

#[test]
fn from_i16_slice() {
    let data: &[i16] = &[100, 200];
    let cc = CommittableColumn::from(data);
    assert_eq!(cc, CommittableColumn::SmallInt(&[100, 200]));
}

#[test]
fn from_i32_slice() {
    let data: &[i32] = &[1, 2];
    let cc = CommittableColumn::from(data);
    assert_eq!(cc, CommittableColumn::Int(&[1, 2]));
}

// === Clone and PartialEq ===

#[test]
fn committable_column_clone() {
    let cc = CommittableColumn::Int(&[1, 2, 3]);
    let cloned = cc.clone();
    assert_eq!(cc, cloned);
}

#[test]
fn committable_column_equality() {
    let cc1 = CommittableColumn::BigInt(&[1, 2]);
    let cc2 = CommittableColumn::BigInt(&[1, 2]);
    assert_eq!(cc1, cc2);
}

#[test]
fn committable_column_inequality() {
    let cc1 = CommittableColumn::Int(&[1, 2]);
    let cc2 = CommittableColumn::Int(&[1, 3]);
    assert_ne!(cc1, cc2);
}
