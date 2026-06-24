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
    use super::ColumnRef;
    use crate::base::database::{ColumnType, TableRef};
    use sqlparser::ast::Ident;

    fn make_table_ref() -> TableRef {
        TableRef::new("myschema", "mytable")
    }

    #[test]
    fn new_stores_all_fields() {
        let tr = make_table_ref();
        let col = ColumnRef::new(tr.clone(), Ident::new("my_col"), ColumnType::BigInt);
        assert_eq!(col.table_ref(), tr);
        assert_eq!(col.column_id(), Ident::new("my_col"));
        assert_eq!(col.column_type(), &ColumnType::BigInt);
    }

    #[test]
    fn table_ref_returns_correct_value() {
        let tr = TableRef::new("s", "t");
        let col = ColumnRef::new(tr.clone(), Ident::new("x"), ColumnType::Int);
        assert_eq!(col.table_ref(), tr);
    }

    #[test]
    fn column_id_returns_correct_identifier() {
        let col = ColumnRef::new(make_table_ref(), Ident::new("salary"), ColumnType::BigInt);
        assert_eq!(col.column_id().value, "salary");
    }

    #[test]
    fn column_type_returns_correct_type() {
        let col = ColumnRef::new(make_table_ref(), Ident::new("flag"), ColumnType::Boolean);
        assert_eq!(col.column_type(), &ColumnType::Boolean);
    }

    #[test]
    fn column_ref_equality() {
        let c1 = ColumnRef::new(make_table_ref(), Ident::new("col"), ColumnType::VarChar);
        let c2 = ColumnRef::new(make_table_ref(), Ident::new("col"), ColumnType::VarChar);
        assert_eq!(c1, c2);
    }

    #[test]
    fn column_ref_inequality_different_column_id() {
        let c1 = ColumnRef::new(make_table_ref(), Ident::new("col_a"), ColumnType::BigInt);
        let c2 = ColumnRef::new(make_table_ref(), Ident::new("col_b"), ColumnType::BigInt);
        assert_ne!(c1, c2);
    }

    #[test]
    fn column_ref_inequality_different_type() {
        let c1 = ColumnRef::new(make_table_ref(), Ident::new("col"), ColumnType::BigInt);
        let c2 = ColumnRef::new(make_table_ref(), Ident::new("col"), ColumnType::Int);
        assert_ne!(c1, c2);
    }

    #[test]
    fn column_ref_clone_equals_original() {
        let c = ColumnRef::new(make_table_ref(), Ident::new("x"), ColumnType::Boolean);
        assert_eq!(c.clone(), c);
    }

    #[test]
    fn column_ref_debug_contains_column_name() {
        let c = ColumnRef::new(make_table_ref(), Ident::new("myfield"), ColumnType::BigInt);
        let debug = format!("{c:?}");
        assert!(debug.contains("myfield"));
    }
}
