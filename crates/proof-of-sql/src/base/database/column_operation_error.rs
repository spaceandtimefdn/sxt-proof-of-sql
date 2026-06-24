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
    use alloc::string::ToString;

    #[test]
    fn different_column_length_displays_both_lengths() {
        let err = ColumnOperationError::DifferentColumnLength { len_a: 3, len_b: 5 };
        assert_eq!(err.to_string(), "Columns have different lengths: 3 != 5");
    }

    #[test]
    fn binary_operation_invalid_type_displays_operator_and_types() {
        let err = ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "Add".to_string(),
            left_type: ColumnType::BigInt,
            right_type: ColumnType::Boolean,
        };
        let msg = err.to_string();
        assert!(msg.contains("Add"));
        assert!(msg.contains("BigInt"));
        assert!(msg.contains("Boolean"));
        assert!(msg.contains("not supported"));
    }

    #[test]
    fn unary_operation_invalid_type_displays_operator_and_operand() {
        let err = ColumnOperationError::UnaryOperationInvalidColumnType {
            operator: "Not".to_string(),
            operand_type: ColumnType::BigInt,
        };
        let msg = err.to_string();
        assert!(msg.contains("Not"));
        assert!(msg.contains("BigInt"));
    }

    #[test]
    fn integer_overflow_displays_error_message() {
        let err = ColumnOperationError::IntegerOverflow { error: "value too large".to_string() };
        assert_eq!(err.to_string(), "Overflow in integer operation: value too large");
    }

    #[test]
    fn division_by_zero_displays_correctly() {
        assert_eq!(ColumnOperationError::DivisionByZero.to_string(), "Division by zero");
    }

    #[test]
    fn union_different_types_displays_both_types() {
        let err = ColumnOperationError::UnionDifferentTypes {
            correct_type: ColumnType::BigInt,
            actual_type: ColumnType::Boolean,
        };
        let msg = err.to_string();
        assert!(msg.contains("BigInt"));
        assert!(msg.contains("Boolean"));
        assert!(msg.contains("Cannot union columns of different types"));
    }

    #[test]
    fn index_out_of_bounds_displays_index_and_len() {
        let err = ColumnOperationError::IndexOutOfBounds { index: 10, len: 5 };
        assert_eq!(err.to_string(), "Index out of bounds: 10 >= 5");
    }

    #[test]
    fn signed_casting_error_displays_types() {
        let err = ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Int,
            right_type: ColumnType::TinyInt,
        };
        let msg = err.to_string();
        assert!(msg.contains("Cannot fit"));
        assert!(msg.contains("without losing data"));
    }

    #[test]
    fn column_operation_errors_implement_partial_eq() {
        assert_eq!(ColumnOperationError::DivisionByZero, ColumnOperationError::DivisionByZero);
        assert_ne!(ColumnOperationError::DivisionByZero,
            ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 });
    }

    #[test]
    fn column_operation_error_debug_contains_variant_name() {
        let debug = format!("{:?}", ColumnOperationError::DivisionByZero);
        assert!(debug.contains("DivisionByZero"));
    }
}
