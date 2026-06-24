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
        TableRef::new("", "orders")
    }

    #[test]
    fn new_stores_all_fields() {
        let col_id = Ident::new("amount");
        let cr = ColumnRef::new(make_table_ref(), col_id.clone(), ColumnType::BigInt);
        assert_eq!(cr.column_id(), col_id);
        assert_eq!(cr.column_type(), &ColumnType::BigInt);
    }

    #[test]
    fn table_ref_returns_stored_table() {
        let tr = make_table_ref();
        let cr = ColumnRef::new(tr.clone(), Ident::new("col"), ColumnType::Boolean);
        assert_eq!(cr.table_ref(), tr);
    }

    #[test]
    fn column_id_returns_identifier() {
        let id = Ident::new("my_col");
        let cr = ColumnRef::new(make_table_ref(), id.clone(), ColumnType::BigInt);
        assert_eq!(cr.column_id(), id);
    }

    #[test]
    fn column_type_returns_reference_to_type() {
        let cr = ColumnRef::new(make_table_ref(), Ident::new("x"), ColumnType::TinyInt);
        assert_eq!(*cr.column_type(), ColumnType::TinyInt);
    }

    #[test]
    fn clone_creates_equal_column_ref() {
        let cr = ColumnRef::new(make_table_ref(), Ident::new("col"), ColumnType::BigInt);
        assert_eq!(cr.clone(), cr);
    }

    #[test]
    fn two_column_refs_with_same_fields_are_equal() {
        let a = ColumnRef::new(make_table_ref(), Ident::new("x"), ColumnType::BigInt);
        let b = ColumnRef::new(make_table_ref(), Ident::new("x"), ColumnType::BigInt);
        assert_eq!(a, b);
    }

    #[test]
    fn two_column_refs_with_different_type_are_not_equal() {
        let a = ColumnRef::new(make_table_ref(), Ident::new("x"), ColumnType::BigInt);
        let b = ColumnRef::new(make_table_ref(), Ident::new("x"), ColumnType::Boolean);
        assert_ne!(a, b);
    }
}
