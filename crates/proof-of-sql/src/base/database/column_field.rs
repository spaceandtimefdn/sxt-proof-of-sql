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
    use crate::base::math::decimal::Precision;

    #[test]
    fn we_can_create_column_field_and_read_its_parts() {
        let column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), -2);
        let field = ColumnField::new(Ident::new("amount"), column_type);

        assert_eq!(field.name(), Ident::new("amount"));
        assert_eq!(field.data_type(), column_type);
    }

    #[test]
    fn column_field_serializes_as_name_and_data_type() {
        let field = ColumnField::new(Ident::new("flag"), ColumnType::Boolean);

        let serialized = serde_json::to_string(&field).unwrap();
        let deserialized: ColumnField = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, field);
    }
}
