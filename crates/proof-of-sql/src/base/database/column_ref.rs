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
    use crate::base::database::TableRef;

    #[test]
    fn constructor_stores_table_column_and_type_metadata() {
        let table_ref = TableRef::from_names(Some("public"), "orders");
        let column_ref =
            ColumnRef::new(table_ref.clone(), Ident::new("amount"), ColumnType::BigInt);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), Ident::new("amount"));
        assert_eq!(*column_ref.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_refs_round_trip_through_serde() {
        let column_ref = ColumnRef::new(
            TableRef::from_names(Some("analytics"), "orders"),
            Ident::new("customer_id"),
            ColumnType::VarChar,
        );

        let json = serde_json::to_string(&column_ref).unwrap();
        assert_eq!(
            serde_json::from_str::<ColumnRef>(&json).unwrap(),
            column_ref
        );
    }
}
