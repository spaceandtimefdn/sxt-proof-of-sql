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
    fn we_can_display_direct_analyze_errors() {
        assert_eq!(
            AnalyzeError::InvalidDataType {
                expr_type: ColumnType::Boolean
            }
            .to_string(),
            "Expression has datatype BOOLEAN, which was not valid"
        );
        assert_eq!(
            AnalyzeError::DataTypeMismatch {
                left_type: "INT".to_string(),
                right_type: "VARCHAR".to_string(),
            }
            .to_string(),
            "Left side has 'INT' type but right side has 'VARCHAR' type"
        );
        assert_eq!(
            AnalyzeError::DifferentColumnLength { len_a: 3, len_b: 5 }.to_string(),
            "Columns have different lengths: 3 != 5"
        );
        assert_eq!(
            AnalyzeError::NotEnoughInputPlans.to_string(),
            "Not enough input plans"
        );
    }

    #[test]
    fn we_can_convert_analyze_errors_into_strings() {
        let err = AnalyzeError::DataTypeMismatch {
            left_type: "BOOLEAN".to_string(),
            right_type: "BIGINT".to_string(),
        };

        assert_eq!(
            String::from(err),
            "Left side has 'BOOLEAN' type but right side has 'BIGINT' type"
        );
    }

    #[test]
    fn we_wrap_intermediate_decimal_errors_as_analyze_errors() {
        let err = AnalyzeError::from(IntermediateDecimalError::LossyCast);

        assert_eq!(err.to_string(), "Fractional part of decimal is non-zero");
        assert!(matches!(
            err,
            AnalyzeError::DecimalConversionError {
                source: DecimalError::IntermediateDecimalConversionError {
                    source: IntermediateDecimalError::LossyCast
                }
            }
        ));
    }
}
