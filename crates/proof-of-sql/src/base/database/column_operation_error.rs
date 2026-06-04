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
    use alloc::string::ToString;

    #[test]
    fn column_operation_errors_display_context() {
        assert_eq!(
            ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 }.to_string(),
            "Columns have different lengths: 2 != 3"
        );
        assert_eq!(
            ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "add".to_string(),
                left_type: ColumnType::Int,
                right_type: ColumnType::VarChar,
            }
            .to_string(),
            "\"add\"(lhs: Int, rhs: VarChar) is not supported"
        );
        assert_eq!(
            ColumnOperationError::UnaryOperationInvalidColumnType {
                operator: "not".to_string(),
                operand_type: ColumnType::Int,
            }
            .to_string(),
            "\"not\"(operand: Int) is not supported"
        );
        assert_eq!(
            ColumnOperationError::IntegerOverflow {
                error: "too large".to_string(),
            }
            .to_string(),
            "Overflow in integer operation: too large"
        );
        assert_eq!(
            ColumnOperationError::DivisionByZero.to_string(),
            "Division by zero"
        );
        assert_eq!(
            ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale {
                    scale: "bad".to_string(),
                },
            }
            .to_string(),
            "Decimal scale is not valid: bad"
        );
        assert_eq!(
            ColumnOperationError::UnionDifferentTypes {
                correct_type: ColumnType::Boolean,
                actual_type: ColumnType::Int,
            }
            .to_string(),
            "Cannot union columns of different types: Boolean and Int"
        );
        assert_eq!(
            ColumnOperationError::IndexOutOfBounds { index: 4, len: 4 }.to_string(),
            "Index out of bounds: 4 >= 4"
        );
    }

    #[test]
    fn casting_errors_display_column_types() {
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
                left_type: ColumnType::VarChar,
                right_type: ColumnType::Int,
            }
            .to_string(),
            "Cannot fit VARCHAR into INT without losing data"
        );
        assert_eq!(
            ColumnOperationError::ScaleCastingError {
                left_type: ColumnType::Int128,
                right_type: ColumnType::Decimal75(
                    crate::base::math::decimal::Precision::new(10).unwrap(),
                    2,
                ),
            }
            .to_string(),
            "Cannot fit DECIMAL into DECIMAL75(PRECISION: 10, SCALE: 2) without losing data"
        );
    }
}
