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
    fn column_ref_accessors_return_the_values_used_to_construct_it() {
        let table_ref = TableRef::new("analytics", "events");
        let column_id = Ident::new("event_count");
        let column_ref = ColumnRef::new(table_ref.clone(), column_id.clone(), ColumnType::BigInt);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), column_id);
        assert_eq!(column_ref.column_type(), &ColumnType::BigInt);
    }

    #[test]
    fn column_ref_serialization_round_trips_schema_and_type_information() {
        let column_ref = ColumnRef::new(
            TableRef::new("public", "accounts"),
            Ident::new("display_name"),
            ColumnType::VarChar,
        );
        let serialized = serde_json::to_string(&column_ref).unwrap();

        let deserialized: ColumnRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, column_ref);
        assert_eq!(deserialized.table_ref().to_string(), "public.accounts");
        assert_eq!(deserialized.column_id(), Ident::new("display_name"));
        assert_eq!(deserialized.column_type(), &ColumnType::VarChar);
    }
}
