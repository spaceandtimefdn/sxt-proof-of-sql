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
    use crate::base::database::ColumnType;
    use alloc::string::ToString;

    #[test]
    fn invalid_data_type_displays_expr_type() {
        let err = AnalyzeError::InvalidDataType { expr_type: ColumnType::BigInt };
        let msg = err.to_string();
        assert!(msg.contains("BigInt"), "expected BigInt in: {msg}");
        assert!(msg.contains("not valid"), "expected 'not valid' in: {msg}");
    }

    #[test]
    fn data_type_mismatch_displays_both_sides() {
        let err = AnalyzeError::DataTypeMismatch {
            left_type: "BigInt".to_string(),
            right_type: "Boolean".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("BigInt"), "expected BigInt in: {msg}");
        assert!(msg.contains("Boolean"), "expected Boolean in: {msg}");
    }

    #[test]
    fn different_column_length_displays_both_lengths() {
        let err = AnalyzeError::DifferentColumnLength { len_a: 3, len_b: 7 };
        assert_eq!(err.to_string(), "Columns have different lengths: 3 != 7");
    }

    #[test]
    fn not_enough_input_plans_displays_correctly() {
        assert_eq!(
            AnalyzeError::NotEnoughInputPlans.to_string(),
            "Not enough input plans"
        );
    }

    #[test]
    fn analyze_error_converts_to_string_via_from_impl() {
        let err = AnalyzeError::NotEnoughInputPlans;
        let s: alloc::string::String = err.into();
        assert_eq!(s, "Not enough input plans");
    }

    #[test]
    fn analyze_error_implements_partial_eq() {
        assert_eq!(AnalyzeError::NotEnoughInputPlans, AnalyzeError::NotEnoughInputPlans);
        assert_ne!(
            AnalyzeError::NotEnoughInputPlans,
            AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 2 }
        );
    }

    #[test]
    fn analyze_error_debug_includes_variant_name() {
        let debug = format!("{:?}", AnalyzeError::NotEnoughInputPlans);
        assert!(debug.contains("NotEnoughInputPlans"));
    }

    #[test]
    fn invalid_data_type_with_boolean_contains_boolean() {
        let err = AnalyzeError::InvalidDataType { expr_type: ColumnType::Boolean };
        let msg = err.to_string();
        assert!(msg.contains("Boolean"), "expected Boolean in: {msg}");
    }

    #[test]
    fn data_type_mismatch_different_column_length_are_not_equal() {
        let mismatch = AnalyzeError::DataTypeMismatch {
            left_type: "Int".to_string(),
            right_type: "BigInt".to_string(),
        };
        let length = AnalyzeError::DifferentColumnLength { len_a: 5, len_b: 10 };
        assert_ne!(mismatch, length);
    }
}
