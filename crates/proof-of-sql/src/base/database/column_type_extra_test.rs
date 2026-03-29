use crate::base::{
    database::ColumnType,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
};
use alloc::string::ToString;

// Display tests (covering all branches of the Display impl)
#[test]
fn we_can_display_all_column_types() {
    assert_eq!(ColumnType::Boolean.to_string(), "BOOLEAN");
    assert_eq!(ColumnType::Uint8.to_string(), "UINT8");
    assert_eq!(ColumnType::TinyInt.to_string(), "TINYINT");
    assert_eq!(ColumnType::SmallInt.to_string(), "SMALLINT");
    assert_eq!(ColumnType::Int.to_string(), "INT");
    assert_eq!(ColumnType::BigInt.to_string(), "BIGINT");
    assert_eq!(ColumnType::Int128.to_string(), "DECIMAL");
    assert_eq!(ColumnType::VarChar.to_string(), "VARCHAR");
    assert_eq!(ColumnType::VarBinary.to_string(), "BINARY");
    assert_eq!(ColumnType::Scalar.to_string(), "SCALAR");

    let decimal = ColumnType::Decimal75(Precision::new(38).unwrap(), 10);
    assert_eq!(decimal.to_string(), "DECIMAL75(PRECISION: 38, SCALE: 10)");

    let timestamp = ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc());
    assert_eq!(
        timestamp.to_string(),
        "TIMESTAMP(TIMEUNIT: seconds (precision: 0), TIMEZONE: +00:00)"
    );

    let timestamp_ms =
        ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::new(3600));
    assert_eq!(
        timestamp_ms.to_string(),
        "TIMESTAMP(TIMEUNIT: milliseconds (precision: 3), TIMEZONE: +01:00)"
    );
}

// is_numeric
#[test]
fn is_numeric_returns_true_for_numeric_types() {
    assert!(ColumnType::Uint8.is_numeric());
    assert!(ColumnType::TinyInt.is_numeric());
    assert!(ColumnType::SmallInt.is_numeric());
    assert!(ColumnType::Int.is_numeric());
    assert!(ColumnType::BigInt.is_numeric());
    assert!(ColumnType::Int128.is_numeric());
    assert!(ColumnType::Scalar.is_numeric());
    assert!(ColumnType::Decimal75(Precision::new(10).unwrap(), 5).is_numeric());
}

#[test]
fn is_numeric_returns_false_for_non_numeric_types() {
    assert!(!ColumnType::Boolean.is_numeric());
    assert!(!ColumnType::VarChar.is_numeric());
    assert!(!ColumnType::VarBinary.is_numeric());
    assert!(
        !ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()).is_numeric()
    );
}

// is_integer
#[test]
fn is_integer_returns_true_for_integer_types() {
    assert!(ColumnType::Uint8.is_integer());
    assert!(ColumnType::TinyInt.is_integer());
    assert!(ColumnType::SmallInt.is_integer());
    assert!(ColumnType::Int.is_integer());
    assert!(ColumnType::BigInt.is_integer());
    assert!(ColumnType::Int128.is_integer());
}

#[test]
fn is_integer_returns_false_for_non_integer_types() {
    assert!(!ColumnType::Boolean.is_integer());
    assert!(!ColumnType::VarChar.is_integer());
    assert!(!ColumnType::VarBinary.is_integer());
    assert!(!ColumnType::Scalar.is_integer());
    assert!(!ColumnType::Decimal75(Precision::new(10).unwrap(), 5).is_integer());
    assert!(
        !ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()).is_integer()
    );
}

// is_signed
#[test]
fn is_signed_returns_true_for_signed_types() {
    assert!(ColumnType::TinyInt.is_signed());
    assert!(ColumnType::SmallInt.is_signed());
    assert!(ColumnType::Int.is_signed());
    assert!(ColumnType::BigInt.is_signed());
    assert!(ColumnType::Int128.is_signed());
    assert!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()).is_signed()
    );
}

#[test]
fn is_signed_returns_false_for_unsigned_types() {
    assert!(!ColumnType::Uint8.is_signed());
    assert!(!ColumnType::Boolean.is_signed());
    assert!(!ColumnType::VarChar.is_signed());
    assert!(!ColumnType::VarBinary.is_signed());
    assert!(!ColumnType::Scalar.is_signed());
    assert!(!ColumnType::Decimal75(Precision::new(10).unwrap(), 5).is_signed());
}

// byte_size
#[test]
fn byte_size_returns_correct_sizes() {
    assert_eq!(ColumnType::Boolean.byte_size(), 1);
    assert_eq!(ColumnType::Uint8.byte_size(), 1);
    assert_eq!(ColumnType::TinyInt.byte_size(), 1);
    assert_eq!(ColumnType::SmallInt.byte_size(), 2);
    assert_eq!(ColumnType::Int.byte_size(), 4);
    assert_eq!(ColumnType::BigInt.byte_size(), 8);
    assert_eq!(ColumnType::Int128.byte_size(), 16);
    assert_eq!(ColumnType::Scalar.byte_size(), 32);
    assert_eq!(
        ColumnType::Decimal75(Precision::new(10).unwrap(), 5).byte_size(),
        32
    );
    assert_eq!(ColumnType::VarChar.byte_size(), 32);
    assert_eq!(ColumnType::VarBinary.byte_size(), 32);
    assert_eq!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()).byte_size(),
        8
    );
}

// bit_size
#[test]
fn bit_size_is_eight_times_byte_size() {
    let types = [
        ColumnType::Boolean,
        ColumnType::Uint8,
        ColumnType::TinyInt,
        ColumnType::SmallInt,
        ColumnType::Int,
        ColumnType::BigInt,
        ColumnType::Int128,
        ColumnType::Scalar,
        ColumnType::VarChar,
        ColumnType::VarBinary,
        ColumnType::Decimal75(Precision::new(10).unwrap(), 5),
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
    ];
    for ct in types {
        assert_eq!(ct.bit_size(), ct.byte_size() as u32 * 8);
    }
}

// max_integer_type
#[test]
fn max_integer_type_returns_larger_of_two_integers() {
    assert_eq!(
        ColumnType::TinyInt.max_integer_type(&ColumnType::BigInt),
        Some(ColumnType::BigInt)
    );
    assert_eq!(
        ColumnType::Int128.max_integer_type(&ColumnType::SmallInt),
        Some(ColumnType::Int128)
    );
    assert_eq!(
        ColumnType::Int.max_integer_type(&ColumnType::Int),
        Some(ColumnType::Int)
    );
    // Uint8 with signed type should return signed (since max_integer_type uses from_signed_integer_bits)
    assert_eq!(
        ColumnType::Uint8.max_integer_type(&ColumnType::TinyInt),
        Some(ColumnType::TinyInt)
    );
    assert_eq!(
        ColumnType::Uint8.max_integer_type(&ColumnType::BigInt),
        Some(ColumnType::BigInt)
    );
}

#[test]
fn max_integer_type_returns_none_for_non_integer() {
    assert_eq!(
        ColumnType::Boolean.max_integer_type(&ColumnType::Int),
        None
    );
    assert_eq!(
        ColumnType::Int.max_integer_type(&ColumnType::VarChar),
        None
    );
    assert_eq!(
        ColumnType::Scalar.max_integer_type(&ColumnType::Scalar),
        None
    );
    assert_eq!(
        ColumnType::Decimal75(Precision::new(10).unwrap(), 5)
            .max_integer_type(&ColumnType::Int),
        None
    );
}

// max_unsigned_integer_type
#[test]
fn max_unsigned_integer_type_returns_uint8_only_for_8_bit_types() {
    // Both Uint8: max bits = 8, from_unsigned_integer_bits(8) => Uint8
    assert_eq!(
        ColumnType::Uint8.max_unsigned_integer_type(&ColumnType::Uint8),
        Some(ColumnType::Uint8)
    );
    // TinyInt is 8 bits, so max is still 8 => Uint8
    assert_eq!(
        ColumnType::Uint8.max_unsigned_integer_type(&ColumnType::TinyInt),
        Some(ColumnType::Uint8)
    );
    assert_eq!(
        ColumnType::TinyInt.max_unsigned_integer_type(&ColumnType::TinyInt),
        Some(ColumnType::Uint8)
    );
}

#[test]
fn max_unsigned_integer_type_returns_none_for_larger_than_8_bits() {
    // SmallInt is 16 bits, from_unsigned_integer_bits(16) => None
    assert_eq!(
        ColumnType::Uint8.max_unsigned_integer_type(&ColumnType::SmallInt),
        None
    );
    assert_eq!(
        ColumnType::Int.max_unsigned_integer_type(&ColumnType::BigInt),
        None
    );
}

#[test]
fn max_unsigned_integer_type_returns_none_for_non_integer() {
    assert_eq!(
        ColumnType::VarChar.max_unsigned_integer_type(&ColumnType::Int),
        None
    );
    assert_eq!(
        ColumnType::Int.max_unsigned_integer_type(&ColumnType::Boolean),
        None
    );
}

// precision_value
#[test]
fn precision_value_returns_correct_values() {
    assert_eq!(ColumnType::Uint8.precision_value(), Some(3));
    assert_eq!(ColumnType::TinyInt.precision_value(), Some(3));
    assert_eq!(ColumnType::SmallInt.precision_value(), Some(5));
    assert_eq!(ColumnType::Int.precision_value(), Some(10));
    assert_eq!(ColumnType::BigInt.precision_value(), Some(19));
    assert_eq!(ColumnType::Int128.precision_value(), Some(39));
    assert_eq!(ColumnType::Scalar.precision_value(), Some(0));
    assert_eq!(
        ColumnType::Decimal75(Precision::new(42).unwrap(), 7).precision_value(),
        Some(42)
    );
    assert_eq!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()).precision_value(),
        Some(19)
    );
}

#[test]
fn precision_value_returns_none_for_non_numeric_types() {
    assert_eq!(ColumnType::Boolean.precision_value(), None);
    assert_eq!(ColumnType::VarChar.precision_value(), None);
    assert_eq!(ColumnType::VarBinary.precision_value(), None);
}

// scale
#[test]
fn scale_returns_correct_values() {
    assert_eq!(ColumnType::TinyInt.scale(), Some(0));
    assert_eq!(ColumnType::Uint8.scale(), Some(0));
    assert_eq!(ColumnType::SmallInt.scale(), Some(0));
    assert_eq!(ColumnType::Int.scale(), Some(0));
    assert_eq!(ColumnType::BigInt.scale(), Some(0));
    assert_eq!(ColumnType::Int128.scale(), Some(0));
    assert_eq!(ColumnType::Scalar.scale(), Some(0));
    assert_eq!(
        ColumnType::Decimal75(Precision::new(10).unwrap(), -3).scale(),
        Some(-3)
    );
    assert_eq!(
        ColumnType::Decimal75(Precision::new(10).unwrap(), 5).scale(),
        Some(5)
    );
}

#[test]
fn scale_returns_correct_values_for_timestamp_units() {
    assert_eq!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()).scale(),
        Some(0)
    );
    assert_eq!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc()).scale(),
        Some(3)
    );
    assert_eq!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Microsecond, PoSQLTimeZone::utc()).scale(),
        Some(6)
    );
    assert_eq!(
        ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc()).scale(),
        Some(9)
    );
}

#[test]
fn scale_returns_none_for_non_numeric_types() {
    assert_eq!(ColumnType::Boolean.scale(), None);
    assert_eq!(ColumnType::VarChar.scale(), None);
    assert_eq!(ColumnType::VarBinary.scale(), None);
}
