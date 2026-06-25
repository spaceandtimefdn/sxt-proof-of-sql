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

    /// Returns whether the column may contain null values.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_columns_are_not_nullable_by_default() {
        let column_field = ColumnField::new("amount".into(), ColumnType::BigInt);

        assert_eq!(column_field.name(), "amount".into());
        assert_eq!(column_field.data_type(), ColumnType::BigInt);
        assert!(!column_field.is_nullable());
    }

    #[test]
    fn nullable_columns_are_marked_nullable() {
        let column_field = ColumnField::new_nullable("amount".into(), ColumnType::BigInt);

        assert_eq!(column_field.name(), "amount".into());
        assert_eq!(column_field.data_type(), ColumnType::BigInt);
        assert!(column_field.is_nullable());
    }

    #[test]
    fn serde_defaults_missing_nullable_to_false() {
        let json = r#"{"name":{"value":"amount","quote_style":null},"data_type":"BIGINT"}"#;
        let column_field: ColumnField = serde_json::from_str(json).unwrap();

        assert_eq!(column_field.name(), "amount".into());
        assert_eq!(column_field.data_type(), ColumnType::BigInt);
        assert!(!column_field.is_nullable());
    }
}
