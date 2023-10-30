use super::OwnedColumn;
use indexmap::IndexMap;
use proofs_sql::Identifier;
use thiserror::Error;

/// An error that occurs when working with tables.
#[derive(Error, Debug)]
pub enum OwnedTableError {
    /// The columns have different lengths.
    #[error("Columns have different lengths")]
    ColumnLengthMismatch,
}

/// A table of data, with schema included. This is simply a map from `Identifier` to `OwnedColumn`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow RecordBatch.
#[derive(Debug, Clone, Eq)]
pub struct OwnedTable {
    table: IndexMap<Identifier, OwnedColumn>,
}
impl OwnedTable {
    /// Creates a new OwnedTable.
    pub fn try_new(table: IndexMap<Identifier, OwnedColumn>) -> Result<Self, OwnedTableError> {
        if table.is_empty() {
            return Ok(Self { table });
        }
        let num_rows = table[0].len();
        if table.values().any(|column| column.len() != num_rows) {
            Err(OwnedTableError::ColumnLengthMismatch)
        } else {
            Ok(Self { table })
        }
    }
    /// Creates a new OwnedTable.
    pub fn try_from_iter<T: IntoIterator<Item = (Identifier, OwnedColumn)>>(
        iter: T,
    ) -> Result<Self, OwnedTableError> {
        Self::try_new(IndexMap::from_iter(iter))
    }
    /// Number of columns in the table.
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }
    /// Whether the table has no columns.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
    /// Returns the columns of this table as an IndexMap
    pub fn into_inner(self) -> IndexMap<Identifier, OwnedColumn> {
        self.table
    }
}

// Note: we modify the default PartialEq for IndexMap to also check for column ordering.
// This is to align with the behaviour of a `RecordBatch`.
impl PartialEq for OwnedTable {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}

/// Utility macro to simplify the creation of OwnedTables.
/// Convinience macro wrapping `OwnedTable::try_from_iter` that is only available in tests.
///
/// Note: this panics if the columns have different lengths or if the table has no columns.
#[macro_export]
#[cfg(test)]
macro_rules! owned_table {
    ($($col_name:expr => $slice:expr), + $(,)?) => {
        {
            $crate::base::database::OwnedTable::try_from_iter([$(
                ($col_name.parse().unwrap(), FromIterator::from_iter($slice))
            ,)+]).unwrap()
        }
    }
}
