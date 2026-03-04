use super::{ColumnType, TableRef};
use crate::base::database::ColumnId;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a column on a table in the database. This does not include columns on temporary tables
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

    /// Get the table
    #[must_use]
    pub fn table_ref(&self) -> TableRef {
        self.table_ref.clone()
    }

    /// Get the column name
    #[must_use]
    pub fn column_name(&self) -> Ident {
        self.column_id.clone()
    }

    /// Get the column type
    #[must_use]
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }

    /// Get the `ColumnId`
    #[must_use]
    pub fn column_id(&self) -> ColumnId {
        ColumnId::new(self.column_id.clone(), Some(self.table_ref()))
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct NewColumnRef {
    column_id: Ident,
    table_ref: Option<TableRef>,
    column_type: ColumnType,
}

impl NewColumnRef {
    /// Create a new `ColumnRef` from a table, column identifier and column type
    #[must_use]
    pub fn new(table_ref: Option<TableRef>, column_id: Ident, column_type: ColumnType) -> Self {
        Self {
            column_id,
            table_ref,
            column_type,
        }
    }

    /// Returns the table reference of this column
    #[must_use]
    pub fn table_ref(&self) -> Option<TableRef> {
        self.table_ref.clone()
    }

    /// Returns the column name
    #[must_use]
    pub fn column_name(&self) -> Ident {
        self.column_id.clone()
    }

    /// Returns the column identifier of this column
    #[must_use]
    pub fn column_id(&self) -> ColumnId {
        ColumnId::new(self.column_id.clone(), self.table_ref())
    }

    /// Returns the column type of this column
    #[must_use]
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }
}
