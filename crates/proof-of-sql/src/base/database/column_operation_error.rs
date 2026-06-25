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
    fn different_column_length_display_contains_lengths() {
        let e = ColumnOperationError::DifferentColumnLength { len_a: 4, len_b: 7 };
        let s = alloc::format!("{e}");
        assert!(s.contains("4") && s.contains("7"));
    }

    #[test]
    fn binary_operation_invalid_column_type_display() {
        let e = ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "Add".into(),
            left_type: ColumnType::BigInt,
            right_type: ColumnType::VarChar,
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("Add") || s.contains("not supported"));
    }

    #[test]
    fn unary_operation_invalid_column_type_display() {
        let e = ColumnOperationError::UnaryOperationInvalidColumnType {
            operator: "Not".into(),
            operand_type: ColumnType::BigInt,
        };
        assert!(alloc::format!("{e}").contains("not supported"));
    }

    #[test]
    fn integer_overflow_display_contains_message() {
        let e = ColumnOperationError::IntegerOverflow { error: "overflow occurred".into() };
        assert!(alloc::format!("{e}").contains("overflow occurred"));
    }

    #[test]
    fn division_by_zero_display() {
        let e = ColumnOperationError::DivisionByZero;
        assert_eq!(alloc::format!("{e}"), "Division by zero");
    }

    #[test]
    fn union_different_types_display() {
        let e = ColumnOperationError::UnionDifferentTypes {
            correct_type: ColumnType::BigInt,
            actual_type: ColumnType::Boolean,
        };
        assert!(alloc::format!("{e}").contains("Cannot union"));
    }

    #[test]
    fn index_out_of_bounds_display_contains_index_and_len() {
        let e = ColumnOperationError::IndexOutOfBounds { index: 10, len: 5 };
        let s = alloc::format!("{e}");
        assert!(s.contains("10") && s.contains("5"));
    }

    #[test]
    fn signed_casting_error_display() {
        let e = ColumnOperationError::SignedCastingError {
            left_type: ColumnType::TinyInt,
            right_type: ColumnType::BigInt,
        };
        assert!(alloc::format!("{e}").contains("Cannot fit") || alloc::format!("{e}").contains("losing"));
    }

    #[test]
    fn casting_error_display() {
        let e = ColumnOperationError::CastingError {
            left_type: ColumnType::BigInt,
            right_type: ColumnType::SmallInt,
        };
        assert!(alloc::format!("{e}").contains("Cannot fit") || alloc::format!("{e}").contains("losing"));
    }

    #[test]
    fn division_by_zero_equality() {
        assert_eq!(ColumnOperationError::DivisionByZero, ColumnOperationError::DivisionByZero);
    }

    #[test]
    fn different_variants_are_not_equal() {
        assert_ne!(ColumnOperationError::DivisionByZero, ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 });
    }

    #[test]
    fn debug_formatting_contains_variant_name() {
        let e = ColumnOperationError::DivisionByZero;
        assert!(alloc::format!("{e:?}").contains("DivisionByZero"));
    }
}
