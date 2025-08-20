//! Module for parsing an `IntermediateDecimal` into a `Decimal75`.
use alloc::string::{String, ToString};
use bigdecimal::ParseBigDecimalError;
use serde::{Deserialize, Deserializer, Serialize};
use snafu::Snafu;

/// Errors related to the processing of decimal values in proof-of-sql
#[derive(Snafu, Debug, PartialEq)]
pub enum IntermediateDecimalError {
    /// Represents an error encountered during the parsing of a decimal string.
    #[snafu(display("{error}"))]
    ParseError {
        /// The underlying error
        error: ParseBigDecimalError,
    },
    /// Error occurs when this decimal cannot fit in a primitive.
    #[snafu(display("Value out of range for target type"))]
    OutOfRange,
    /// Error occurs when this decimal cannot be losslessly cast into a primitive.
    #[snafu(display("Fractional part of decimal is non-zero"))]
    LossyCast,
    /// Cannot cast this decimal to a big integer
    #[snafu(display("Conversion to integer failed"))]
    ConversionFailure,
}

impl Eq for IntermediateDecimalError {}

/// Errors related to decimal operations.
#[derive(Snafu, Debug, Eq, PartialEq)]
pub enum DecimalError {
    #[snafu(display("Invalid decimal format or value: {error}"))]
    /// Error when a decimal format or value is incorrect,
    /// the string isn't even a decimal e.g. "notastring",
    /// "-21.233.122" etc aka `InvalidDecimal`
    InvalidDecimal {
        /// The underlying error
        error: String,
    },

    #[snafu(display("Decimal precision is not valid: {error}"))]
    /// Decimal precision exceeds the allowed limit,
    /// e.g. precision above 75/76/whatever set by Scalar
    /// or non-positive aka `InvalidPrecision`
    InvalidPrecision {
        /// The underlying error
        error: String,
    },

    #[snafu(display("Decimal scale is not valid: {scale}"))]
    /// Decimal scale is not valid. Here we use i16 in order to include
    /// invalid scale values
    InvalidScale {
        /// The invalid scale value
        scale: String,
    },

    #[snafu(display("Unsupported operation: cannot round decimal: {error}"))]
    /// This error occurs when attempting to scale a
    /// decimal in such a way that a loss of precision occurs.
    RoundingError {
        /// The underlying error
        error: String,
    },

    /// Errors that may occur when parsing an intermediate decimal
    /// into a posql decimal
    #[snafu(transparent)]
    IntermediateDecimalConversionError {
        /// The underlying source error
        source: IntermediateDecimalError,
    },
}

/// Result type for decimal operations.
pub type DecimalResult<T> = Result<T, DecimalError>;

// This exists because `TryFrom<arrow::datatypes::DataType>` for `ColumnType` error is String
impl From<DecimalError> for String {
    fn from(error: DecimalError) -> Self {
        error.to_string()
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Copy)]
/// limit-enforced precision
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Precision(#[cfg_attr(test, proptest(strategy = "1..76u8"))] u8);
pub(crate) const MAX_SUPPORTED_PRECISION: u8 = 75;

impl Precision {
    /// Constructor for creating a Precision instance
    pub fn new(value: u8) -> Result<Self, DecimalError> {
        if value > MAX_SUPPORTED_PRECISION || value == 0 {
            Err(DecimalError::InvalidPrecision {
                error: value.to_string(),
            })
        } else {
            Ok(Precision(value))
        }
    }

    /// Gets the precision as a u8 for this decimal
    #[must_use]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl TryFrom<u64> for Precision {
    type Error = DecimalError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Precision::new(
            value
                .try_into()
                .map_err(|_| DecimalError::InvalidPrecision {
                    error: value.to_string(),
                })?,
        )
    }
}

// Custom deserializer for precision since we need to limit its value to 75
impl<'de> Deserialize<'de> for Precision {
    fn deserialize<D>(deserializer: D) -> Result<Precision, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a u8
        let value = u8::deserialize(deserializer)?;

        // Use the Precision::new method to ensure the value is within the allowed range
        Precision::new(value).map_err(serde::de::Error::custom)
    }
}
