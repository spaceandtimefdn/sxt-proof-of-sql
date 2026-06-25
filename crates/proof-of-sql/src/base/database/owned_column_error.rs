use crate::base::database::ColumnType;
use alloc::string::String;
use snafu::Snafu;

/// Errors from operations related to `OwnedColumn`s.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum OwnedColumnError {
    /// Can not perform type casting.
    #[snafu(display("Can not perform type casting from {from_type:?} to {to_type:?}"))]
    TypeCastError {
        /// The type from which we are trying to cast.
        from_type: ColumnType,
        /// The type to which we are trying to cast.
        to_type: ColumnType,
    },
    /// Error in converting scalars to a given column type.   
    #[snafu(display("Error in converting scalars to a given column type: {error}"))]
    ScalarConversionError {
        /// The underlying error
        error: String,
    },
    /// Unsupported operation.
    #[snafu(display("Unsupported operation: {error}"))]
    Unsupported {
        /// The underlying error
        error: String,
    },
}

/// Errors that can occur when coercing a column.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub(crate) enum ColumnCoercionError {
    /// Overflow when coercing a column.
    #[snafu(display("Overflow when coercing a column"))]
    Overflow,
    /// Invalid type coercion.
    #[snafu(display("Invalid type coercion"))]
    InvalidTypeCoercion,
}

/// Result type for operations related to `OwnedColumn`s.
pub type OwnedColumnResult<T> = core::result::Result<T, OwnedColumnError>;

#[cfg(test)]
mod tests {
    use super::{ColumnCoercionError, OwnedColumnError};
    use crate::base::database::ColumnType;

    #[test]
    fn type_cast_error_display_mentions_types() {
        let e = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        assert!(alloc::format!("{e}").contains("type casting"));
    }

    #[test]
    fn scalar_conversion_error_display_contains_message() {
        let e = OwnedColumnError::ScalarConversionError { error: "bad value".into() };
        assert!(alloc::format!("{e}").contains("bad value"));
    }

    #[test]
    fn unsupported_display_contains_message() {
        let e = OwnedColumnError::Unsupported { error: "operation x".into() };
        assert!(alloc::format!("{e}").contains("operation x"));
    }

    #[test]
    fn type_cast_error_equality() {
        let e1 = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        let e2 = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        assert_eq!(e1, e2);
    }

    #[test]
    fn debug_contains_variant_name() {
        let e = OwnedColumnError::Unsupported { error: "x".into() };
        assert!(alloc::format!("{e:?}").contains("Unsupported"));
    }

    #[test]
    fn coercion_error_overflow_display() {
        let e = ColumnCoercionError::Overflow;
        assert!(alloc::format!("{e}").contains("Overflow") || alloc::format!("{e}").contains("overflow"));
    }

    #[test]
    fn coercion_error_invalid_type_display() {
        let e = ColumnCoercionError::InvalidTypeCoercion;
        assert!(alloc::format!("{e}").contains("Invalid") || alloc::format!("{e}").contains("coercion"));
    }
}
