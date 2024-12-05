use super::{OwnedColumn, Table};
use crate::base::{map::IndexMap, polynomial::compute_evaluation_vector, scalar::Scalar};
use alloc::{vec, vec::Vec};
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// An error that occurs when working with tables.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum OwnedTableError {
    /// The columns have different lengths.
    #[snafu(display("Columns have different lengths"))]
    ColumnLengthMismatch,
}
/// A table of data, with schema included. This is simply a map from `Identifier` to `OwnedColumn`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow [`RecordBatch`](arrow::record_batch::RecordBatch).
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct OwnedTable<S: Scalar> {
    table: IndexMap<Identifier, OwnedColumn<S>>,
}
impl<S: Scalar> OwnedTable<S> {
    /// Creates a new [`OwnedTable`].
    pub fn try_new(table: IndexMap<Identifier, OwnedColumn<S>>) -> Result<Self, OwnedTableError> {
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
    /// Creates a new [`OwnedTable`].
    pub fn try_from_iter<T: IntoIterator<Item = (Identifier, OwnedColumn<S>)>>(
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
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Identifier, OwnedColumn<S>> {
        self.table
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn inner_table(&self) -> &IndexMap<Identifier, OwnedColumn<S>> {
        &self.table
    }
    /// Returns the columns of this table as an Iterator
    pub fn column_names(&self) -> impl Iterator<Item = &Identifier> {
        self.table.keys()
    }

    pub(crate) fn mle_evaluations(&self, evaluation_point: &[S]) -> Vec<S> {
        let mut evaluation_vector = vec![S::ZERO; self.num_rows()];
        compute_evaluation_vector(&mut evaluation_vector, evaluation_point);
        self.table
            .values()
            .map(|column| column.inner_product(&evaluation_vector))
            .collect()
    }
}

// Note: we modify the default PartialEq for IndexMap to also check for column ordering.
// This is to align with the behaviour of a `RecordBatch`.
impl<S: Scalar> PartialEq for OwnedTable<S> {
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
impl<S: Scalar> core::ops::Index<&str> for OwnedTable<S> {
    type Output = OwnedColumn<S>;
    fn index(&self, index: &str) -> &Self::Output {
        self.table
            .get(&index.parse::<Identifier>().unwrap())
            .unwrap()
    }
}

impl<'a, S: Scalar> From<&Table<'a, S>> for OwnedTable<S> {
    fn from(value: &Table<'a, S>) -> Self {
        OwnedTable::try_from_iter(
            value
                .inner_table()
                .iter()
                .map(|(name, column)| (*name, OwnedColumn::from(column))),
        )
        .expect("Tables should not have columns with differing lengths")
    }
}

impl<'a, S: Scalar> From<Table<'a, S>> for OwnedTable<S> {
    fn from(value: Table<'a, S>) -> Self {
        OwnedTable::try_from_iter(
            value
                .into_inner()
                .into_iter()
                .map(|(name, column)| (name, OwnedColumn::from(&column))),
        )
        .expect("Tables should not have columns with differing lengths")
    }
}

#[cfg(test)]
mod tests {
    use super::OwnedTable;
    use crate::base::{
        database::{owned_table_utility::*, table_utility::*, Table, TableOptions},
        map::indexmap,
        scalar::test_scalar::TestScalar,
    };
    use bumpalo::Bump;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn test_conversion_from_table_to_owned_table() {
        let alloc = Bump::new();

        let borrowed_table = table::<TestScalar>([
            borrowed_bigint(
                "bigint",
                [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
                &alloc,
            ),
            borrowed_int128(
                "decimal",
                [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
                &alloc,
            ),
            borrowed_varchar(
                "varchar",
                ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
                &alloc,
            ),
            borrowed_scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8], &alloc),
            borrowed_boolean(
                "boolean",
                [true, false, true, false, true, false, true, false, true],
                &alloc,
            ),
            borrowed_timestamptz(
                "time_stamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
                &alloc,
            ),
        ]);

        let expected_table = owned_table::<TestScalar>([
            bigint("bigint", [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            int128("decimal", [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
            varchar("varchar", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
            scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
            boolean(
                "boolean",
                [true, false, true, false, true, false, true, false, true],
            ),
            timestamptz(
                "time_stamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            ),
        ]);

        assert_eq!(OwnedTable::from(&borrowed_table), expected_table);
        assert_eq!(OwnedTable::from(borrowed_table), expected_table);
    }

    #[test]
    fn test_empty_and_no_columns_tables() {
        let alloc = Bump::new();
        // Test with no rows
        let empty_table = table::<TestScalar>([borrowed_bigint("bigint", [0; 0], &alloc)]);
        let expected_empty_table = owned_table::<TestScalar>([bigint("bigint", [0; 0])]);
        assert_eq!(OwnedTable::from(&empty_table), expected_empty_table);
        assert_eq!(OwnedTable::from(empty_table), expected_empty_table);

        // Test with no columns
        let no_columns_table_no_rows =
            Table::try_new_with_options(indexmap! {}, TableOptions::new(Some(0))).unwrap();
        let no_columns_table_two_rows =
            Table::try_new_with_options(indexmap! {}, TableOptions::new(Some(2))).unwrap();
        let expected_no_columns_table = owned_table::<TestScalar>([]);
        assert_eq!(
            OwnedTable::from(&no_columns_table_no_rows),
            expected_no_columns_table
        );
        assert_eq!(
            OwnedTable::from(no_columns_table_no_rows),
            expected_no_columns_table
        );
        assert_eq!(
            OwnedTable::from(&no_columns_table_two_rows),
            expected_no_columns_table
        );
        assert_eq!(
            OwnedTable::from(no_columns_table_two_rows),
            expected_no_columns_table
        );
    }
}
