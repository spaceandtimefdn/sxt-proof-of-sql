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
    use alloc::string::ToString;

    #[test]
    fn type_cast_error_displays_both_types() {
        let err = OwnedColumnError::TypeCastError {
            from_type: ColumnType::BigInt,
            to_type: ColumnType::Boolean,
        };
        let msg = err.to_string();
        assert!(msg.contains("BigInt"));
        assert!(msg.contains("Boolean"));
        assert!(msg.contains("Can not perform type casting"));
    }

    #[test]
    fn scalar_conversion_error_displays_message() {
        let err = OwnedColumnError::ScalarConversionError { error: "bad value".to_string() };
        assert!(err.to_string().contains("bad value"));
    }

    #[test]
    fn unsupported_displays_message() {
        let err = OwnedColumnError::Unsupported { error: "op not allowed".to_string() };
        assert!(err.to_string().contains("op not allowed"));
        assert!(err.to_string().contains("Unsupported operation"));
    }

    #[test]
    fn owned_column_errors_implement_partial_eq() {
        assert_eq!(
            OwnedColumnError::Unsupported { error: "x".to_string() },
            OwnedColumnError::Unsupported { error: "x".to_string() }
        );
    }

    #[test]
    fn owned_column_error_debug_contains_variant_name() {
        let debug = format!("{:?}", OwnedColumnError::Unsupported { error: "x".to_string() });
        assert!(debug.contains("Unsupported"));
    }

    #[test]
    fn column_coercion_overflow_displays_correctly() {
        assert_eq!(ColumnCoercionError::Overflow.to_string(), "Overflow when coercing a column");
    }

    #[test]
    fn column_coercion_invalid_type_displays_correctly() {
        assert_eq!(ColumnCoercionError::InvalidTypeCoercion.to_string(), "Invalid type coercion");
    }

    #[test]
    fn column_coercion_errors_implement_partial_eq() {
        assert_eq!(ColumnCoercionError::Overflow, ColumnCoercionError::Overflow);
        assert_ne!(ColumnCoercionError::Overflow, ColumnCoercionError::InvalidTypeCoercion);
    }
}
