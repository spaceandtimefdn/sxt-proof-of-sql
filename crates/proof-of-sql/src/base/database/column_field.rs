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

    fn make_field() -> ColumnField {
        ColumnField::new(Ident::new("my_col"), ColumnType::BigInt)
    }

    #[test]
    fn column_field_name_returns_correct_value() {
        let f = make_field();
        assert_eq!(f.name().value, "my_col");
    }

    #[test]
    fn column_field_data_type_returns_correct_value() {
        let f = make_field();
        assert_eq!(f.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_field_equality() {
        let a = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        let b = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        assert_eq!(a, b);
    }

    #[test]
    fn column_field_inequality_by_name() {
        let a = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
        let b = ColumnField::new(Ident::new("b"), ColumnType::BigInt);
        assert_ne!(a, b);
    }

    #[test]
    fn column_field_inequality_by_type() {
        let a = ColumnField::new(Ident::new("col"), ColumnType::BigInt);
        let b = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
        assert_ne!(a, b);
    }

    #[test]
    fn column_field_clone_equals_original() {
        let f = make_field();
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn column_field_is_debug_formattable() {
        let f = make_field();
        let s = alloc::format!("{f:?}");
        assert!(s.contains("my_col"));
    }

    #[test]
    fn column_field_varchar_type() {
        let f = ColumnField::new(Ident::new("s"), ColumnType::VarChar);
        assert_eq!(f.data_type(), ColumnType::VarChar);
    }
}
