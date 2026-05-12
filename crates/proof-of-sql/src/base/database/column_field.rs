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
    use alloc::string::String;

    #[test]
    fn we_can_create_and_read_column_fields() {
        let field = ColumnField::new(Ident::new("amount"), ColumnType::BigInt);

        assert_eq!(field.name().value, "amount");
        assert_eq!(field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_field_name_accessor_returns_a_clone() {
        let field = ColumnField::new(Ident::new("customer_id"), ColumnType::VarChar);
        let mut cloned_name = field.name();
        cloned_name.value = String::from("changed");

        assert_eq!(field.name().value, "customer_id");
        assert_eq!(cloned_name.value, "changed");
    }

    #[test]
    fn column_fields_round_trip_through_serde_json() {
        let field = ColumnField::new(Ident::new("is_active"), ColumnType::Boolean);

        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: ColumnField = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, field);
    }
}
