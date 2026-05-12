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
    use alloc::{
        string::{String, ToString},
        vec,
    };

    #[test]
    fn we_can_render_analyze_error_messages() {
        let errors = vec![
            (
                AnalyzeError::InvalidDataType {
                    expr_type: ColumnType::Boolean,
                },
                "Expression has datatype BOOLEAN, which was not valid",
            ),
            (
                AnalyzeError::DataTypeMismatch {
                    left_type: "INT".to_string(),
                    right_type: "BIGINT".to_string(),
                },
                "Left side has 'INT' type but right side has 'BIGINT' type",
            ),
            (
                AnalyzeError::DifferentColumnLength { len_a: 2, len_b: 3 },
                "Columns have different lengths: 2 != 3",
            ),
            (AnalyzeError::NotEnoughInputPlans, "Not enough input plans"),
        ];

        for (error, expected_message) in errors {
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn we_can_convert_analyze_errors_into_strings() {
        let message: String = AnalyzeError::NotEnoughInputPlans.into();

        assert_eq!(message, "Not enough input plans");
    }

    #[test]
    fn we_can_convert_intermediate_decimal_errors_into_analyze_errors() {
        let error = AnalyzeError::from(IntermediateDecimalError::OutOfRange);

        assert!(matches!(error, AnalyzeError::DecimalConversionError { .. }));
        assert_eq!(error.to_string(), "Value out of range for target type");
    }

    #[test]
    fn we_can_render_transparent_placeholder_errors() {
        let error = AnalyzeError::PlaceholderError {
            source: PlaceholderError::InvalidPlaceholderIndex {
                index: 2,
                num_params: 1,
            },
        };

        assert_eq!(
            error.to_string(),
            "Invalid placeholder index: 2, number of params: 1"
        );
    }
}
