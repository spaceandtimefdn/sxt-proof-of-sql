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
    fn column_field_accessors_return_expected_owned_metadata() {
        let column_field = ColumnField::new(Ident::new("customer_id"), ColumnType::Int);

        let mut returned_name = column_field.name();
        returned_name.value = "mutated".into();

        assert_eq!(column_field.name(), Ident::new("customer_id"));
        assert_eq!(column_field.data_type(), ColumnType::Int);
    }

    #[test]
    fn column_field_round_trips_through_json() {
        let column_field = ColumnField::new(Ident::new("region"), ColumnType::VarChar);

        let serialized = serde_json::to_string(&column_field).unwrap();
        let deserialized: ColumnField = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, column_field);
    }
}
