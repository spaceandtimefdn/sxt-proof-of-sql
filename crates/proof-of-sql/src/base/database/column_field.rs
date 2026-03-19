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
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn we_can_create_and_access_column_field() {
        let field = ColumnField::new("age".into(), ColumnType::Int);
        assert_eq!(field.name(), "age".into());
        assert_eq!(field.data_type(), ColumnType::Int);
    }

    #[test]
    fn we_can_create_column_field_with_various_types() {
        let types_and_names = [
            ("bool_col", ColumnType::Boolean),
            ("u8_col", ColumnType::Uint8),
            ("tiny_col", ColumnType::TinyInt),
            ("small_col", ColumnType::SmallInt),
            ("int_col", ColumnType::Int),
            ("big_col", ColumnType::BigInt),
            ("i128_col", ColumnType::Int128),
            ("varchar_col", ColumnType::VarChar),
            ("binary_col", ColumnType::VarBinary),
            ("scalar_col", ColumnType::Scalar),
            (
                "decimal_col",
                ColumnType::Decimal75(Precision::new(38).unwrap(), 10),
            ),
            (
                "ts_col",
                ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc()),
            ),
        ];

        for (name, col_type) in types_and_names {
            let field = ColumnField::new(name.into(), col_type);
            assert_eq!(field.name(), name.into());
            assert_eq!(field.data_type(), col_type);
        }
    }

    #[test]
    fn column_fields_with_same_data_are_equal() {
        let a = ColumnField::new("x".into(), ColumnType::BigInt);
        let b = ColumnField::new("x".into(), ColumnType::BigInt);
        assert_eq!(a, b);
    }

    #[test]
    fn column_fields_with_different_names_are_not_equal() {
        let a = ColumnField::new("x".into(), ColumnType::BigInt);
        let b = ColumnField::new("y".into(), ColumnType::BigInt);
        assert_ne!(a, b);
    }

    #[test]
    fn column_fields_with_different_types_are_not_equal() {
        let a = ColumnField::new("x".into(), ColumnType::BigInt);
        let b = ColumnField::new("x".into(), ColumnType::Int);
        assert_ne!(a, b);
    }

    #[test]
    fn we_can_clone_column_field() {
        let original = ColumnField::new("col".into(), ColumnType::VarChar);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn we_can_serialize_and_deserialize_column_field() {
        let field = ColumnField::new("amount".into(), ColumnType::Int128);
        let json = serde_json::to_string(&field).unwrap();
        let deserialized: ColumnField = serde_json::from_str(&json).unwrap();
        assert_eq!(field, deserialized);
    }

    #[test]
    fn we_can_hash_column_field() {
        use core::hash::{Hash, Hasher};
        let field = ColumnField::new("col".into(), ColumnType::Int);
        let mut hasher = ahash::AHasher::default();
        field.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = ahash::AHasher::default();
        field.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }
}
