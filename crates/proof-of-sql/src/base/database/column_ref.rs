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
    fn we_can_access_column_ref_metadata() {
        let table_ref = TableRef::new("analytics", "accounts");
        let column_ref =
            ColumnRef::new(table_ref.clone(), Ident::new("balance"), ColumnType::Int128);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), Ident::new("balance"));
        assert_eq!(column_ref.column_type(), &ColumnType::Int128);
    }

    #[test]
    fn we_can_roundtrip_column_ref_through_json() {
        let column_ref = ColumnRef::new(
            TableRef::new("analytics", "accounts"),
            Ident::new("email"),
            ColumnType::VarChar,
        );

        let serialized = serde_json::to_string(&column_ref).unwrap();
        let deserialized: ColumnRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, column_ref);
    }
}
