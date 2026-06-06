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
    fn test_invalid_data_type_error() {
        let error = AnalyzeError::InvalidDataType {
            expr_type: ColumnType::BigInt,
        };
        let error_msg = error.to_string();
        assert!(error_msg.contains("BigInt"));
        assert!(error_msg.contains("not valid"));
    }

    #[test]
    fn test_data_type_mismatch_error() {
        let error = AnalyzeError::DataTypeMismatch {
            left_type: "Integer".to_string(),
            right_type: "String".to_string(),
        };
        let error_msg = error.to_string();
        assert!(error_msg.contains("Integer"));
        assert!(error_msg.contains("String"));
    }

    #[test]
    fn test_different_column_length_error() {
        let error = AnalyzeError::DifferentColumnLength {
            len_a: 10,
            len_b: 5,
        };
        let error_msg = error.to_string();
        assert!(error_msg.contains("10"));
        assert!(error_msg.contains("5"));
    }

    #[test]
    fn test_not_enough_input_plans_error() {
        let error = AnalyzeError::NotEnoughInputPlans;
        let error_msg = error.to_string();
        assert!(error_msg.contains("Not enough input plans"));
    }

    #[test]
    fn test_error_to_string_conversion() {
        let error = AnalyzeError::DataTypeMismatch {
            left_type: "Int".to_string(),
            right_type: "Float".to_string(),
        };
        let s: String = error.into();
        assert!(s.contains("Int"));
        assert!(s.contains("Float"));
    }

    #[test]
    fn test_from_intermediate_decimal_error() {
        let intermediate_error = IntermediateDecimalError::LossyCast {
            from: "1.5",
            to: "Integer",
        };
        let analyze_error: AnalyzeError = intermediate_error.into();
        match analyze_error {
            AnalyzeError::DecimalConversionError { source } => match source {
                DecimalError::IntermediateDecimalConversionError { .. } => {}
                _ => panic!("Expected IntermediateDecimalConversionError"),
            },
            _ => panic!("Expected DecimalConversionError"),
        }
    }

    #[test]
    fn test_analyze_result_ok() {
        let result: AnalyzeResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_analyze_result_err() {
        let result: AnalyzeResult<i32> = Err(AnalyzeError::NotEnoughInputPlans);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_equality() {
        let error1 = AnalyzeError::NotEnoughInputPlans;
        let error2 = AnalyzeError::NotEnoughInputPlans;
        assert_eq!(error1, error2);

        let error3 = AnalyzeError::DifferentColumnLength {
            len_a: 5,
            len_b: 10,
        };
        let error4 = AnalyzeError::DifferentColumnLength {
            len_a: 5,
            len_b: 10,
        };
        assert_eq!(error3, error4);
    }

    #[test]
    fn test_error_inequality() {
        let error1 = AnalyzeError::NotEnoughInputPlans;
        let error2 = AnalyzeError::DifferentColumnLength {
            len_a: 5,
            len_b: 10,
        };
        assert_ne!(error1, error2);
    }
}
