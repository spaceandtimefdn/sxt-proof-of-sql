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

    fn make_ref() -> ColumnRef {
        let table = TableRef::new("schema", "table");
        ColumnRef::new(table, Ident::new("col"), ColumnType::BigInt)
    }

    #[test]
    fn column_ref_table_ref_returns_correct_value() {
        let r = make_ref();
        assert_eq!(r.table_ref(), TableRef::new("schema", "table"));
    }

    #[test]
    fn column_ref_column_id_returns_correct_value() {
        let r = make_ref();
        assert_eq!(r.column_id().value, "col");
    }

    #[test]
    fn column_ref_column_type_returns_correct_value() {
        let r = make_ref();
        assert_eq!(*r.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_ref_equality() {
        let a = make_ref();
        let b = make_ref();
        assert_eq!(a, b);
    }

    #[test]
    fn column_ref_inequality_by_column_id() {
        let table = TableRef::new("schema", "table");
        let a = ColumnRef::new(table.clone(), Ident::new("a"), ColumnType::BigInt);
        let b = ColumnRef::new(table, Ident::new("b"), ColumnType::BigInt);
        assert_ne!(a, b);
    }

    #[test]
    fn column_ref_inequality_by_type() {
        let table = TableRef::new("schema", "table");
        let a = ColumnRef::new(table.clone(), Ident::new("col"), ColumnType::BigInt);
        let b = ColumnRef::new(table, Ident::new("col"), ColumnType::Boolean);
        assert_ne!(a, b);
    }

    #[test]
    fn column_ref_clone_equals_original() {
        let r = make_ref();
        assert_eq!(r.clone(), r);
    }

    #[test]
    fn column_ref_is_debug_formattable() {
        let r = make_ref();
        let s = alloc::format!("{r:?}");
        assert!(s.contains("col"));
    }

    #[test]
    fn column_ref_no_schema_table() {
        let table = TableRef::new("", "mytable");
        let r = ColumnRef::new(table, Ident::new("id"), ColumnType::Int128);
        assert_eq!(*r.column_type(), ColumnType::Int128);
    }
}
