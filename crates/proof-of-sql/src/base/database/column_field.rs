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
    fn column_field_stores_name_and_type() {
        let field = ColumnField::new(Ident::new("my_col"), ColumnType::BigInt);
        assert_eq!(field.name(), Ident::new("my_col"));
        assert_eq!(field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_field_name_returns_correct_value() {
        let field = ColumnField::new(Ident::new("age"), ColumnType::Int);
        assert_eq!(field.name().value, "age");
    }

    #[test]
    fn column_field_data_type_returns_correct_type() {
        let field = ColumnField::new(Ident::new("flag"), ColumnType::Boolean);
        assert_eq!(field.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn column_field_equality_same_fields() {
        let f1 = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        let f2 = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        assert_eq!(f1, f2);
    }

    #[test]
    fn column_field_inequality_different_name() {
        let f1 = ColumnField::new(Ident::new("col_a"), ColumnType::BigInt);
        let f2 = ColumnField::new(Ident::new("col_b"), ColumnType::BigInt);
        assert_ne!(f1, f2);
    }

    #[test]
    fn column_field_inequality_different_type() {
        let f1 = ColumnField::new(Ident::new("col"), ColumnType::BigInt);
        let f2 = ColumnField::new(Ident::new("col"), ColumnType::Int);
        assert_ne!(f1, f2);
    }

    #[test]
    fn column_field_clone_equals_original() {
        let f = ColumnField::new(Ident::new("x"), ColumnType::VarChar);
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn column_field_debug_output_contains_field_name() {
        let f = ColumnField::new(Ident::new("score"), ColumnType::BigInt);
        let debug = format!("{f:?}");
        assert!(debug.contains("score"));
    }

    #[test]
    fn column_field_int128_type() {
        let field = ColumnField::new(Ident::new("amount"), ColumnType::Int128);
        assert_eq!(field.data_type(), ColumnType::Int128);
    }

    #[test]
    fn column_field_varchar_type() {
        let field = ColumnField::new(Ident::new("name"), ColumnType::VarChar);
        assert_eq!(field.data_type(), ColumnType::VarChar);
        assert_eq!(field.name().value, "name");
    }
}
