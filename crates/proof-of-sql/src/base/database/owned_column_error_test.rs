use super::{ColumnCoercionError, ColumnType, OwnedColumnError};
use alloc::string::{String, ToString};

#[test]
fn owned_column_error_messages_are_formatted_as_expected() {
    let cast = OwnedColumnError::TypeCastError {
        from_type: ColumnType::Boolean,
        to_type: ColumnType::BigInt,
    };
    assert_eq!(
        cast.to_string(),
        "Can not perform type casting from Boolean to BigInt"
    );

    let scalar = OwnedColumnError::ScalarConversionError {
        error: String::from("overflow"),
    };
    assert_eq!(
        scalar.to_string(),
        "Error in converting scalars to a given column type: overflow"
    );

    let unsupported = OwnedColumnError::Unsupported {
        error: String::from("nope"),
    };
    assert_eq!(unsupported.to_string(), "Unsupported operation: nope");
}

#[test]
fn owned_column_error_equality_depends_on_fields() {
    let a = OwnedColumnError::TypeCastError {
        from_type: ColumnType::Boolean,
        to_type: ColumnType::BigInt,
    };
    let same = OwnedColumnError::TypeCastError {
        from_type: ColumnType::Boolean,
        to_type: ColumnType::BigInt,
    };
    let different = OwnedColumnError::TypeCastError {
        from_type: ColumnType::Boolean,
        to_type: ColumnType::Int,
    };
    assert_eq!(a, same);
    assert_ne!(a, different);
    assert_ne!(
        a,
        OwnedColumnError::Unsupported {
            error: String::from("x")
        }
    );
}

#[test]
fn column_coercion_error_messages_are_formatted_as_expected() {
    assert_eq!(
        ColumnCoercionError::Overflow.to_string(),
        "Overflow when coercing a column"
    );
    assert_eq!(
        ColumnCoercionError::InvalidTypeCoercion.to_string(),
        "Invalid type coercion"
    );
    assert_ne!(
        ColumnCoercionError::Overflow,
        ColumnCoercionError::InvalidTypeCoercion
    );
}
