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
    #[serde(default)]
    nullable: bool,
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

    /// Returns whether the column is nullable.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_field_defaults_to_non_nullable() {
        let field = ColumnField::new("score".into(), ColumnType::BigInt);

        assert!(!field.is_nullable());
    }

    #[test]
    fn column_field_can_be_explicitly_nullable() {
        let field = ColumnField::new_nullable("score".into(), ColumnType::BigInt);

        assert!(field.is_nullable());
    }

    #[test]
    fn column_field_serde_is_backward_compatible_without_nullable_flag() {
        let field: ColumnField = serde_json::from_str(
            r#"{"name":{"value":"score","quote_style":null},"data_type":"BigInt"}"#,
        )
        .unwrap();

        assert_eq!(field.name(), Ident::new("score"));
        assert_eq!(field.data_type(), ColumnType::BigInt);
        assert!(!field.is_nullable());
    }
}
