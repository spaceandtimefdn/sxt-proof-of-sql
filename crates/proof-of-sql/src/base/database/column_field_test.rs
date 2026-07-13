//! Tests for ColumnField.

#[cfg(test)]
mod column_field_test {
    use crate::base::database::{ColumnField, ColumnType};
    use alloc::string::ToString;

    #[test]
    fn test_column_field_new() {
        let field = ColumnField::new("col1".parse().unwrap(), ColumnType::BigInt);
        assert_eq!(field.name().to_string(), "col1");
        assert_eq!(field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_column_field_clone() {
        let field = ColumnField::new("col".parse().unwrap(), ColumnType::Int);
        let cloned = field.clone();
        assert_eq!(field, cloned);
    }

    #[test]
    fn test_column_field_debug() {
        let field = ColumnField::new("test".parse().unwrap(), ColumnType::Boolean);
        let debug_str = format!("{:?}", field);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_column_field_partial_eq() {
        let field1 = ColumnField::new("col".parse().unwrap(), ColumnType::BigInt);
        let field2 = ColumnField::new("col".parse().unwrap(), ColumnType::BigInt);
        let field3 = ColumnField::new("col".parse().unwrap(), ColumnType::Int);
        assert_eq!(field1, field2);
        assert_ne!(field1, field3);
    }

    #[test]
    fn test_column_field_hash() {
        use core::hash::{Hash, Hasher};
        let field1 = ColumnField::new("col".parse().unwrap(), ColumnType::BigInt);
        let field2 = ColumnField::new("col".parse().unwrap(), ColumnType::BigInt);
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        field1.hash(&mut h1);
        field2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_column_field_with_different_types() {
        let types = vec![
            ColumnType::BigInt,
            ColumnType::Int,
            ColumnType::SmallInt,
            ColumnType::TinyInt,
            ColumnType::Uint8,
            ColumnType::Boolean,
            ColumnType::VarChar,
            ColumnType::Scalar,
        ];
        for ty in types {
            let field = ColumnField::new("col".parse().unwrap(), ty);
            assert_eq!(field.data_type(), ty);
        }
    }

    #[test]
    fn test_column_field_serialize() {
        let field = ColumnField::new("test".parse().unwrap(), ColumnType::BigInt);
        let serialized = serde_json::to_string(&field).unwrap();
        assert!(!serialized.is_empty());
    }
}