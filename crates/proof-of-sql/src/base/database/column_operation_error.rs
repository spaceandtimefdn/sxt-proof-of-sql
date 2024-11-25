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
}

/// Result type for column operations
pub type ColumnOperationResult<T> = Result<T, ColumnOperationError>;
