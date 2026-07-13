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
    use alloc::string::ToString;

    #[test]
    fn we_can_display_analyze_errors() {
        let cases = [
            (
                AnalyzeError::InvalidDataType {
                    expr_type: ColumnType::VarChar,
                },
                "Expression has datatype VARCHAR, which was not valid",
            ),
            (
                AnalyzeError::DataTypeMismatch {
                    left_type: "BIGINT".to_string(),
                    right_type: "BOOLEAN".to_string(),
                },
                "Left side has 'BIGINT' type but right side has 'BOOLEAN' type",
            ),
            (
                AnalyzeError::DifferentColumnLength { len_a: 2, len_b: 5 },
                "Columns have different lengths: 2 != 5",
            ),
            (
                AnalyzeError::DecimalConversionError {
                    source: DecimalError::InvalidScale {
                        scale: "129".to_string(),
                    },
                },
                "Decimal scale is not valid: 129",
            ),
            (
                AnalyzeError::PlaceholderError {
                    source: PlaceholderError::ZeroPlaceholderId,
                },
                "Placeholder id must be greater than 0",
            ),
            (AnalyzeError::NotEnoughInputPlans, "Not enough input plans"),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn we_can_convert_analyze_errors_into_strings() {
        let message = String::from(AnalyzeError::DataTypeMismatch {
            left_type: "INT".to_string(),
            right_type: "VARCHAR".to_string(),
        });

        assert_eq!(
            message,
            "Left side has 'INT' type but right side has 'VARCHAR' type"
        );
    }

    #[test]
    fn we_can_convert_intermediate_decimal_errors_into_analyze_errors() {
        let error = AnalyzeError::from(IntermediateDecimalError::OutOfRange);

        assert_eq!(error.to_string(), "Value out of range for target type");
    }
}
