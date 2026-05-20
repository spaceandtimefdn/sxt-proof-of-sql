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
    use super::*;

    #[test]
    fn query_and_analyze_error_we_can_convert_analyze_errors_to_strings() {
        let error = AnalyzeError::InvalidDataType {
            expr_type: ColumnType::Boolean,
        };
        assert_eq!(
            String::from(error),
            "Expression has datatype BOOLEAN, which was not valid"
        );

        let error = AnalyzeError::DataTypeMismatch {
            left_type: "INT".to_string(),
            right_type: "VARCHAR".to_string(),
        };
        assert_eq!(
            String::from(error),
            "Left side has 'INT' type but right side has 'VARCHAR' type"
        );

        let error = AnalyzeError::DifferentColumnLength { len_a: 2, len_b: 3 };
        assert_eq!(
            String::from(error),
            "Columns have different lengths: 2 != 3"
        );

        assert_eq!(
            String::from(AnalyzeError::NotEnoughInputPlans),
            "Not enough input plans"
        );
    }

    #[test]
    fn query_and_analyze_error_we_can_convert_intermediate_decimal_error_to_analyze_error() {
        let error = AnalyzeError::from(IntermediateDecimalError::LossyCast);
        assert_eq!(
            error,
            AnalyzeError::DecimalConversionError {
                source: DecimalError::IntermediateDecimalConversionError {
                    source: IntermediateDecimalError::LossyCast
                }
            }
        );
        assert_eq!(
            String::from(error),
            "Fractional part of decimal is non-zero"
        );
    }
}
