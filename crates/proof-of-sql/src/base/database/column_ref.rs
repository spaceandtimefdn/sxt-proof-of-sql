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

    fn make_ref(table: &str, col: &str, ty: ColumnType) -> ColumnRef {
        ColumnRef::new(TableRef::new("s", table), Ident::new(col), ty)
    }

    #[test]
    fn table_ref_returns_correct_table() {
        let r = make_ref("t", "col", ColumnType::BigInt);
        assert_eq!(r.table_ref().table_id().value.as_str(), "t");
    }

    #[test]
    fn column_id_returns_correct_ident() {
        let r = make_ref("t", "mycol", ColumnType::BigInt);
        assert_eq!(r.column_id().value.as_str(), "mycol");
    }

    #[test]
    fn column_type_returns_correct_type() {
        let r = make_ref("t", "col", ColumnType::Boolean);
        assert_eq!(*r.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn equality_holds_for_same_values() {
        let a = make_ref("t", "col", ColumnType::BigInt);
        let b = make_ref("t", "col", ColumnType::BigInt);
        assert_eq!(a, b);
    }

    #[test]
    fn different_types_are_not_equal() {
        let a = make_ref("t", "col", ColumnType::BigInt);
        let b = make_ref("t", "col", ColumnType::Boolean);
        assert_ne!(a, b);
    }

    #[test]
    fn debug_contains_column_ref() {
        let r = make_ref("t", "col", ColumnType::BigInt);
        assert!(alloc::format!("{r:?}").contains("ColumnRef"));
    }

    #[test]
    fn clone_produces_equal_value() {
        let r = make_ref("t", "col", ColumnType::BigInt);
        assert_eq!(r.clone(), r);
    }
}
