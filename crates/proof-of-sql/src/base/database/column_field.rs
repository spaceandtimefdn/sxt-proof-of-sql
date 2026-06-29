use super::ColumnType;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// This type is used to represent the metadata
/// of a column in a table. Namely: it's name and type.
///
/// This is the analog of a `Field` in Apache Arrow.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnField {
    name: Ident,
    data_type: ColumnType,
    #[serde(default, skip_serializing_if = "is_false")]
    nullable: bool,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde skip_serializing_if requires a by-reference predicate"
)]
fn is_false(value: &bool) -> bool {
    !*value
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField {
            name,
            data_type,
            nullable: false,
        }
    }

    /// Create a new nullable `ColumnField` from a name and a type.
    #[must_use]
    pub fn new_nullable(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField {
            name,
            data_type,
            nullable: true,
        }
    }

    /// Returns the name of the column
    #[must_use]
    pub fn name(&self) -> Ident {
        self.name.clone()
    }

    /// Returns the type of the column
    #[must_use]
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }

    /// Returns true when the column is nullable.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn non_nullable_column_fields_do_not_serialize_nullable_false() {
        let field = ColumnField::new("score".into(), ColumnType::BigInt);

        let json = serde_json::to_value(&field).unwrap();

        assert!(json.get("nullable").is_none());
    }

    #[test]
    fn column_fields_without_nullable_metadata_deserialize_as_non_nullable() {
        let field = ColumnField::new("score".into(), ColumnType::BigInt);
        let mut json = serde_json::to_value(&field).unwrap();
        let Value::Object(ref mut object) = json else {
            panic!("ColumnField should serialize as an object");
        };
        object.remove("nullable");

        let field_without_nullable: ColumnField = serde_json::from_value(json).unwrap();

        assert!(!field_without_nullable.is_nullable());
        assert_eq!(field_without_nullable.name(), "score".into());
        assert_eq!(field_without_nullable.data_type(), ColumnType::BigInt);
    }
}
