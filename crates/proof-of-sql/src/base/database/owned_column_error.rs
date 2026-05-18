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

    #[test]
    fn owned_column_errors_have_descriptive_display_messages() {
        assert_eq!(
            OwnedColumnError::TypeCastError {
                from_type: ColumnType::Boolean,
                to_type: ColumnType::BigInt,
            }
            .to_string(),
            "Can not perform type casting from Boolean to BigInt"
        );
        assert_eq!(
            OwnedColumnError::ScalarConversionError {
                error: "outside target range".into(),
            }
            .to_string(),
            "Error in converting scalars to a given column type: outside target range"
        );
        assert_eq!(
            OwnedColumnError::Unsupported {
                error: "VarBinary arithmetic".into(),
            }
            .to_string(),
            "Unsupported operation: VarBinary arithmetic"
        );
    }

    #[test]
    fn column_coercion_errors_have_descriptive_display_messages() {
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
