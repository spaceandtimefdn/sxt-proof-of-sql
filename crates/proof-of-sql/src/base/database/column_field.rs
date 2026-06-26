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
    fn constructor_stores_name_and_type_metadata() {
        let field = ColumnField::new(Ident::new("is_active"), ColumnType::Boolean);

        assert_eq!(field.name(), Ident::new("is_active"));
        assert_eq!(field.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn column_fields_round_trip_through_serde() {
        let field = ColumnField::new(Ident::new("payload"), ColumnType::VarBinary);
        let json = serde_json::to_string(&field).unwrap();

        assert_eq!(serde_json::from_str::<ColumnField>(&json).unwrap(), field);
    }
}
