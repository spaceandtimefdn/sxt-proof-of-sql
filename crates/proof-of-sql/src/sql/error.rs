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
    use alloc::{format, string::ToString};

    #[test]
    fn analyze_error_invalid_data_type_displays_correctly() {
        let err = AnalyzeError::InvalidDataType {
            expr_type: ColumnType::VarChar,
        };
        let msg = format!("{err}");
        assert!(msg.contains("VARCHAR"));
        assert!(msg.contains("not valid"));
    }

    #[test]
    fn analyze_error_data_type_mismatch_displays_correctly() {
        let err = AnalyzeError::DataTypeMismatch {
            left_type: "INT".to_string(),
            right_type: "VARCHAR".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("INT"));
        assert!(msg.contains("VARCHAR"));
    }

    #[test]
    fn analyze_error_different_column_length_displays_correctly() {
        let err = AnalyzeError::DifferentColumnLength { len_a: 5, len_b: 10 };
        let msg = format!("{err}");
        assert!(msg.contains("5"));
        assert!(msg.contains("10"));
    }

    #[test]
    fn analyze_error_not_enough_input_plans_displays_correctly() {
        let err = AnalyzeError::NotEnoughInputPlans;
        assert_eq!(format!("{err}"), "Not enough input plans");
    }

    #[test]
    fn analyze_error_converts_to_string() {
        let err = AnalyzeError::NotEnoughInputPlans;
        let s: String = err.into();
        assert_eq!(s, "Not enough input plans");
    }

    #[test]
    fn intermediate_decimal_error_converts_to_analyze_error() {
        use crate::base::math::decimal::IntermediateDecimalError;
        let decimal_err = IntermediateDecimalError::LossyCast;
        let analyze_err: AnalyzeError = decimal_err.into();
        assert!(matches!(
            analyze_err,
            AnalyzeError::DecimalConversionError { .. }
        ));
    }
}
