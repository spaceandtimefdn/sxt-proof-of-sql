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
    use super::ColumnField;
    use crate::base::database::ColumnType;
    use sqlparser::ast::Ident;

    #[test]
    fn new_stores_name() {
        let f = ColumnField::new(Ident::new("mycolumn"), ColumnType::BigInt);
        assert_eq!(f.name().value.as_str(), "mycolumn");
    }

    #[test]
    fn new_stores_data_type() {
        let f = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        assert_eq!(f.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn equality_holds_for_same_values() {
        let a = ColumnField::new(Ident::new("x"), ColumnType::BigInt);
        let b = ColumnField::new(Ident::new("x"), ColumnType::BigInt);
        assert_eq!(a, b);
    }

    #[test]
    fn inequality_for_different_type() {
        let a = ColumnField::new(Ident::new("x"), ColumnType::BigInt);
        let b = ColumnField::new(Ident::new("x"), ColumnType::Boolean);
        assert_ne!(a, b);
    }

    #[test]
    fn clone_produces_equal_value() {
        let f = ColumnField::new(Ident::new("col"), ColumnType::BigInt);
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn debug_contains_struct_name() {
        let f = ColumnField::new(Ident::new("col"), ColumnType::BigInt);
        assert!(alloc::format!("{f:?}").contains("ColumnField"));
    }
}
