//! Additional tests for column.rs — covering from_literal_with_length, rho, scalar_at, to_scalar
use crate::base::{
    database::{Column, ColumnType, LiteralValue, OwnedColumn},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;

// === from_literal_with_length tests ===

#[test]
fn from_literal_boolean_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::Boolean(true), 4, &alloc);
    assert_eq!(col, Column::Boolean(&[true, true, true, true]));
    assert_eq!(col.column_type(), ColumnType::Boolean);
}

#[test]
fn from_literal_int_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::Int(42), 3, &alloc);
    assert_eq!(col, Column::Int(&[42, 42, 42]));
}

#[test]
fn from_literal_bigint_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::BigInt(99), 2, &alloc);
    assert_eq!(col, Column::BigInt(&[99, 99]));
}

#[test]
fn from_literal_tinyint_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::TinyInt(-5), 3, &alloc);
    assert_eq!(col, Column::TinyInt(&[-5, -5, -5]));
}

#[test]
fn from_literal_smallint_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::SmallInt(1000), 2, &alloc);
    assert_eq!(col, Column::SmallInt(&[1000, 1000]));
}

#[test]
fn from_literal_uint8_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::Uint8(255), 2, &alloc);
    assert_eq!(col, Column::Uint8(&[255, 255]));
}

#[test]
fn from_literal_int128_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::Int128(12345), 2, &alloc);
    assert_eq!(col, Column::Int128(&[12345, 12345]));
}

#[test]
fn from_literal_varchar_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::VarChar("hello".to_string()), 3, &alloc);
    match col {
        Column::VarChar((strs, scalars)) => {
            assert_eq!(strs.len(), 3);
            assert_eq!(scalars.len(), 3);
            for s in strs.iter() {
                assert_eq!(*s, "hello");
            }
        }
        _ => panic!("Expected VarChar column"),
    }
}

#[test]
fn from_literal_varbinary_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(
        &LiteralValue::VarBinary(vec![1, 2, 3]),
        2,
        &alloc,
    );
    match col {
        Column::VarBinary((bytes, scalars)) => {
            assert_eq!(bytes.len(), 2);
            assert_eq!(scalars.len(), 2);
        }
        _ => panic!("Expected VarBinary column"),
    }
}

#[test]
fn from_literal_timestamptz_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(
        &LiteralValue::TimeStampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), 1000),
        3,
        &alloc,
    );
    assert_eq!(
        col,
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[1000, 1000, 1000])
    );
}

#[test]
fn from_literal_decimal75_creates_constant_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(
        &LiteralValue::Decimal75(Precision::new(12).unwrap(), 2, 7010.into()),
        3,
        &alloc,
    );
    assert!(matches!(col, Column::Decimal75(_, _, _)));
    assert_eq!(col.len(), 3);
}

#[test]
fn from_literal_with_zero_length() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::from_literal_with_length(&LiteralValue::Int(42), 0, &alloc);
    assert_eq!(col, Column::Int(&[]));
    assert!(col.is_empty());
}

// === rho function tests ===

#[test]
fn rho_creates_sequential_int128_column() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::rho(5, &alloc);
    assert_eq!(col, Column::Int128(&[0, 1, 2, 3, 4]));
    assert_eq!(col.column_type(), ColumnType::Int128);
}

#[test]
fn rho_with_zero_length() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::rho(0, &alloc);
    assert_eq!(col, Column::Int128(&[]));
    assert!(col.is_empty());
}

#[test]
fn rho_with_single_element() {
    let alloc = Bump::new();
    let col = Column::<TestScalar>::rho(1, &alloc);
    assert_eq!(col, Column::Int128(&[0]));
}

// === column_type tests for all variants ===

#[test]
fn column_type_matches_variant() {
    assert_eq!(Column::<TestScalar>::Boolean(&[true]).column_type(), ColumnType::Boolean);
    assert_eq!(Column::<TestScalar>::Uint8(&[1u8]).column_type(), ColumnType::Uint8);
    assert_eq!(Column::<TestScalar>::TinyInt(&[1i8]).column_type(), ColumnType::TinyInt);
    assert_eq!(Column::<TestScalar>::SmallInt(&[1i16]).column_type(), ColumnType::SmallInt);
    assert_eq!(Column::<TestScalar>::Int(&[1]).column_type(), ColumnType::Int);
    assert_eq!(Column::<TestScalar>::BigInt(&[1i64]).column_type(), ColumnType::BigInt);
    assert_eq!(Column::<TestScalar>::Int128(&[1i128]).column_type(), ColumnType::Int128);
    assert_eq!(
        Column::Scalar(&[TestScalar::from(1)]).column_type(),
        ColumnType::Scalar
    );
}

#[test]
fn column_type_timestamptz_preserves_unit_and_timezone() {
    let col = Column::<TestScalar>::TimestampTZ(
        PoSQLTimeUnit::Millisecond,
        PoSQLTimeZone::utc(),
        &[100],
    );
    assert_eq!(col.column_type(), ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc()));
}

// === from_owned_column tests for remaining types ===

#[test]
fn from_owned_column_uint8() {
    let alloc = Bump::new();
    let owned = OwnedColumn::Uint8(vec![1, 2, 3]);
    let col = Column::<TestScalar>::from_owned_column(&owned, &alloc);
    assert_eq!(col, Column::Uint8(&[1, 2, 3]));
}

#[test]
fn from_owned_column_tinyint() {
    let alloc = Bump::new();
    let owned = OwnedColumn::TinyInt(vec![-1, 0, 1]);
    let col = Column::<TestScalar>::from_owned_column(&owned, &alloc);
    assert_eq!(col, Column::TinyInt(&[-1, 0, 1]));
}

#[test]
fn from_owned_column_smallint() {
    let alloc = Bump::new();
    let owned = OwnedColumn::SmallInt(vec![100, 200]);
    let col = Column::<TestScalar>::from_owned_column(&owned, &alloc);
    assert_eq!(col, Column::SmallInt(&[100, 200]));
}

#[test]
fn from_owned_column_timestamptz() {
    let alloc = Bump::new();
    let owned = OwnedColumn::TimestampTZ(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        vec![1000, 2000],
    );
    let col = Column::<TestScalar>::from_owned_column(&owned, &alloc);
    assert_eq!(
        col,
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[1000, 2000])
    );
}

// === to_scalar tests ===

#[test]
fn to_scalar_converts_boolean_column() {
    let col = Column::<TestScalar>::Boolean(&[true, false, true]);
    let scalars = col.to_scalar();
    assert_eq!(scalars.len(), 3);
}

#[test]
fn to_scalar_converts_int_column() {
    let col = Column::<TestScalar>::Int(&[1, 2, 3]);
    let scalars = col.to_scalar();
    assert_eq!(scalars.len(), 3);
}

#[test]
fn to_scalar_converts_bigint_column() {
    let col = Column::<TestScalar>::BigInt(&[10, 20]);
    let scalars = col.to_scalar();
    assert_eq!(scalars.len(), 2);
}

// === as_* accessor tests for wrong type ===

#[test]
fn as_boolean_returns_none_for_non_boolean() {
    let col = Column::<TestScalar>::Int(&[1, 2, 3]);
    assert!(col.as_boolean().is_none());
    assert!(col.as_int().is_some());
}

#[test]
fn as_int_returns_none_for_bigint() {
    let col = Column::<TestScalar>::BigInt(&[1i64, 2]);
    assert!(col.as_int().is_none());
    assert!(col.as_bigint().is_some());
}

#[test]
fn as_varchar_returns_none_for_scalar() {
    let col = Column::<TestScalar>::Scalar(&[TestScalar::from(1)]);
    assert!(col.as_varchar().is_none());
}
