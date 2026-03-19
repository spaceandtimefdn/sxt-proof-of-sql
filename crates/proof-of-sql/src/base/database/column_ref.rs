use super::{ColumnType, TableRef};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnRef {
    column_id: Ident,
    table_ref: TableRef,
    column_type: ColumnType,
}

impl ColumnRef {
    /// Create a new `ColumnRef` from a table, column identifier and column type
    #[must_use]
    pub fn new(table_ref: TableRef, column_id: Ident, column_type: ColumnType) -> Self {
        Self {
            column_id,
            table_ref,
            column_type,
        }
    }

    /// Returns the table reference of this column
    #[must_use]
    pub fn table_ref(&self) -> TableRef {
        self.table_ref.clone()
    }

    /// Returns the column identifier of this column
    #[must_use]
    pub fn column_id(&self) -> Ident {
        self.column_id.clone()
    }

    /// Returns the column type of this column
    #[must_use]
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_create_and_access_column_ref() {
        let table_ref = TableRef::new("schema", "table");
        let column_ref = ColumnRef::new(table_ref.clone(), "col_a".into(), ColumnType::BigInt);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), "col_a".into());
        assert_eq!(*column_ref.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_create_column_ref_with_various_types() {
        let table_ref = TableRef::new("ns", "tbl");

        for column_type in [
            ColumnType::Boolean,
            ColumnType::Uint8,
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
            ColumnType::VarChar,
            ColumnType::VarBinary,
            ColumnType::Scalar,
        ] {
            let col_ref = ColumnRef::new(table_ref.clone(), "c".into(), column_type);
            assert_eq!(*col_ref.column_type(), column_type);
        }
    }

    #[test]
    fn column_refs_with_same_data_are_equal() {
        let a = ColumnRef::new(
            TableRef::new("s", "t"),
            "x".into(),
            ColumnType::Int,
        );
        let b = ColumnRef::new(
            TableRef::new("s", "t"),
            "x".into(),
            ColumnType::Int,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn column_refs_with_different_names_are_not_equal() {
        let a = ColumnRef::new(
            TableRef::new("s", "t"),
            "x".into(),
            ColumnType::Int,
        );
        let b = ColumnRef::new(
            TableRef::new("s", "t"),
            "y".into(),
            ColumnType::Int,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn column_refs_with_different_types_are_not_equal() {
        let a = ColumnRef::new(
            TableRef::new("s", "t"),
            "x".into(),
            ColumnType::Int,
        );
        let b = ColumnRef::new(
            TableRef::new("s", "t"),
            "x".into(),
            ColumnType::BigInt,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn column_refs_with_different_tables_are_not_equal() {
        let a = ColumnRef::new(
            TableRef::new("s", "t1"),
            "x".into(),
            ColumnType::Int,
        );
        let b = ColumnRef::new(
            TableRef::new("s", "t2"),
            "x".into(),
            ColumnType::Int,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn we_can_clone_column_ref() {
        let original = ColumnRef::new(
            TableRef::new("schema", "table"),
            "col".into(),
            ColumnType::VarChar,
        );
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn we_can_hash_column_ref() {
        use core::hash::{Hash, Hasher};
        let col_ref = ColumnRef::new(
            TableRef::new("s", "t"),
            "c".into(),
            ColumnType::Int,
        );
        let mut hasher = ahash::AHasher::default();
        col_ref.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = ahash::AHasher::default();
        col_ref.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }
}
