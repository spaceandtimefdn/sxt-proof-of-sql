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
    fn owned_column_error_type_cast_display() {
        let e = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("type casting"));
    }

    #[test]
    fn owned_column_error_scalar_conversion_display() {
        let e = OwnedColumnError::ScalarConversionError {
            error: "overflow".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("overflow"));
    }

    #[test]
    fn owned_column_error_unsupported_display() {
        let e = OwnedColumnError::Unsupported {
            error: "not yet".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("not yet"));
    }

    #[test]
    fn owned_column_error_type_cast_equality() {
        let a = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        let b = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn owned_column_error_scalar_conversion_equality() {
        let a = OwnedColumnError::ScalarConversionError {
            error: "test".into(),
        };
        let b = OwnedColumnError::ScalarConversionError {
            error: "test".into(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn owned_column_error_is_debug_formattable() {
        let e = OwnedColumnError::Unsupported {
            error: "test".into(),
        };
        let s = alloc::format!("{e:?}");
        assert!(s.contains("Unsupported"));
    }

    #[test]
    fn column_coercion_error_overflow_display() {
        let e = ColumnCoercionError::Overflow;
        let s = alloc::format!("{e}");
        assert!(s.contains("Overflow"));
    }

    #[test]
    fn column_coercion_error_invalid_type_coercion_display() {
        let e = ColumnCoercionError::InvalidTypeCoercion;
        let s = alloc::format!("{e}");
        assert!(s.contains("Invalid"));
    }

    #[test]
    fn column_coercion_errors_are_not_equal() {
        assert_ne!(
            ColumnCoercionError::Overflow,
            ColumnCoercionError::InvalidTypeCoercion
        );
    }

    #[test]
    fn column_coercion_overflow_equality() {
        assert_eq!(ColumnCoercionError::Overflow, ColumnCoercionError::Overflow);
    }
}
