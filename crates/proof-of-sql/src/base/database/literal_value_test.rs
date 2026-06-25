use crate::base::{
    database::LiteralValue,
    math::{decimal::Precision, i256::I256},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
};
use alloc::string::ToString;

// to_scalar tests for all LiteralValue variants
#[test]
fn to_scalar_boolean_false() {
    let lit = LiteralValue::Boolean(false);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::ZERO);
}

#[test]
fn to_scalar_boolean_true() {
    let lit = LiteralValue::Boolean(true);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::ONE);
}

#[test]
fn to_scalar_uint8() {
    let lit = LiteralValue::Uint8(255);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(255u8));
}

#[test]
fn to_scalar_tinyint() {
    let lit = LiteralValue::TinyInt(-42);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(-42i8));
}

#[test]
fn to_scalar_smallint() {
    let lit = LiteralValue::SmallInt(1000);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(1000i16));
}

#[test]
fn to_scalar_int() {
    let lit = LiteralValue::Int(-999_999);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(-999_999i32));
}

#[test]
fn to_scalar_bigint() {
    let lit = LiteralValue::BigInt(i64::MAX);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(i64::MAX));
}

#[test]
fn to_scalar_int128() {
    let lit = LiteralValue::Int128(i128::MIN);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(i128::MIN));
}

#[test]
fn to_scalar_varchar() {
    let lit = LiteralValue::VarChar("hello".to_string());
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from("hello"));
}

#[test]
fn to_scalar_varchar_empty() {
    let lit = LiteralValue::VarChar(String::new());
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(""));
}

#[test]
fn to_scalar_decimal75() {
    let value = I256::from(12345);
    let lit = LiteralValue::Decimal75(Precision::new(10).unwrap(), 2, value);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, value.into_scalar());
}

#[test]
fn to_scalar_timestamp_tz() {
    let lit = LiteralValue::TimeStampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), 1_000_000);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(1_000_000i64));
}

#[test]
fn to_scalar_scalar() {
    let limbs: [u64; 4] = [1, 2, 3, 4];
    let lit = LiteralValue::Scalar(limbs);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from(limbs));
}

#[test]
fn to_scalar_varbinary() {
    let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let lit = LiteralValue::VarBinary(data.clone());
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from_byte_slice_via_hash(&data));
}

#[test]
fn to_scalar_varbinary_empty() {
    let lit = LiteralValue::VarBinary(vec![]);
    let s: TestScalar = lit.to_scalar();
    assert_eq!(s, TestScalar::from_byte_slice_via_hash(&[]));
}

// column_type tests
#[test]
fn column_type_matches_for_all_variants() {
    use crate::base::database::ColumnType;

    assert_eq!(LiteralValue::Boolean(true).column_type(), ColumnType::Boolean);
    assert_eq!(LiteralValue::Uint8(0).column_type(), ColumnType::Uint8);
    assert_eq!(LiteralValue::TinyInt(0).column_type(), ColumnType::TinyInt);
    assert_eq!(LiteralValue::SmallInt(0).column_type(), ColumnType::SmallInt);
    assert_eq!(LiteralValue::Int(0).column_type(), ColumnType::Int);
    assert_eq!(LiteralValue::BigInt(0).column_type(), ColumnType::BigInt);
    assert_eq!(LiteralValue::Int128(0).column_type(), ColumnType::Int128);
    assert_eq!(
        LiteralValue::VarChar("x".to_string()).column_type(),
        ColumnType::VarChar
    );
    assert_eq!(
        LiteralValue::VarBinary(vec![1]).column_type(),
        ColumnType::VarBinary
    );
    assert_eq!(LiteralValue::Scalar([0; 4]).column_type(), ColumnType::Scalar);

    let p = Precision::new(20).unwrap();
    assert_eq!(
        LiteralValue::Decimal75(p, 5, I256::from(0)).column_type(),
        ColumnType::Decimal75(p, 5)
    );

    assert_eq!(
        LiteralValue::TimeStampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc(), 0)
            .column_type(),
        ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc())
    );
}

// serde round-trip for LiteralValue
#[test]
fn literal_value_can_be_serialized_and_deserialized() {
    let values = vec![
        LiteralValue::Boolean(false),
        LiteralValue::Uint8(128),
        LiteralValue::TinyInt(-1),
        LiteralValue::SmallInt(256),
        LiteralValue::Int(100_000),
        LiteralValue::BigInt(-1_000_000_000),
        LiteralValue::Int128(i128::MAX),
        LiteralValue::VarChar("test serde".to_string()),
        LiteralValue::Decimal75(Precision::new(5).unwrap(), 2, I256::from(999)),
        LiteralValue::TimeStampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::new(-3600), 42),
        LiteralValue::Scalar([10, 20, 30, 40]),
        LiteralValue::VarBinary(vec![1, 2, 3]),
    ];
    for val in &values {
        let serialized = serde_json::to_string(val).unwrap();
        let deserialized: LiteralValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val, &deserialized);
    }
}
