use crate::base::{database::ColumnType, math::decimal::Precision, scalar::Scalar};
use serde::{Deserialize, Serialize};

/// Represents a literal value.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LiteralValue<S: Scalar> {
    /// Boolean literals
    Boolean(bool),
    /// i64 literals
    BigInt(i64),
    /// String literals
    ///  - the first element maps to the str value.
    ///  - the second element maps to the str hash (see [crate::base::scalar::Scalar]).
    VarChar((String, S)),
    /// i128 literals
    Int128(i128),
    /// Decimal literals with a max width of 252 bits
    ///  - the backing store maps to the type [crate::base::scalar::Curve25519Scalar]
    Decimal75(Precision, i8, S),
    /// Scalar literals
    Scalar(S),
}

impl<S: Scalar> LiteralValue<S> {
    /// Provides the column type associated with the column
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
            Self::Scalar(_) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
        }
    }

    /// Converts the literal to a scalar
    pub(crate) fn to_scalar(&self) -> S {
        match self {
            Self::Boolean(b) => b.into(),
            Self::BigInt(i) => i.into(),
            Self::VarChar((_, s)) => *s,
            Self::Int128(i) => i.into(),
            Self::Decimal75(_, _, s) => *s,
            Self::Scalar(scalar) => *scalar,
        }
    }
}
