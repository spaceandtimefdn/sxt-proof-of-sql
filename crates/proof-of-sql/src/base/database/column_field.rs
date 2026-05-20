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
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_create_and_read_column_fields() {
        let column_field = ColumnField::new(Ident::new("order_id"), ColumnType::BigInt);

        assert_eq!(column_field.name().value.as_str(), "order_id");
        assert_eq!(column_field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_clone_and_compare_column_fields() {
        let column_field = ColumnField::new(Ident::new("amount"), ColumnType::Int128);

        assert_eq!(column_field.clone(), column_field);
    }

    #[test]
    fn we_can_serialize_and_deserialize_column_fields() {
        let column_field = ColumnField::new(Ident::new("is_paid"), ColumnType::Boolean);

        let encoded = serde_json::to_string(&column_field).unwrap();
        let decoded = serde_json::from_str::<ColumnField>(&encoded).unwrap();

        assert_eq!(decoded, column_field);
    }
}
