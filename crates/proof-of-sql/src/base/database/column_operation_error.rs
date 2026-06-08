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
    use super::ColumnOperationError;
    use crate::base::database::ColumnType;

    #[test]
    fn we_display_column_length_mismatch_errors() {
        let error = ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 };

        assert_eq!(error.to_string(), "Columns have different lengths: 2 != 3");
    }

    #[test]
    fn we_display_invalid_binary_column_type_errors() {
        let error = ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "add".into(),
            left_type: ColumnType::VarChar,
            right_type: ColumnType::BigInt,
        };

        assert_eq!(
            error.to_string(),
            r#""add"(lhs: VarChar, rhs: BigInt) is not supported"#
        );
    }

    #[test]
    fn we_display_invalid_unary_column_type_errors() {
        let error = ColumnOperationError::UnaryOperationInvalidColumnType {
            operator: "negation".into(),
            operand_type: ColumnType::VarBinary,
        };

        assert_eq!(
            error.to_string(),
            r#""negation"(operand: VarBinary) is not supported"#
        );
    }

    #[test]
    fn we_display_column_operation_runtime_errors() {
        assert_eq!(
            ColumnOperationError::IntegerOverflow {
                error: "i64 addition overflow".into()
            }
            .to_string(),
            "Overflow in integer operation: i64 addition overflow"
        );
        assert_eq!(
            ColumnOperationError::DivisionByZero.to_string(),
            "Division by zero"
        );
        assert_eq!(
            ColumnOperationError::IndexOutOfBounds { index: 5, len: 4 }.to_string(),
            "Index out of bounds: 5 >= 4"
        );
    }

    #[test]
    fn we_display_column_casting_errors() {
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
                left_type: ColumnType::Int128,
                right_type: ColumnType::Scalar,
            }
            .to_string(),
            "Cannot fit DECIMAL into SCALAR without losing data"
        );
    }
}
