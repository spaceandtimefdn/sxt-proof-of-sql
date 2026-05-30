use super::{ColumnField, NullableColumn, NullableOwnedTable, TableError, TableOptions};
use crate::base::{database::OwnedColumn, map::IndexMap, scalar::Scalar};
use alloc::vec::Vec;
use sqlparser::ast::Ident;

/// A borrowed table whose columns can carry nullable row-presence data.
#[derive(Debug, Clone, Eq)]
pub struct NullableTable<'a, S: Scalar> {
    table: IndexMap<Ident, NullableColumn<'a, S>>,
    row_count: usize,
}

impl<'a, S: Scalar> NullableTable<'a, S> {
    /// Creates a new [`NullableTable`] with default [`TableOptions`].
    pub fn try_new(table: IndexMap<Ident, NullableColumn<'a, S>>) -> Result<Self, TableError> {
        Self::try_new_with_options(table, TableOptions::default())
    }

    /// Creates a new [`NullableTable`] with explicit [`TableOptions`].
    pub fn try_new_with_options(
        table: IndexMap<Ident, NullableColumn<'a, S>>,
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

    /// Creates a new [`NullableTable`] from an iterator of `(Ident, NullableColumn)` pairs.
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, NullableColumn<'a, S>)>>(
        iter: T,
    ) -> Result<Self, TableError> {
        Self::try_from_iter_with_options(iter, TableOptions::default())
    }

    /// Creates a new [`NullableTable`] from an iterator and explicit [`TableOptions`].
    pub fn try_from_iter_with_options<T: IntoIterator<Item = (Ident, NullableColumn<'a, S>)>>(
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
    pub const fn num_rows(&self) -> usize {
        self.row_count
    }

    /// Whether the table has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the columns of this table.
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Ident, NullableColumn<'a, S>> {
        self.table
    }

    /// Returns the columns of this table by reference.
    #[must_use]
    pub const fn inner_table(&self) -> &IndexMap<Ident, NullableColumn<'a, S>> {
        &self.table
    }

    /// Return the schema of this table as a `Vec` of `ColumnField`s.
    #[must_use]
    pub fn schema(&self) -> Vec<ColumnField> {
        self.table
            .iter()
            .map(|(name, column)| {
                if column.is_nullable() {
                    ColumnField::new_nullable(name.clone(), column.values().column_type())
                } else {
                    ColumnField::new(name.clone(), column.values().column_type())
                }
            })
            .collect()
    }

    /// Returns the column names as an iterator.
    pub fn column_names(&self) -> impl Iterator<Item = &Ident> {
        self.table.keys()
    }

    /// Returns the columns as an iterator.
    pub fn columns(&self) -> impl Iterator<Item = &NullableColumn<'a, S>> {
        self.table.values()
    }

    /// Returns the column with the given position.
    #[must_use]
    pub fn column(&self, index: usize) -> Option<&NullableColumn<'a, S>> {
        self.table.values().nth(index)
    }
}

impl<S: Scalar> PartialEq for NullableTable<'_, S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}

impl<'a, S: Scalar> From<&NullableTable<'a, S>> for NullableOwnedTable<S> {
    fn from(value: &NullableTable<'a, S>) -> Self {
        NullableOwnedTable::try_from_iter(value.inner_table().iter().map(|(name, column)| {
            let values = column.values();
            (
                name.clone(),
                super::NullableOwnedColumn::try_new(
                    OwnedColumn::from(&values),
                    column.presence().map(<[bool]>::to_vec),
                )
                .expect("Nullable columns should have matching value and presence lengths"),
            )
        }))
        .expect("Tables should not have columns with differing lengths")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::{Column, ColumnType},
        map::indexmap,
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn nullable_table_schema_marks_nullable_columns() {
        let table = NullableTable::try_new(indexmap! {
            "id".into() => NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[1, 2])),
            "amount".into() => NullableColumn::<TestScalar>::try_new(
                Column::BigInt(&[10, 20]),
                Some(&[true, false])
            ).unwrap(),
        })
        .unwrap();

        let schema = table.schema();

        assert_eq!(schema[0].name(), "id".into());
        assert_eq!(schema[0].data_type(), ColumnType::BigInt);
        assert!(!schema[0].is_nullable());
        assert_eq!(schema[1].name(), "amount".into());
        assert_eq!(schema[1].data_type(), ColumnType::BigInt);
        assert!(schema[1].is_nullable());
    }

    #[test]
    fn nullable_table_rejects_column_length_mismatches() {
        let result = NullableTable::try_new(indexmap! {
            "id".into() => NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[1, 2])),
            "amount".into() => NullableColumn::<TestScalar>::try_new(
                Column::BigInt(&[10]),
                Some(&[true])
            ).unwrap(),
        });

        assert_eq!(result, Err(TableError::ColumnLengthMismatch));
    }
}
