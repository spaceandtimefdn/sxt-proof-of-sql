use super::TableRef;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnRef {
    column_id: Ident,
    table_ref: TableRef,
}

impl ColumnRef {
    /// Create a new `ColumnRef` from a table and column identifier
    #[must_use]
    pub fn new(table_ref: TableRef, column_id: Ident) -> Self {
        Self {
            column_id,
            table_ref,
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
}
