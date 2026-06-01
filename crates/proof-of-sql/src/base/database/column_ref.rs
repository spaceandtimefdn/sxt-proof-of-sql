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
    fn we_can_retrieve_column_reference_parts() {
        let table_ref = TableRef::new("sxt", "trades");
        let column_id = Ident::new("amount");
        let column_type = ColumnType::BigInt;

        let column_ref = ColumnRef::new(table_ref.clone(), column_id.clone(), column_type);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), column_id);
        assert_eq!(column_ref.column_type(), &column_type);
    }

    #[test]
    fn we_can_clone_column_references() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "trades"),
            Ident::new("price"),
            ColumnType::BigInt,
        );

        assert_eq!(column_ref.clone(), column_ref);
    }
}
