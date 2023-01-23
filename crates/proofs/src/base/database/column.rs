use super::TableRef;
use arrow::datatypes::DataType;
use proofs_sql::Identifier;
use serde::{Deserialize, Serialize};

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
pub enum Column<'a> {
    BigInt(&'a [i64]),
}

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub enum ColumnType {
    BigInt,
}

/// Convert ColumnType values to some arrow DataType
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::BigInt => DataType::Int64,
        }
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ColumnRef {
    column_name: Identifier,
    table_ref: TableRef,
    column_type: ColumnType,
}

impl ColumnRef {
    pub fn new(table_ref: TableRef, column_name: Identifier, column_type: ColumnType) -> Self {
        Self {
            column_name,
            column_type,
            table_ref,
        }
    }

    pub fn column_name(&self) -> &str {
        self.column_name.name()
    }

    pub fn schema(&self) -> &str {
        self.table_ref.schema()
    }

    pub fn table_name(&self) -> &str {
        self.table_ref.table_name()
    }

    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }
}
