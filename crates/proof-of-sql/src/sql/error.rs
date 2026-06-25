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
    use alloc::string::String;

    #[test]
    fn invalid_data_type_displays_type() {
        let err = AnalyzeError::InvalidDataType { expr_type: ColumnType::BigInt };
        let msg = err.to_string();
        assert!(msg.contains("BIGINT"));
        assert!(msg.contains("was not valid"));
    }

    #[test]
    fn data_type_mismatch_displays_both_types() {
        let err = AnalyzeError::DataTypeMismatch {
            left_type: "Int".to_string(),
            right_type: "Boolean".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Int"));
        assert!(msg.contains("Boolean"));
    }

    #[test]
    fn different_column_length_displays_lengths() {
        let err = AnalyzeError::DifferentColumnLength { len_a: 5, len_b: 10 };
        assert_eq!(err.to_string(), "Columns have different lengths: 5 != 10");
    }

    #[test]
    fn not_enough_input_plans_displays_correctly() {
        assert_eq!(AnalyzeError::NotEnoughInputPlans.to_string(), "Not enough input plans");
    }

    #[test]
    fn from_impl_converts_to_string_via_display() {
        let err = AnalyzeError::NotEnoughInputPlans;
        let s: String = err.into();
        assert_eq!(s, "Not enough input plans");
    }

    #[test]
    fn analyze_errors_implement_partial_eq() {
        assert_eq!(AnalyzeError::NotEnoughInputPlans, AnalyzeError::NotEnoughInputPlans);
        let e1 = AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 2 };
        let e2 = AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 2 };
        assert_eq!(e1, e2);
    }

    #[test]
    fn analyze_error_debug_contains_variant_name() {
        let debug = format!("{:?}", AnalyzeError::NotEnoughInputPlans);
        assert!(debug.contains("NotEnoughInputPlans"));
    }
}
