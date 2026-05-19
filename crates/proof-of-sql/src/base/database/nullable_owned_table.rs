use super::{NullableOwnedColumn, NullableTable, OwnedTableError};
use crate::base::{map::IndexMap, scalar::Scalar};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// An owned table whose columns can carry nullable row-presence data.
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct NullableOwnedTable<S: Scalar> {
    table: IndexMap<Ident, NullableOwnedColumn<S>>,
}

impl<S: Scalar> NullableOwnedTable<S> {
    /// Creates a new [`NullableOwnedTable`].
    pub fn try_new(
        table: IndexMap<Ident, NullableOwnedColumn<S>>,
    ) -> Result<Self, OwnedTableError> {
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

    /// Creates a new [`NullableOwnedTable`] from an iterator.
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, NullableOwnedColumn<S>)>>(
        iter: T,
    ) -> Result<Self, OwnedTableError> {
        Self::try_new(IndexMap::from_iter(iter))
    }

    /// Number of columns in the table.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }

    /// Number of rows in the table.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        if self.table.is_empty() {
            0
        } else {
            self.table[0].len()
        }
    }

    /// Whether the table has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the columns of this table.
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Ident, NullableOwnedColumn<S>> {
        self.table
    }

    /// Returns the columns of this table by reference.
    #[must_use]
    pub const fn inner_table(&self) -> &IndexMap<Ident, NullableOwnedColumn<S>> {
        &self.table
    }

    /// Returns the column names as an iterator.
    pub fn column_names(&self) -> impl Iterator<Item = &Ident> {
        self.table.keys()
    }

    /// Returns the column with the given position.
    #[must_use]
    pub fn column_by_index(&self, index: usize) -> Option<&NullableOwnedColumn<S>> {
        self.table.get_index(index).map(|(_, v)| v)
    }
}

impl<S: Scalar> PartialEq for NullableOwnedTable<S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}

impl<'a, S: Scalar> From<NullableTable<'a, S>> for NullableOwnedTable<S> {
    fn from(value: NullableTable<'a, S>) -> Self {
        NullableOwnedTable::from(&value)
    }
}

#[cfg(test)]
impl<S: Scalar> core::ops::Index<&str> for NullableOwnedTable<S> {
    type Output = NullableOwnedColumn<S>;

    fn index(&self, index: &str) -> &Self::Output {
        self.table.get(&Ident::new(index)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::{Column, NullableColumn, OwnedColumn},
        map::indexmap,
        scalar::test_scalar::TestScalar,
    };
    use alloc::vec;

    #[test]
    fn nullable_owned_table_rejects_column_length_mismatches() {
        let result = NullableOwnedTable::try_new(indexmap! {
            "id".into() => NullableOwnedColumn::<TestScalar>::new_nonnullable(
                OwnedColumn::BigInt(vec![1, 2])
            ),
            "amount".into() => NullableOwnedColumn::<TestScalar>::try_new(
                OwnedColumn::BigInt(vec![10]),
                Some(vec![true])
            ).unwrap(),
        });

        assert_eq!(result, Err(OwnedTableError::ColumnLengthMismatch));
    }

    #[test]
    fn nullable_table_converts_to_owned_table() {
        let borrowed = NullableTable::try_new(indexmap! {
            "id".into() => NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[1, 2])),
            "amount".into() => NullableColumn::<TestScalar>::try_new(
                Column::BigInt(&[10, 20]),
                Some(&[true, false])
            ).unwrap(),
        })
        .unwrap();

        let owned = NullableOwnedTable::from(&borrowed);

        assert_eq!(
            owned["id"],
            NullableOwnedColumn::new_nonnullable(OwnedColumn::BigInt(vec![1, 2]))
        );
        assert_eq!(
            owned["amount"],
            NullableOwnedColumn::try_new(
                OwnedColumn::BigInt(vec![10, 20]),
                Some(vec![true, false])
            )
            .unwrap()
        );
        assert_eq!(owned.num_rows(), 2);
        assert_eq!(owned.num_columns(), 2);
    }
}
