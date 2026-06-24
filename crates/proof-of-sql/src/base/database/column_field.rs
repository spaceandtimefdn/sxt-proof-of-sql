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
    use super::{ColumnField, ColumnType};
    use sqlparser::ast::Ident;

    #[test]
    fn column_field_name_returns_correct_ident() {
        let ident = Ident::new("my_column");
        let field = ColumnField::new(ident.clone(), ColumnType::BigInt);
        assert_eq!(field.name(), ident);
    }

    #[test]
    fn column_field_data_type_returns_correct_type() {
        let field = ColumnField::new(Ident::new("col"), ColumnType::BigInt);
        assert_eq!(field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_field_boolean_type() {
        let field = ColumnField::new(Ident::new("flag"), ColumnType::Boolean);
        assert_eq!(field.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn column_field_varchar_type() {
        let field = ColumnField::new(Ident::new("name"), ColumnType::VarChar);
        assert_eq!(field.data_type(), ColumnType::VarChar);
    }

    #[test]
    fn column_field_int128_type() {
        let field = ColumnField::new(Ident::new("big"), ColumnType::Int128);
        assert_eq!(field.data_type(), ColumnType::Int128);
    }

    #[test]
    fn column_field_equality_same_fields() {
        let f1 = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
        let f2 = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
        assert_eq!(f1, f2);
    }

    #[test]
    fn column_field_inequality_different_name() {
        let f1 = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
        let f2 = ColumnField::new(Ident::new("b"), ColumnType::BigInt);
        assert_ne!(f1, f2);
    }

    #[test]
    fn column_field_inequality_different_type() {
        let f1 = ColumnField::new(Ident::new("col"), ColumnType::BigInt);
        let f2 = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        assert_ne!(f1, f2);
    }

    #[test]
    fn column_field_clone_equals_original() {
        let f = ColumnField::new(Ident::new("x"), ColumnType::Int128);
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn column_field_debug_contains_name() {
        let f = ColumnField::new(Ident::new("test_col"), ColumnType::BigInt);
        let debug = alloc::format!("{:?}", f);
        assert!(debug.contains("test_col"));
    }

    #[test]
    fn column_field_tinyint_type() {
        let field = ColumnField::new(Ident::new("t"), ColumnType::TinyInt);
        assert_eq!(field.data_type(), ColumnType::TinyInt);
    }

    #[test]
    fn column_field_smallint_type() {
        let field = ColumnField::new(Ident::new("s"), ColumnType::SmallInt);
        assert_eq!(field.data_type(), ColumnType::SmallInt);
    }
}
