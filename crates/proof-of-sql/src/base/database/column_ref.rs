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
    use crate::base::database::ColumnType;

    fn make_column_ref() -> ColumnRef {
        let table_ref: TableRef = "schema.table".parse().unwrap();
        let column_id = Ident::new("col1");
        ColumnRef::new(table_ref, column_id, ColumnType::BigInt)
    }

    #[test]
    fn test_new_and_accessors() {
        let table_ref: TableRef = "schema.table".parse().unwrap();
        let column_id = Ident::new("col1");
        let cr = ColumnRef::new(table_ref.clone(), column_id.clone(), ColumnType::BigInt);
        assert_eq!(cr.table_ref(), table_ref);
        assert_eq!(cr.column_id(), column_id);
        assert_eq!(*cr.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_equality() {
        let a = make_column_ref();
        let b = make_column_ref();
        assert_eq!(a, b);
    }

    #[test]
    fn test_inequality_different_type() {
        let table_ref: TableRef = "schema.table".parse().unwrap();
        let col = Ident::new("col1");
        let a = ColumnRef::new(table_ref.clone(), col.clone(), ColumnType::BigInt);
        let b = ColumnRef::new(table_ref, col, ColumnType::Boolean);
        assert_ne!(a, b);
    }

    #[test]
    fn test_clone() {
        let original = make_column_ref();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;
        let cr = make_column_ref();
        let mut set = HashSet::new();
        set.insert(cr.clone());
        assert!(set.contains(&cr));
    }
}
