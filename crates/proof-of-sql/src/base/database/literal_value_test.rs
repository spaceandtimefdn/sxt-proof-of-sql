//! Tests for literal_value.rs — additional coverage beyond inline tests
use crate::base::{
    database::LiteralValue,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
};

#[test]
fn literal_value_column_type_for_all_variants() {
    assert_eq!(LiteralValue::Boolean(true).column_type(), ColumnType::Boolean);
    assert_eq!(LiteralValue::Uint8(1).column_type(), ColumnType::Uint8);
    assert_eq!(LiteralValue::TinyInt(-1).column_type(), ColumnType::TinyInt);
    assert_eq!(LiteralValue::SmallInt(100).column_type(), ColumnType::SmallInt);
    assert_eq!(LiteralValue::Int(42).column_type(), ColumnType::Int);
    assert_eq!(LiteralValue::BigInt(i64::MAX).column_type(), ColumnType::BigInt);
    assert_eq!(LiteralValue::Int128(i128::MIN).column_type(), ColumnType::Int128);
    assert_eq!(LiteralValue::VarChar("hello".to_string()).column_type(), ColumnType::VarChar);
    assert_eq!(LiteralValue::VarBinary(vec![1, 2, 3]).column_type(), ColumnType::VarBinary);
    assert_eq!(
        LiteralValue::Decimal75(Precision::new(12).unwrap(), 2, 7010.into()).column_type(),
        ColumnType::Decimal75(Precision::new(12).unwrap(), 2)
    );
    assert_eq!(
        LiteralValue::TimeStampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc(), 10).column_type(),
        ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc())
    );
    assert_eq!(LiteralValue::Scalar([1; 4]).column_type(), ColumnType::Scalar);
}

#[test]
fn literal_value_equality() {
    assert_eq!(LiteralValue::Int(42), LiteralValue::Int(42));
    assert_ne!(LiteralValue::Int(42), LiteralValue::Int(43));
    assert_eq!(LiteralValue::Boolean(true), LiteralValue::Boolean(true));
    assert_ne!(LiteralValue::Boolean(true), LiteralValue::Boolean(false));
    assert_eq!(
        LiteralValue::VarChar("test".to_string()),
        LiteralValue::VarChar("test".to_string())
    );
}

#[test]
fn literal_value_clone() {
    let lv = LiteralValue::BigInt(123);
    let cloned = lv.clone();
    assert_eq!(lv, cloned);
}

#[test]
fn literal_value_serde_roundtrip_boolean() {
    let lv = LiteralValue::Boolean(true);
    let json = serde_json::to_string(&lv).unwrap();
    let deserialized: LiteralValue = serde_json::from_str(&json).unwrap();
    assert_eq!(lv, deserialized);
}

#[test]
fn literal_value_serde_roundtrip_int() {
    let lv = LiteralValue::Int(42);
    let json = serde_json::to_string(&lv).unwrap();
    let deserialized: LiteralValue = serde_json::from_str(&json).unwrap();
    assert_eq!(lv, deserialized);
}

#[test]
fn literal_value_serde_roundtrip_bigint() {
    let lv = LiteralValue::BigInt(i64::MAX);
    let json = serde_json::to_string(&lv).unwrap();
    let deserialized: LiteralValue = serde_json::from_str(&json).unwrap();
    assert_eq!(lv, deserialized);
}

#[test]
fn literal_value_serde_roundtrip_varchar() {
    let lv = LiteralValue::VarChar("hello world".to_string());
    let json = serde_json::to_string(&lv).unwrap();
    let deserialized: LiteralValue = serde_json::from_str(&json).unwrap();
    assert_eq!(lv, deserialized);
}

#[test]
fn literal_value_serde_roundtrip_varbinary() {
    let lv = LiteralValue::VarBinary(vec![1, 2, 3, 4]);
    let json = serde_json::to_string(&lv).unwrap();
    let deserialized: LiteralValue = serde_json::from_str(&json).unwrap();
    assert_eq!(lv, deserialized);
}

#[test]
fn literal_value_serde_roundtrip_timestamptz() {
    let lv = LiteralValue::TimeStampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), 1625072400);
    let json = serde_json::to_string(&lv).unwrap();
    let deserialized: LiteralValue = serde_json::from_str(&json).unwrap();
    assert_eq!(lv, deserialized);
}

use crate::base::database::ColumnType;

#[test]
fn literal_value_to_scalar_boolean() {
    use crate::base::scalar::test_scalar::TestScalar;
    let lv = LiteralValue::Boolean(true);
    let s: TestScalar = lv.to_scalar();
    assert_eq!(s, TestScalar::from(true));
}

#[test]
fn literal_value_to_scalar_int() {
    use crate::base::scalar::test_scalar::TestScalar;
    let lv = LiteralValue::Int(42);
    let s: TestScalar = lv.to_scalar();
    assert_eq!(s, TestScalar::from(42));
}

#[test]
fn literal_value_to_scalar_bigint() {
    use crate::base::scalar::test_scalar::TestScalar;
    let lv = LiteralValue::BigInt(123);
    let s: TestScalar = lv.to_scalar();
    assert_eq!(s, TestScalar::from(123i64));
}
