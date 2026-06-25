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
    use std::collections::HashSet;

    #[test]
    fn we_can_build_and_query_column_refs() {
        let table_ref = TableRef::new("public", "employees");
        let column_ref =
            ColumnRef::new(table_ref.clone(), Ident::new("salary"), ColumnType::BigInt);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), Ident::new("salary"));
        assert_eq!(column_ref.column_type(), &ColumnType::BigInt);
    }

    #[test]
    fn we_can_hash_compare_and_serialize_column_refs() {
        let first = ColumnRef::new(
            TableRef::new("public", "employees"),
            Ident::new("id"),
            ColumnType::Int,
        );
        let same = ColumnRef::new(
            TableRef::new("public", "employees"),
            Ident::new("id"),
            ColumnType::Int,
        );
        let different = ColumnRef::new(
            TableRef::new("public", "employees"),
            Ident::new("department_id"),
            ColumnType::Int,
        );

        assert_eq!(first, same);
        assert_ne!(first, different);

        let mut set = HashSet::new();
        set.insert(first.clone());
        assert!(set.contains(&same));
        assert!(!set.contains(&different));

        let json = serde_json::to_string(&first).unwrap();
        let round_trip: ColumnRef = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, first);
    }
}
