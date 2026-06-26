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
    use crate::base::{
        database::ColumnType,
        math::decimal::{DecimalError, IntermediateDecimalError},
    };

    #[test]
    fn we_can_format_invalid_data_type_error() {
        let err = AnalyzeError::InvalidDataType {
            expr_type: ColumnType::Boolean,
        };
        assert_eq!(
            err.to_string(),
            "Expression has datatype BOOLEAN, which was not valid"
        );
    }

    #[test]
    fn we_can_format_invalid_data_type_error_for_each_simple_type() {
        let cases = [
            (ColumnType::Uint8, "UINT8"),
            (ColumnType::TinyInt, "TINYINT"),
            (ColumnType::SmallInt, "SMALLINT"),
            (ColumnType::Int, "INT"),
            (ColumnType::BigInt, "BIGINT"),
            (ColumnType::Int128, "DECIMAL"),
            (ColumnType::VarChar, "VARCHAR"),
            (ColumnType::VarBinary, "BINARY"),
            (ColumnType::Scalar, "SCALAR"),
        ];
        for (col_type, type_name) in cases {
            let err = AnalyzeError::InvalidDataType { expr_type: col_type };
            assert_eq!(
                err.to_string(),
                format!("Expression has datatype {type_name}, which was not valid")
            );
        }
    }

    #[test]
    fn we_can_format_data_type_mismatch_error() {
        let err = AnalyzeError::DataTypeMismatch {
            left_type: "BIGINT".into(),
            right_type: "BOOLEAN".into(),
        };
        assert_eq!(
            err.to_string(),
            "Left side has 'BIGINT' type but right side has 'BOOLEAN' type"
        );
    }

    #[test]
    fn we_can_format_data_type_mismatch_with_empty_strings() {
        let err = AnalyzeError::DataTypeMismatch {
            left_type: String::new(),
            right_type: String::new(),
        };
        assert_eq!(
            err.to_string(),
            "Left side has '' type but right side has '' type"
        );
    }

    #[test]
    fn we_can_format_different_column_length_error() {
        let err = AnalyzeError::DifferentColumnLength { len_a: 5, len_b: 7 };
        assert_eq!(err.to_string(), "Columns have different lengths: 5 != 7");
    }

    #[test]
    fn we_can_format_different_column_length_error_with_zero() {
        let err = AnalyzeError::DifferentColumnLength { len_a: 0, len_b: 0 };
        assert_eq!(err.to_string(), "Columns have different lengths: 0 != 0");
    }

    #[test]
    fn we_can_format_not_enough_input_plans_error() {
        let err = AnalyzeError::NotEnoughInputPlans;
        assert_eq!(err.to_string(), "Not enough input plans");
    }

    #[test]
    fn we_can_convert_analyze_error_to_string_via_from() {
        let err = AnalyzeError::NotEnoughInputPlans;
        let s: String = String::from(err);
        assert_eq!(s, "Not enough input plans");
    }

    #[test]
    fn we_can_convert_analyze_error_to_string_via_into() {
        let err = AnalyzeError::DifferentColumnLength { len_a: 3, len_b: 4 };
        let s: String = err.into();
        assert_eq!(s, "Columns have different lengths: 3 != 4");
    }

    #[test]
    fn we_can_convert_intermediate_decimal_error_to_analyze_error() {
        let err: AnalyzeError = IntermediateDecimalError::OutOfRange.into();
        // The conversion wraps it under DecimalConversionError → IntermediateDecimalConversionError.
        assert!(matches!(
            err,
            AnalyzeError::DecimalConversionError {
                source: DecimalError::IntermediateDecimalConversionError {
                    source: IntermediateDecimalError::OutOfRange,
                },
            }
        ));
    }

    #[test]
    fn we_can_convert_lossy_cast_intermediate_decimal_error() {
        let err: AnalyzeError = IntermediateDecimalError::LossyCast.into();
        assert!(matches!(
            err,
            AnalyzeError::DecimalConversionError {
                source: DecimalError::IntermediateDecimalConversionError {
                    source: IntermediateDecimalError::LossyCast,
                },
            }
        ));
    }

    #[test]
    fn analyze_errors_with_same_data_are_equal() {
        assert_eq!(
            AnalyzeError::NotEnoughInputPlans,
            AnalyzeError::NotEnoughInputPlans
        );
        assert_eq!(
            AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 2 },
            AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 2 }
        );
        assert_eq!(
            AnalyzeError::InvalidDataType {
                expr_type: ColumnType::Int,
            },
            AnalyzeError::InvalidDataType {
                expr_type: ColumnType::Int,
            }
        );
    }

    #[test]
    fn analyze_errors_with_different_data_are_not_equal() {
        assert_ne!(
            AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 2 },
            AnalyzeError::DifferentColumnLength { len_a: 1, len_b: 3 }
        );
        assert_ne!(
            AnalyzeError::InvalidDataType {
                expr_type: ColumnType::Int,
            },
            AnalyzeError::InvalidDataType {
                expr_type: ColumnType::BigInt,
            }
        );
        assert_ne!(
            AnalyzeError::DataTypeMismatch {
                left_type: "BIGINT".into(),
                right_type: "BOOLEAN".into(),
            },
            AnalyzeError::DataTypeMismatch {
                left_type: "BOOLEAN".into(),
                right_type: "BIGINT".into(),
            }
        );
    }

    #[test]
    fn analyze_errors_with_different_variants_are_not_equal() {
        assert_ne!(
            AnalyzeError::NotEnoughInputPlans,
            AnalyzeError::DifferentColumnLength { len_a: 0, len_b: 0 }
        );
    }
}
