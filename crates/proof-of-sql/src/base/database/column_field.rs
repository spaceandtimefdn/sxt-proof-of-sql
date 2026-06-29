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
    fn column_field_preserves_name_and_type() {
        let name: Ident = "revenue_cents".into();
        let field = ColumnField::new(name.clone(), ColumnType::BigInt);

        assert_eq!(field.name(), name);
        assert_eq!(field.data_type(), ColumnType::BigInt);

        let renamed = field.name();
        assert_eq!(renamed.value, "revenue_cents");
        assert_eq!(field.name().value, "revenue_cents");
    }
}
