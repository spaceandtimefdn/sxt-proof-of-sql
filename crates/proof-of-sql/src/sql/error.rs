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

    #[test]
    fn invalid_data_type_display_contains_type() {
        let e = AnalyzeError::InvalidDataType { expr_type: ColumnType::BigInt };
        assert!(alloc::format!("{e}").contains("BIGINT") || alloc::format!("{e}").contains("BigInt"));
    }

    #[test]
    fn data_type_mismatch_display_contains_both_types() {
        let e = AnalyzeError::DataTypeMismatch {
            left_type: "BIGINT".into(),
            right_type: "BOOLEAN".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("BIGINT") && s.contains("BOOLEAN"));
    }

    #[test]
    fn different_column_length_display_contains_lengths() {
        let e = AnalyzeError::DifferentColumnLength { len_a: 5, len_b: 3 };
        let s = alloc::format!("{e}");
        assert!(s.contains("5") && s.contains("3"));
    }

    #[test]
    fn not_enough_input_plans_display() {
        let e = AnalyzeError::NotEnoughInputPlans;
        assert!(alloc::format!("{e}").contains("input") || alloc::format!("{e}").contains("plans"));
    }

    #[test]
    fn analyze_error_debug_for_invalid_data_type() {
        let e = AnalyzeError::InvalidDataType { expr_type: ColumnType::Boolean };
        assert!(alloc::format!("{e:?}").contains("InvalidDataType"));
    }

    #[test]
    fn data_type_mismatch_equality() {
        let e1 = AnalyzeError::DataTypeMismatch {
            left_type: "A".into(),
            right_type: "B".into(),
        };
        let e2 = AnalyzeError::DataTypeMismatch {
            left_type: "A".into(),
            right_type: "B".into(),
        };
        assert_eq!(e1, e2);
    }

    #[test]
    fn not_enough_input_plans_inequality_with_different_variant() {
        assert_ne!(
            AnalyzeError::NotEnoughInputPlans,
            AnalyzeError::InvalidDataType { expr_type: ColumnType::BigInt },
        );
    }

    #[test]
    fn analyze_error_from_converts_to_string() {
        let e = AnalyzeError::NotEnoughInputPlans;
        let s: alloc::string::String = e.into();
        assert!(!s.is_empty());
    }
}
