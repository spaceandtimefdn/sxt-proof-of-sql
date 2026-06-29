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
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_type_cast_error_display_includes_column_types() {
        let error = OwnedColumnError::TypeCastError {
            from_type: ColumnType::Int,
            to_type: ColumnType::BigInt,
        };

        assert_eq!(
            error.to_string(),
            "Can not perform type casting from Int to BigInt"
        );
    }

    #[test]
    fn test_scalar_conversion_error_display_includes_source_error() {
        let error = OwnedColumnError::ScalarConversionError {
            error: "invalid scalar".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Error in converting scalars to a given column type: invalid scalar"
        );
    }

    #[test]
    fn test_unsupported_error_display_includes_source_error() {
        let error = OwnedColumnError::Unsupported {
            error: "nested arrays".to_string(),
        };

        assert_eq!(error.to_string(), "Unsupported operation: nested arrays");
    }

    #[test]
    fn test_column_coercion_error_display_messages() {
        assert_eq!(
            ColumnCoercionError::Overflow.to_string(),
            "Overflow when coercing a column"
        );
        assert_eq!(
            ColumnCoercionError::InvalidTypeCoercion.to_string(),
            "Invalid type coercion"
        );
    }
}
