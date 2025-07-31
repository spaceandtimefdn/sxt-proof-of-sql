use crate::base::{
    database::ColumnType,
    math::{decimal::Precision, i256::I256},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{Scalar, ScalarExt},
};
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

/// Represents a literal value.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LiteralValue {
    /// Boolean literals
    Boolean(bool),
    /// u8 literals
    Uint8(u8),
    /// i8 literals
    TinyInt(i8),
    /// i16 literals
    SmallInt(i16),
    /// i32 literals
    Int(i32),
    /// i64 literals
    BigInt(i64),

    /// String literals
    ///  - the first element maps to the str value.
    ///  - the second element maps to the str hash (see [`crate::base::scalar::Scalar`]).
    VarChar(String),
    /// Binary data literals
    ///  - the backing store is a Vec<u8> for variable length binary data
    VarBinary(Vec<u8>),
    /// i128 literals
    Int128(i128),
    /// Decimal literals with a max width of 252 bits
    ///  - the backing store maps to the type [`crate::base::scalar::TestScalar`]
    Decimal75(Precision, i8, I256),
    /// Scalar literals. The underlying `[u64; 4]` is the limbs of the canonical form of the literal
    Scalar([u64; 4]),
    /// `TimeStamp` defined over a unit (s, ms, ns, etc) and timezone with backing store
    /// mapped to i64, which is time units since unix epoch
    TimeStampTZ(PoSQLTimeUnit, PoSQLTimeZone, i64),
}

impl LiteralValue {
    /// Provides the column type associated with the column
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::Uint8(_) => ColumnType::Uint8,
            Self::TinyInt(_) => ColumnType::TinyInt,
            Self::SmallInt(_) => ColumnType::SmallInt,
            Self::Int(_) => ColumnType::Int,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::VarBinary(_) => ColumnType::VarBinary,
            Self::Int128(_) => ColumnType::Int128,
            Self::Scalar(_) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
            Self::TimeStampTZ(tu, tz, _) => ColumnType::TimestampTZ(*tu, *tz),
        }
    }

    /// Converts the literal to a scalar
    pub(crate) fn to_scalar<S: Scalar>(&self) -> S {
        match self {
            Self::Boolean(b) => b.into(),
            Self::Uint8(i) => i.into(),
            Self::TinyInt(i) => i.into(),
            Self::SmallInt(i) => i.into(),
            Self::Int(i) => i.into(),
            Self::BigInt(i) => i.into(),
            Self::VarChar(str) => str.into(),
            Self::VarBinary(bytes) => S::from_byte_slice_via_hash(bytes),
            Self::Decimal75(_, _, i) => i.into_scalar(),
            Self::Int128(i) => i.into(),
            Self::Scalar(limbs) => (*limbs).into(),
            Self::TimeStampTZ(_, _, time) => time.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        math::{decimal::Precision, i256::I256},
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn test_boolean_literal_value() {
        let true_val = LiteralValue::Boolean(true);
        let false_val = LiteralValue::Boolean(false);

        assert_eq!(true_val.column_type(), ColumnType::Boolean);
        assert_eq!(false_val.column_type(), ColumnType::Boolean);

        let true_scalar: TestScalar = true_val.to_scalar();
        let false_scalar: TestScalar = false_val.to_scalar();
        assert_ne!(true_scalar, false_scalar);
    }

    #[test]
    fn test_integer_literal_values() {
        let uint8_val = LiteralValue::Uint8(255);
        let tinyint_val = LiteralValue::TinyInt(-128);
        let smallint_val = LiteralValue::SmallInt(-32768);
        let int_val = LiteralValue::Int(-2_147_483_648);
        let bigint_val = LiteralValue::BigInt(-9_223_372_036_854_775_808);
        let int128_val = LiteralValue::Int128(-170_141_183_460_469_231_731_687_303_715_884_105_728);

        assert_eq!(uint8_val.column_type(), ColumnType::Uint8);
        assert_eq!(tinyint_val.column_type(), ColumnType::TinyInt);
        assert_eq!(smallint_val.column_type(), ColumnType::SmallInt);
        assert_eq!(int_val.column_type(), ColumnType::Int);
        assert_eq!(bigint_val.column_type(), ColumnType::BigInt);
        assert_eq!(int128_val.column_type(), ColumnType::Int128);

        // Test scalar conversion and verify values
        let uint8_scalar: TestScalar = uint8_val.to_scalar();
        let tinyint_scalar: TestScalar = tinyint_val.to_scalar();
        let smallint_scalar: TestScalar = smallint_val.to_scalar();
        let int_scalar: TestScalar = int_val.to_scalar();
        let bigint_scalar: TestScalar = bigint_val.to_scalar();
        let int128_scalar: TestScalar = int128_val.to_scalar();

        // Verify the scalar values match the original values
        assert_eq!(uint8_scalar, TestScalar::from(255u8));
        assert_eq!(tinyint_scalar, TestScalar::from(-128i8));
        assert_eq!(smallint_scalar, TestScalar::from(-32768i16));
        assert_eq!(int_scalar, TestScalar::from(-2_147_483_648i32));
        assert_eq!(bigint_scalar, TestScalar::from(-9_223_372_036_854_775_808i64));
        assert_eq!(int128_scalar, TestScalar::from(-170_141_183_460_469_231_731_687_303_715_884_105_728i128));
    }

    #[test]
    fn test_varchar_literal_value() {
        let varchar_val = LiteralValue::VarChar("test string".to_string());

        assert_eq!(varchar_val.column_type(), ColumnType::VarChar);

        let varchar_scalar: TestScalar = varchar_val.to_scalar();
        let varchar_scalar2: TestScalar =
            LiteralValue::VarChar("test string".to_string()).to_scalar();
        let different_varchar_scalar: TestScalar =
            LiteralValue::VarChar("different string".to_string()).to_scalar();

        assert_eq!(varchar_scalar, varchar_scalar2);
        assert_ne!(varchar_scalar, different_varchar_scalar);
    }

    #[test]
    fn test_varbinary_literal_value() {
        let varbinary_val = LiteralValue::VarBinary(vec![1, 2, 3, 4, 5]);

        assert_eq!(varbinary_val.column_type(), ColumnType::VarBinary);

        let varbinary_scalar: TestScalar = varbinary_val.to_scalar();
        let varbinary_scalar2: TestScalar =
            LiteralValue::VarBinary(vec![1, 2, 3, 4, 5]).to_scalar();
        let different_varbinary_scalar: TestScalar =
            LiteralValue::VarBinary(vec![5, 4, 3, 2, 1]).to_scalar();

        assert_eq!(varbinary_scalar, varbinary_scalar2);
        assert_ne!(varbinary_scalar, different_varbinary_scalar);
    }

    #[test]
    fn test_decimal75_literal_value() {
        let precision = Precision::new(10).unwrap();
        let scale = 2;
        let value = I256::from(12345);
        let decimal_val = LiteralValue::Decimal75(precision, scale, value);

        assert_eq!(
            decimal_val.column_type(),
            ColumnType::Decimal75(precision, scale)
        );

        let decimal_scalar: TestScalar = decimal_val.to_scalar();
        let decimal_scalar2: TestScalar =
            LiteralValue::Decimal75(precision, scale, value).to_scalar();

        assert_eq!(decimal_scalar, decimal_scalar2);
    }

    #[test]
    fn test_scalar_literal_value() {
        let limbs = [1, 2, 3, 4];
        let scalar_val = LiteralValue::Scalar(limbs);

        assert_eq!(scalar_val.column_type(), ColumnType::Scalar);

        let scalar_result: TestScalar = scalar_val.to_scalar();
        let scalar_result2: TestScalar = LiteralValue::Scalar(limbs).to_scalar();

        assert_eq!(scalar_result, scalar_result2);
    }

    #[test]
    fn test_timestamp_literal_value() {
        let unit = PoSQLTimeUnit::Millisecond;
        let timezone = PoSQLTimeZone::utc();
        let timestamp = 1_234_567_890_123;
        let timestamp_val = LiteralValue::TimeStampTZ(unit, timezone, timestamp);

        assert_eq!(
            timestamp_val.column_type(),
            ColumnType::TimestampTZ(unit, timezone)
        );

        let timestamp_scalar: TestScalar = timestamp_val.to_scalar();
        let timestamp_scalar2: TestScalar =
            LiteralValue::TimeStampTZ(unit, timezone, timestamp).to_scalar();

        assert_eq!(timestamp_scalar, timestamp_scalar2);
    }

    #[test]
    fn test_literal_value_equality() {
        let val1 = LiteralValue::Int(42);
        let val2 = LiteralValue::Int(42);
        let val3 = LiteralValue::Int(43);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);

        let val4 = LiteralValue::VarChar("hello".to_string());
        let val5 = LiteralValue::VarChar("hello".to_string());
        let val6 = LiteralValue::VarChar("world".to_string());

        assert_eq!(val4, val5);
        assert_ne!(val4, val6);
    }

    #[test]
    fn test_literal_value_clone() {
        let original = LiteralValue::VarChar("clone test".to_string());
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_literal_value_serialization() {
        let values = vec![
            LiteralValue::Boolean(true),
            LiteralValue::Uint8(255),
            LiteralValue::TinyInt(-128),
            LiteralValue::SmallInt(1000),
            LiteralValue::Int(-50000),
            LiteralValue::BigInt(9_223_372_036_854_775_807),
            LiteralValue::VarChar("test".to_string()),
            LiteralValue::VarBinary(vec![1, 2, 3]),
            LiteralValue::Int128(123_456_789),
            LiteralValue::Scalar([1, 2, 3, 4]),
            LiteralValue::TimeStampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), 1_234_567_890),
        ];

        for value in values {
            let serialized = serde_json::to_string(&value).unwrap();
            let deserialized: LiteralValue = serde_json::from_str(&serialized).unwrap();
            assert_eq!(value, deserialized);
        }
    }

    #[test]
    fn test_literal_value_debug() {
        let value = LiteralValue::Int(42);
        let debug_str = format!("{value:?}");
        assert!(debug_str.contains("Int"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_edge_case_values() {
        // Test boundary values
        let max_uint8 = LiteralValue::Uint8(u8::MAX);
        let min_tinyint = LiteralValue::TinyInt(i8::MIN);
        let max_tinyint = LiteralValue::TinyInt(i8::MAX);
        let min_smallint = LiteralValue::SmallInt(i16::MIN);
        let max_smallint = LiteralValue::SmallInt(i16::MAX);
        let min_int = LiteralValue::Int(i32::MIN);
        let max_int = LiteralValue::Int(i32::MAX);
        let min_bigint = LiteralValue::BigInt(i64::MIN);
        let max_bigint = LiteralValue::BigInt(i64::MAX);

        // Ensure all can be converted to scalar without panicking
        let _: TestScalar = max_uint8.to_scalar();
        let _: TestScalar = min_tinyint.to_scalar();
        let _: TestScalar = max_tinyint.to_scalar();
        let _: TestScalar = min_smallint.to_scalar();
        let _: TestScalar = max_smallint.to_scalar();
        let _: TestScalar = min_int.to_scalar();
        let _: TestScalar = max_int.to_scalar();
        let _: TestScalar = min_bigint.to_scalar();
        let _: TestScalar = max_bigint.to_scalar();
    }

    #[test]
    fn test_empty_string_and_binary() {
        let empty_string = LiteralValue::VarChar(String::new());
        let empty_binary = LiteralValue::VarBinary(Vec::new());

        assert_eq!(empty_string.column_type(), ColumnType::VarChar);
        assert_eq!(empty_binary.column_type(), ColumnType::VarBinary);

        // Ensure empty values can be converted to scalar
        let _: TestScalar = empty_string.to_scalar();
        let _: TestScalar = empty_binary.to_scalar();
    }
}
