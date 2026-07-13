use crate::base::{
    database::ColumnType,
    math::decimal::{DecimalError, IntermediateDecimalError},
    proof::PlaceholderError,
};
use alloc::string::{String, ToString};
use core::result::Result;
use snafu::Snafu;

/// Errors related to queries that can not be run due to invalid column references, data types, etc.
/// Will be replaced once we fully switch to the planner.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum AnalyzeError {
    #[snafu(display("Expression has datatype {expr_type}, which was not valid"))]
    /// Invalid data type received
    InvalidDataType {
        /// data type found
        expr_type: ColumnType,
    },

    #[snafu(display("Left side has '{left_type}' type but right side has '{right_type}' type"))]
    /// Data types do not match
    DataTypeMismatch {
        /// The left side datatype
        left_type: String,
        /// The right side datatype
        right_type: String,
    },

    #[snafu(display("Columns have different lengths: {len_a} != {len_b}"))]
    /// Two columns do not have the same length
    DifferentColumnLength {
        /// The length of the first column
        len_a: usize,
        /// The length of the second column
        len_b: usize,
    },

    #[snafu(transparent)]
    /// Errors related to decimal operations
    DecimalConversionError {
        /// The underlying source error
        source: DecimalError,
    },

    #[snafu(transparent)]
    /// Errors related to placeholders
    PlaceholderError {
        /// The underlying source error
        source: PlaceholderError,
    },

    #[snafu(display("Not enough input plans"))]
    /// Error for when there are not enough input plans (for a union for example)
    NotEnoughInputPlans,
}

impl From<AnalyzeError> for String {
    fn from(error: AnalyzeError) -> Self {
        error.to_string()
    }
}

impl From<IntermediateDecimalError> for AnalyzeError {
    fn from(err: IntermediateDecimalError) -> AnalyzeError {
        AnalyzeError::DecimalConversionError {
            source: DecimalError::IntermediateDecimalConversionError { source: err },
        }
    }
}

/// Result type for analyze errors
pub type AnalyzeResult<T> = Result<T, AnalyzeError>;

#[cfg(test)]
mod tests {
    use super::AnalyzeError;
    use crate::base::{
        database::ColumnType, math::decimal::IntermediateDecimalError, proof::PlaceholderError,
    };
    use alloc::string::String;

    #[test]
    fn we_display_basic_analyze_errors() {
        assert_eq!(
            AnalyzeError::InvalidDataType {
                expr_type: ColumnType::Boolean
            }
            .to_string(),
            "Expression has datatype BOOLEAN, which was not valid"
        );
        assert_eq!(
            AnalyzeError::DataTypeMismatch {
                left_type: "BIGINT".into(),
                right_type: "VARCHAR".into(),
            }
            .to_string(),
            "Left side has 'BIGINT' type but right side has 'VARCHAR' type"
        );
        assert_eq!(
            AnalyzeError::DifferentColumnLength { len_a: 7, len_b: 8 }.to_string(),
            "Columns have different lengths: 7 != 8"
        );
        assert_eq!(
            AnalyzeError::NotEnoughInputPlans.to_string(),
            "Not enough input plans"
        );
    }

    #[test]
    fn we_convert_analyze_errors_to_strings() {
        let message: String = AnalyzeError::DataTypeMismatch {
            left_type: "INT".into(),
            right_type: "SCALAR".into(),
        }
        .into();

        assert_eq!(
            message,
            "Left side has 'INT' type but right side has 'SCALAR' type"
        );
    }

    #[test]
    fn we_transparently_display_placeholder_errors() {
        let error = AnalyzeError::PlaceholderError {
            source: PlaceholderError::InvalidPlaceholderType {
                index: 2,
                expected: ColumnType::Int,
                actual: ColumnType::VarChar,
            },
        };

        assert_eq!(
            error.to_string(),
            "Invalid placeholder type: 2, expected: INT, actual: VARCHAR"
        );
    }

    #[test]
    fn we_convert_intermediate_decimal_errors_into_analyze_errors() {
        let error: AnalyzeError = IntermediateDecimalError::LossyCast.into();

        assert_eq!(error.to_string(), "Fractional part of decimal is non-zero");
    }
}
