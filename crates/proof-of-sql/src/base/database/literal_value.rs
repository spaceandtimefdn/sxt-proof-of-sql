use crate::base::{
    database::ColumnType,
    math::{decimal::Precision, i256::I256},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{Scalar, ScalarExt},
    standard_serializations::limbs::{deserialize_to_limbs, serialize_limbs},
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
    /// i128 literals
    Int128(i128),

    /// String literals
    ///  - the first element maps to the str value.
    ///  - the second element maps to the str hash (see [`crate::base::scalar::Scalar`]).
    VarChar(String),
    /// Decimal literals with a max width of 252 bits
    ///  - the backing store maps to the type [`crate::base::scalar::Curve25519Scalar`]
    Decimal75(Precision, i8, I256),
    /// `TimeStamp` defined over a unit (s, ms, ns, etc) and timezone with backing store
    /// mapped to i64, which is time units since unix epoch
    TimeStampTZ(PoSQLTimeUnit, PoSQLTimeZone, i64),
    /// Scalar literals. The underlying `[u64; 4]` is the limbs of the canonical form of the literal
    #[serde(
        serialize_with = "serialize_limbs",
        deserialize_with = "deserialize_to_limbs"
    )]
    Scalar([u64; 4]),
    /// Binary data literals
    ///  - the backing store is a Vec<u8> for variable length binary data
    VarBinary(Vec<u8>),
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
    use crate::base::{
        database::LiteralValue,
        math::{decimal::Precision, i256::I256},
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
        try_standard_binary_serialization,
    };

    /// This allows us to reuse code within solidity more safely
    #[test]
    fn literal_value_and_column_type_varaints_should_have_same_data_type_serialization() {
        let literal_values = vec![
            LiteralValue::Boolean(true),
            LiteralValue::Uint8(2),
            LiteralValue::TinyInt(3),
            LiteralValue::SmallInt(4),
            LiteralValue::Int(5),
            LiteralValue::BigInt(6),
            LiteralValue::Int128(7),
            LiteralValue::VarChar("test".to_string()),
            LiteralValue::Decimal75(Precision::new(9).unwrap(), 2, 7010.into()),
            LiteralValue::TimeStampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc(), 10),
            LiteralValue::Scalar([1; 4]),
            LiteralValue::VarBinary(vec![1]),
        ];
        for literal_value in literal_values {
            let column_type = literal_value.column_type();
            let serialized_column_type =
                hex::encode(try_standard_binary_serialization(column_type).unwrap());
            let serialized_literal_value =
                hex::encode(try_standard_binary_serialization(literal_value).unwrap());
            assert!(serialized_literal_value.starts_with(&serialized_column_type));
        }
    }

    #[test]
    fn literal_values_convert_to_expected_scalars() {
        let decimal_value = I256::from(-12_345_i128);
        let scalar_limbs = [11, 12, 13, 14];
        let binary_value = vec![1, 2, 3, 4, 5];

        let cases = vec![
            (LiteralValue::Boolean(false), TestScalar::from(false)),
            (LiteralValue::Boolean(true), TestScalar::from(true)),
            (LiteralValue::Uint8(2), TestScalar::from(2_u8)),
            (LiteralValue::TinyInt(-3), TestScalar::from(-3_i8)),
            (LiteralValue::SmallInt(-4), TestScalar::from(-4_i16)),
            (LiteralValue::Int(-5), TestScalar::from(-5_i32)),
            (LiteralValue::BigInt(-6), TestScalar::from(-6_i64)),
            (LiteralValue::Int128(-7), TestScalar::from(-7_i128)),
            (
                LiteralValue::VarChar("pii@example.com".to_string()),
                TestScalar::from("pii@example.com"),
            ),
            (
                LiteralValue::VarBinary(vec![]),
                TestScalar::from_byte_slice_via_hash(&[]),
            ),
            (
                LiteralValue::VarBinary(binary_value.clone()),
                TestScalar::from_byte_slice_via_hash(&binary_value),
            ),
            (
                LiteralValue::Decimal75(Precision::new(20).unwrap(), 4, decimal_value),
                decimal_value.into_scalar(),
            ),
            (
                LiteralValue::TimeStampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc(), -42),
                TestScalar::from(-42_i64),
            ),
            (
                LiteralValue::Scalar(scalar_limbs),
                TestScalar::from(scalar_limbs),
            ),
        ];

        for (literal_value, expected_scalar) in cases {
            assert_eq!(
                literal_value.to_scalar::<TestScalar>(),
                expected_scalar,
                "{literal_value:?} should use the same scalar conversion as column evaluation"
            );
        }
    }
}
