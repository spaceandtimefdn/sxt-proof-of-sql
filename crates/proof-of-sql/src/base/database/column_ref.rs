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
    fn new_stores_column_metadata() {
        let table_ref = TableRef::new("public", "transactions");
        let column_id = Ident::new("amount");
        let column_ref = ColumnRef::new(table_ref.clone(), column_id.clone(), ColumnType::BigInt);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), column_id);
        assert_eq!(column_ref.column_type(), &ColumnType::BigInt);
    }

    #[test]
    fn accessors_return_owned_identifier_values() {
        let table_ref = TableRef::new("public", "transactions");
        let column_ref = ColumnRef::new(table_ref, Ident::new("amount"), ColumnType::BigInt);

        let mut returned_column_id = column_ref.column_id();
        returned_column_id.value = "other_column".into();

        assert_eq!(
            column_ref.table_ref(),
            TableRef::new("public", "transactions")
        );
        assert_eq!(column_ref.column_id().value, "amount");
    }
}
