use crate::base::{database::ColumnType, math::decimal::DecimalError};
use alloc::string::String;
use core::result::Result;
use snafu::Snafu;

/// Errors from operations on columns.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ColumnOperationError {
    /// Two columns do not have the same length
    #[snafu(display("Columns have different lengths: {len_a} != {len_b}"))]
    DifferentColumnLength {
        /// The length of the first column
        len_a: usize,
        /// The length of the second column
        len_b: usize,
    },

    /// Incorrect `ColumnType` in binary operations
    #[snafu(display("{operator:?}(lhs: {left_type:?}, rhs: {right_type:?}) is not supported"))]
    BinaryOperationInvalidColumnType {
        /// Binary operator that caused the error
        operator: String,
        /// `ColumnType` of left operand
        left_type: ColumnType,
        /// `ColumnType` of right operand
        right_type: ColumnType,
    },

    /// Incorrect `ColumnType` in unary operations
    #[snafu(display("{operator:?}(operand: {operand_type:?}) is not supported"))]
    UnaryOperationInvalidColumnType {
        /// Unary operator that caused the error
        operator: String,
        /// `ColumnType` of the operand
        operand_type: ColumnType,
    },

    /// Overflow in integer operations
    #[snafu(display("Overflow in integer operation: {error}"))]
    IntegerOverflow {
        /// The underlying overflow error
        error: String,
    },

    /// Division by zero
    #[snafu(display("Division by zero"))]
    DivisionByZero,

    /// Errors related to decimal operations
    #[snafu(transparent)]
    DecimalConversionError {
        /// The underlying source error
        source: DecimalError,
    },

    /// Errors related to unioning columns of different types
    #[snafu(display(
        "Cannot union columns of different types: {correct_type:?} and {actual_type:?}"
    ))]
    UnionDifferentTypes {
        /// The correct data type
        correct_type: ColumnType,
        /// The type of the column that caused the error
        actual_type: ColumnType,
    },

    /// Errors related to index out of bounds
    #[snafu(display("Index out of bounds: {index} >= {len}"))]
    IndexOutOfBounds {
        /// The index that caused the error
        index: usize,
        /// The length of the column
        len: usize,
    },

    /// Errors related to casting between signed and unsigned types. This error can be
    /// used as a signal that a casting operation is currently unsupported.
    /// For example, an i8 can fit inside of a u8 iff it is greater than zero. The library
    /// needs to have a way to check *and* prove that this condition is true. If the library
    /// does not have a proving mechanism in place for this check, then this error is
    /// then used to indicate that the operation is not supported.
    #[snafu(display("Cannot fit {left_type} into {right_type} without losing data"))]
    SignedCastingError {
        /// `ColumnType` of left operand
        left_type: ColumnType,
        /// `ColumnType` of right operand
        right_type: ColumnType,
    },

    /// Errors related to casting between two types.
    #[snafu(display("Cannot fit {left_type} into {right_type} without losing data"))]
    CastingError {
        /// `ColumnType` of left operand
        left_type: ColumnType,
        /// `ColumnType` of right operand
        right_type: ColumnType,
    },

    /// Errors related to casting with scaling between two types.
    #[snafu(display("Cannot fit {left_type} into {right_type} without losing data"))]
    ScaleCastingError {
        /// `ColumnType` of left operand
        left_type: ColumnType,
        /// `ColumnType` of right operand
        right_type: ColumnType,
    },
}

/// Result type for column operations
pub type ColumnOperationResult<T> = Result<T, ColumnOperationError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::DecimalError;

    #[test]
    fn we_can_display_column_operation_errors() {
        assert_eq!(
            ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 }.to_string(),
            "Columns have different lengths: 2 != 3"
        );
        assert_eq!(
            ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "add".into(),
                left_type: ColumnType::BigInt,
                right_type: ColumnType::VarChar,
            }
            .to_string(),
            "\"add\"(lhs: BigInt, rhs: VarChar) is not supported"
        );
        assert_eq!(
            ColumnOperationError::UnaryOperationInvalidColumnType {
                operator: "neg".into(),
                operand_type: ColumnType::Boolean,
            }
            .to_string(),
            "\"neg\"(operand: Boolean) is not supported"
        );
        assert_eq!(
            ColumnOperationError::IntegerOverflow {
                error: "too large".into(),
            }
            .to_string(),
            "Overflow in integer operation: too large"
        );
        assert_eq!(
            ColumnOperationError::IndexOutOfBounds { index: 7, len: 4 }.to_string(),
            "Index out of bounds: 7 >= 4"
        );
    }

    #[test]
    fn we_can_display_type_conversion_column_operation_errors() {
        assert_eq!(
            ColumnOperationError::UnionDifferentTypes {
                correct_type: ColumnType::Int,
                actual_type: ColumnType::BigInt,
            }
            .to_string(),
            "Cannot union columns of different types: Int and BigInt"
        );
        assert_eq!(
            ColumnOperationError::SignedCastingError {
                left_type: ColumnType::TinyInt,
                right_type: ColumnType::Uint8,
            }
            .to_string(),
            "Cannot fit TINYINT into UINT8 without losing data"
        );
        assert_eq!(
            ColumnOperationError::CastingError {
                left_type: ColumnType::BigInt,
                right_type: ColumnType::Int,
            }
            .to_string(),
            "Cannot fit BIGINT into INT without losing data"
        );
        assert_eq!(
            ColumnOperationError::ScaleCastingError {
                left_type: ColumnType::Int,
                right_type: ColumnType::Decimal75(
                    crate::base::math::decimal::Precision::new(10).unwrap(),
                    2
                ),
            }
            .to_string(),
            "Cannot fit INT into DECIMAL75(PRECISION: 10, SCALE: 2) without losing data"
        );
    }

    #[test]
    fn we_can_display_transparent_decimal_operation_errors() {
        let error = ColumnOperationError::DecimalConversionError {
            source: DecimalError::InvalidScale {
                scale: "128".into(),
            },
        };

        assert_eq!(error.to_string(), "Decimal scale is not valid: 128");
    }
}
