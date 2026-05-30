use super::{Column, ColumnField};
use crate::base::{map::IndexMap, scalar::Scalar};
use alloc::vec::Vec;
use bumpalo::Bump;
use snafu::Snafu;
use sqlparser::ast::Ident;

/// Options for creating a table.
/// Inspired by [`RecordBatchOptions`](https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatchOptions.html)
#[derive(Debug, Default, Clone, Copy)]
pub struct TableOptions {
    /// The number of rows in the table. Mostly useful for tables without columns.
    pub row_count: Option<usize>,
}

impl TableOptions {
    /// Creates a new [`TableOptions`].
    #[must_use]
    pub fn new(row_count: Option<usize>) -> Self {
        Self { row_count }
    }
}

/// An error that occurs when working with tables.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum TableError {
    /// The columns have different lengths.
    #[snafu(display("Columns have different lengths"))]
    ColumnLengthMismatch,

    /// At least one column has length different from the provided row count.
    #[snafu(display("Column has length different from the provided row count"))]
    ColumnLengthMismatchWithSpecifiedRowCount,

    /// The table is empty and there is no specified row count.
    #[snafu(display("Table is empty and no row count is specified"))]
    EmptyTableWithoutSpecifiedRowCount,
}
/// A table of data, with schema included. This is simply a map from `Ident` to `Column`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow [`RecordBatch`](arrow::record_batch::RecordBatch).
#[derive(Debug, Clone, Eq)]
pub struct Table<'a, S: Scalar> {
    table: IndexMap<Ident, Column<'a, S>>,
    row_count: usize,
}
impl<'a, S: Scalar> Table<'a, S> {
    /// Creates a new [`Table`] with the given columns and default [`TableOptions`].
    pub fn try_new(table: IndexMap<Ident, Column<'a, S>>) -> Result<Self, TableError> {
        Self::try_new_with_options(table, TableOptions::default())
    }

    /// Creates a new [`Table`] with the given columns and with [`TableOptions`].
    pub fn try_new_with_options(
        table: IndexMap<Ident, Column<'a, S>>,
        options: TableOptions,
    ) -> Result<Self, TableError> {
        match (table.is_empty(), options.row_count) {
            (true, None) => Err(TableError::EmptyTableWithoutSpecifiedRowCount),
            (true, Some(row_count)) => Ok(Self { table, row_count }),
            (false, None) => {
                let row_count = table[0].len();
                if table.values().any(|column| column.len() != row_count) {
                    Err(TableError::ColumnLengthMismatch)
                } else {
                    Ok(Self { table, row_count })
                }
            }
            (false, Some(row_count)) => {
                if table.values().any(|column| column.len() != row_count) {
                    Err(TableError::ColumnLengthMismatchWithSpecifiedRowCount)
                } else {
                    Ok(Self { table, row_count })
                }
            }
        }
    }

    /// Creates a new [`Table`] from an iterator of `(Ident, Column)` pairs with default [`TableOptions`].
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, Column<'a, S>)>>(
        iter: T,
    ) -> Result<Self, TableError> {
        Self::try_from_iter_with_options(iter, TableOptions::default())
    }

    /// Creates a new [`Table`] from an iterator of `(Ident, Column)` pairs with [`TableOptions`].
    pub fn try_from_iter_with_options<T: IntoIterator<Item = (Ident, Column<'a, S>)>>(
        iter: T,
        options: TableOptions,
    ) -> Result<Self, TableError> {
        Self::try_new_with_options(IndexMap::from_iter(iter), options)
    }

    /// Number of columns in the table.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }
    /// Number of rows in the table.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        self.row_count
    }
    /// Whether the table has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Ident, Column<'a, S>> {
        self.table
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn inner_table(&self) -> &IndexMap<Ident, Column<'a, S>> {
        &self.table
    }
    /// Return the schema of this table as a `Vec` of `ColumnField`s
    #[must_use]
    pub fn schema(&self) -> Vec<ColumnField> {
        self.table
            .iter()
            .map(|(name, column)| ColumnField::new(name.clone(), column.column_type()))
            .collect()
    }
    /// Returns the columns of this table as an Iterator
    pub fn column_names(&self) -> impl Iterator<Item = &Ident> {
        self.table.keys()
    }
    /// Returns the columns of this table as an Iterator
    pub fn columns(&self) -> impl Iterator<Item = &Column<'a, S>> {
        self.table.values()
    }
    /// Returns the column with the given position.
    #[must_use]
    pub fn column(&self, index: usize) -> Option<&Column<'a, S>> {
        self.table.values().nth(index)
    }
    /// Add the `rho` column as the last column to the table.
    #[must_use]
    pub fn add_rho_column(mut self, alloc: &'a Bump) -> Self {
        self.table
            .insert(Ident::new("rho"), Column::rho(self.row_count, alloc));
        self
    }
}

// Note: we modify the default PartialEq for IndexMap to also check for column ordering.
// This is to align with the behaviour of a `RecordBatch`.
impl<S: Scalar> PartialEq for Table<'_, S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}

#[cfg(test)]
impl<'a, S: Scalar> core::ops::Index<&str> for Table<'a, S> {
    type Output = Column<'a, S>;
    fn index(&self, index: &str) -> &Self::Output {
        self.table.get(&Ident::new(index)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, Table, TableOptions};
    use crate::base::{database::ColumnType, map::IndexMap, scalar::test_scalar::TestScalar};
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    #[test]
    fn table_iterators_and_index_preserve_ordered_columns() {
        let ids = [10_i64, 20_i64];
        let flags = [true, false];
        let table = Table::<TestScalar>::try_from_iter([
            (Ident::new("id"), Column::BigInt(&ids)),
            (Ident::new("flag"), Column::Boolean(&flags)),
        ])
        .unwrap();

        assert!(!table.is_empty());
        assert_eq!(
            table
                .column_names()
                .map(|name| name.value.as_str())
                .collect::<Vec<_>>(),
            ["id", "flag"]
        );
        assert_eq!(
            table.columns().map(Column::column_type).collect::<Vec<_>>(),
            [ColumnType::BigInt, ColumnType::Boolean]
        );
        assert_eq!(table["id"], Column::BigInt(&ids));
        assert_eq!(table["flag"], Column::Boolean(&flags));

        let alloc = Bump::new();
        let table_with_rho = table.add_rho_column(&alloc);
        assert_eq!(table_with_rho["rho"], Column::Int128(&[0_i128, 1_i128]));
    }

    #[test]
    fn empty_table_with_row_count_reports_empty_iterators() {
        let table = Table::<TestScalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions::new(Some(3)),
        )
        .unwrap();

        assert!(table.is_empty());
        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.column_names().count(), 0);
        assert_eq!(table.columns().count(), 0);
    }
}
