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
    use super::{ColumnRef, ColumnType, TableRef};
    use sqlparser::ast::Ident;

    #[test]
    fn column_ref_accessors_return_owned_reference_parts() {
        let table_ref = TableRef::new("sxt", "orders");
        let column_ref = ColumnRef::new(table_ref.clone(), Ident::new("total"), ColumnType::Int128);

        assert_eq!(column_ref.table_ref(), table_ref);
        assert_eq!(column_ref.column_id(), Ident::new("total"));
        assert_eq!(column_ref.column_type(), &ColumnType::Int128);

        let cloned_table = column_ref.table_ref();
        let mut cloned_table_name = cloned_table.table_id().clone();
        cloned_table_name.value = "mutated".into();
        let mut cloned_column = column_ref.column_id();
        cloned_column.value = "mutated".into();

        assert_eq!(column_ref.table_ref(), TableRef::new("sxt", "orders"));
        assert_eq!(column_ref.column_id(), Ident::new("total"));
    }

    #[test]
    fn column_ref_round_trips_through_serde() {
        let column_ref = ColumnRef::new(
            TableRef::new("analytics", "events"),
            Ident::new("payload"),
            ColumnType::VarBinary,
        );

        let serialized = serde_json::to_string(&column_ref).unwrap();
        let deserialized: ColumnRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, column_ref);
        assert_eq!(
            deserialized.table_ref(),
            TableRef::new("analytics", "events")
        );
        assert_eq!(deserialized.column_id(), Ident::new("payload"));
        assert_eq!(deserialized.column_type(), &ColumnType::VarBinary);
    }
}
