//! Additional tests for column_type.rs — covering is_numeric, is_integer, max_integer_type,
//! max_unsigned_integer_type, precision_value, scale, is_signed, min_scalar
use crate::base::database::ColumnType;
use crate::base::math::decimal::Precision;
use crate::base::scalar::test_scalar::TestScalar;

// === is_numeric tests ===

#[test]
fn is_numeric_for_numeric_types() {
    assert!(ColumnType::Uint8.is_numeric());
    assert!(ColumnType::TinyInt.is_numeric());
    assert!(ColumnType::SmallInt.is_numeric());
    assert!(ColumnType::Int.is_numeric());
    assert!(ColumnType::BigInt.is_numeric());
    assert!(ColumnType::Int128.is_numeric());
    assert!(ColumnType::Scalar.is_numeric());
    assert!(ColumnType::Decimal75(Precision::new(12).unwrap(), 2).is_numeric());
}

#[test]
fn is_numeric_false_for_non_numeric() {
    assert!(!ColumnType::Boolean.is_numeric());
    assert!(!ColumnType::VarChar.is_numeric());
    assert!(!ColumnType::VarBinary.is_numeric());
}

// === is_integer tests ===

#[test]
fn is_integer_for_integer_types() {
    assert!(ColumnType::Uint8.is_integer());
    assert!(ColumnType::TinyInt.is_integer());
    assert!(ColumnType::SmallInt.is_integer());
    assert!(ColumnType::Int.is_integer());
    assert!(ColumnType::BigInt.is_integer());
    assert!(ColumnType::Int128.is_integer());
}

#[test]
fn is_integer_false_for_non_integer() {
    assert!(!ColumnType::Boolean.is_integer());
    assert!(!ColumnType::Scalar.is_integer());
    assert!(!ColumnType::VarChar.is_integer());
    assert!(!ColumnType::Decimal75(Precision::new(12).unwrap(), 2).is_integer());
}

// === is_signed tests ===

#[test]
fn is_signed_for_signed_types() {
    assert!(ColumnType::TinyInt.is_signed());
    assert!(ColumnType::SmallInt.is_signed());
    assert!(ColumnType::Int.is_signed());
    assert!(ColumnType::BigInt.is_signed());
    assert!(ColumnType::Int128.is_signed());
}

#[test]
fn is_signed_false_for_unsigned() {
    assert!(!ColumnType::Uint8.is_signed());
    assert!(!ColumnType::Boolean.is_signed());
}

// === max_integer_type tests ===

#[test]
fn max_integer_type_same_types() {
    assert_eq!(ColumnType::Int.max_integer_type(&ColumnType::Int), Some(ColumnType::Int));
    assert_eq!(ColumnType::BigInt.max_integer_type(&ColumnType::BigInt), Some(ColumnType::BigInt));
}

#[test]
fn max_integer_type_different_types() {
    assert_eq!(ColumnType::Int.max_integer_type(&ColumnType::BigInt), Some(ColumnType::BigInt));
    assert_eq!(ColumnType::BigInt.max_integer_type(&ColumnType::Int), Some(ColumnType::BigInt));
    assert_eq!(ColumnType::SmallInt.max_integer_type(&ColumnType::Int), Some(ColumnType::Int));
}

#[test]
fn max_integer_type_with_non_integer_returns_none() {
    assert_eq!(ColumnType::Int.max_integer_type(&ColumnType::Boolean), None);
    assert_eq!(ColumnType::Boolean.max_integer_type(&ColumnType::Int), None);
}

// === max_unsigned_integer_type tests ===

#[test]
fn max_unsigned_integer_type_uint8() {
    assert_eq!(
        ColumnType::Uint8.max_unsigned_integer_type(&ColumnType::Uint8),
        Some(ColumnType::Uint8)
    );
}

#[test]
fn max_unsigned_integer_type_with_signed_returns_none() {
    assert_eq!(ColumnType::Uint8.max_unsigned_integer_type(&ColumnType::Int), None);
    assert_eq!(ColumnType::Int.max_unsigned_integer_type(&ColumnType::Uint8), None);
}

// === precision_value and scale tests ===

#[test]
fn precision_value_decimal75() {
    let ct = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
    assert_eq!(ct.precision_value(), Some(12));
}

#[test]
fn precision_value_non_decimal() {
    assert_eq!(ColumnType::Int.precision_value(), None);
    assert_eq!(ColumnType::BigInt.precision_value(), None);
}

#[test]
fn scale_decimal75() {
    let ct = ColumnType::Decimal75(Precision::new(12).unwrap(), 3);
    assert_eq!(ct.scale(), Some(3));
}

#[test]
fn scale_non_decimal() {
    assert_eq!(ColumnType::Int.scale(), None);
}

// === byte_size and bit_size ===

#[test]
fn byte_size_for_types() {
    assert_eq!(ColumnType::Boolean.byte_size(), 1);
    assert_eq!(ColumnType::Uint8.byte_size(), 1);
    assert_eq!(ColumnType::TinyInt.byte_size(), 1);
    assert_eq!(ColumnType::SmallInt.byte_size(), 2);
    assert_eq!(ColumnType::Int.byte_size(), 4);
    assert_eq!(ColumnType::BigInt.byte_size(), 8);
    assert_eq!(ColumnType::Int128.byte_size(), 16);
    assert_eq!(ColumnType::Scalar.byte_size(), 32);
}

#[test]
fn bit_size_for_types() {
    assert_eq!(ColumnType::Boolean.bit_size(), 8);
    assert_eq!(ColumnType::Uint8.bit_size(), 8);
    assert_eq!(ColumnType::SmallInt.bit_size(), 16);
    assert_eq!(ColumnType::Int.bit_size(), 32);
    assert_eq!(ColumnType::BigInt.bit_size(), 64);
    assert_eq!(ColumnType::Int128.bit_size(), 128);
    assert_eq!(ColumnType::Scalar.bit_size(), 256);
}

// === min_scalar tests ===

#[test]
fn min_scalar_for_integer_types() {
    assert_eq!(ColumnType::TinyInt.min_scalar::<TestScalar>(), Some(TestScalar::from(i8::MIN)));
    assert_eq!(ColumnType::SmallInt.min_scalar::<TestScalar>(), Some(TestScalar::from(i16::MIN)));
    assert_eq!(ColumnType::Int.min_scalar::<TestScalar>(), Some(TestScalar::from(i32::MIN)));
    assert_eq!(ColumnType::BigInt.min_scalar::<TestScalar>(), Some(TestScalar::from(i64::MIN)));
}

#[test]
fn min_scalar_for_uint8() {
    assert_eq!(ColumnType::Uint8.min_scalar::<TestScalar>(), Some(TestScalar::from(0u8)));
}

#[test]
fn min_scalar_for_non_integer_returns_none() {
    assert_eq!(ColumnType::Boolean.min_scalar::<TestScalar>(), None);
    assert_eq!(ColumnType::VarChar.min_scalar::<TestScalar>(), None);
    assert_eq!(ColumnType::Scalar.min_scalar::<TestScalar>(), None);
}

// === Equality and Clone ===

#[test]
fn column_type_equality() {
    assert_eq!(ColumnType::Int, ColumnType::Int);
    assert_ne!(ColumnType::Int, ColumnType::BigInt);
}

#[test]
fn column_type_decimal75_equality() {
    let ct1 = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
    let ct2 = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
    assert_eq!(ct1, ct2);
}

#[test]
fn column_type_clone() {
    let ct = ColumnType::Int128;
    let cloned = ct.clone();
    assert_eq!(ct, cloned);
}

#[test]
fn column_type_serde_roundtrip() {
    let ct = ColumnType::BigInt;
    let json = serde_json::to_string(&ct).unwrap();
    let deserialized: ColumnType = serde_json::from_str(&json).unwrap();
    assert_eq!(ct, deserialized);
}

#[test]
fn column_type_serde_roundtrip_decimal75() {
    let ct = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
    let json = serde_json::to_string(&ct).unwrap();
    let deserialized: ColumnType = serde_json::from_str(&json).unwrap();
    assert_eq!(ct, deserialized);
}

#[test]
fn column_type_serde_roundtrip_varchar() {
    let ct = ColumnType::VarChar;
    let json = serde_json::to_string(&ct).unwrap();
    let deserialized: ColumnType = serde_json::from_str(&json).unwrap();
    assert_eq!(ct, deserialized);
}
