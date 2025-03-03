use super::{ColumnField, OwnedColumn, Table};
use crate::base::{
    database::ColumnCoercionError, map::IndexMap, polynomial::compute_evaluation_vector,
    scalar::Scalar,
};
use alloc::{vec, vec::Vec};
use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use sqlparser::ast::Ident;

/// An error that occurs when working with tables.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum OwnedTableError {
    /// The columns have different lengths.
    #[snafu(display("Columns have different lengths"))]
    ColumnLengthMismatch,
}

/// Errors that can occur when coercing a table.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub(crate) enum TableCoercionError {
    #[snafu(transparent)]
    ColumnCoercionError { source: ColumnCoercionError },
    /// Name mismatch between column and field.
    #[snafu(display("Name mismatch between column and field"))]
    NameMismatch,
    /// Column count mismatch.
    #[snafu(display("Column count mismatch"))]
    ColumnCountMismatch,
}

/// A table of data, with schema included. This is simply a map from `Ident` to `OwnedColumn`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow [`RecordBatch`](arrow::record_batch::RecordBatch).
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct OwnedTable<S: Scalar> {
    table: IndexMap<Ident, OwnedColumn<S>>,
}
impl<S: Scalar> OwnedTable<S> {
    /// Creates a new [`OwnedTable`].
    pub fn try_new(table: IndexMap<Ident, OwnedColumn<S>>) -> Result<Self, OwnedTableError> {
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
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, OwnedColumn<S>)>>(
        iter: T,
    ) -> Result<Self, OwnedTableError> {
        Self::try_new(IndexMap::from_iter(iter))
    }

    #[allow(
        clippy::missing_panics_doc,
        reason = "Mapping from one table to another should not result in column mismatch"
    )]
    /// Attempts to coerce the columns of the table to match the provided fields.
    ///
    /// # Arguments
    ///
    /// * `fields` - An iterator of `ColumnField` items that specify the desired schema.
    ///
    /// # Errors
    ///
    /// Returns a `TableCoercionError` if:
    /// * The number of columns in the table does not match the number of fields.
    /// * The name of a column does not match the name of the corresponding field.
    /// * A column cannot be coerced to the type specified by the corresponding field.
    pub(crate) fn try_coerce_with_fields<T: IntoIterator<Item = ColumnField>>(
        self,
        fields: T,
    ) -> Result<Self, TableCoercionError> {
        self.into_inner()
            .into_iter()
            .zip_longest(fields)
            .map(|p| match p {
                EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => {
                    Err(TableCoercionError::ColumnCountMismatch)
                }
                EitherOrBoth::Both((name, column), field) if name == field.name() => Ok((
                    name,
                    column.try_coerce_scalar_to_numeric(field.data_type())?,
                )),
                EitherOrBoth::Both(_, _) => Err(TableCoercionError::NameMismatch),
            })
            .process_results(|iter| {
                Self::try_from_iter(iter).expect("Columns should have the same length")
            })
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
    pub fn into_inner(self) -> IndexMap<Ident, OwnedColumn<S>> {
        self.table
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn inner_table(&self) -> &IndexMap<Ident, OwnedColumn<S>> {
        &self.table
    }
    /// Returns the columns of this table as an Iterator
    pub fn column_names(&self) -> impl Iterator<Item = &Ident> {
        self.table.keys()
    }
    /// Returns the column with the given position.
    #[must_use]
    pub fn column_by_index(&self, index: usize) -> Option<&OwnedColumn<S>> {
        self.table.get_index(index).map(|(_, v)| v)
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
        self.table.get(&Ident::new(index)).unwrap()
    }
}

impl<'a, S: Scalar> From<&Table<'a, S>> for OwnedTable<S> {
    fn from(value: &Table<'a, S>) -> Self {
        OwnedTable::try_from_iter(
            value
                .inner_table()
                .iter()
                .map(|(name, column)| (name.clone(), OwnedColumn::from(column))),
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
        database::{
            owned_table_utility::*, table_utility::*, ColumnCoercionError, Table,
            TableCoercionError, TableOptions,
        },
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
                PoSQLTimeZone::utc(),
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
                PoSQLTimeZone::utc(),
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

    #[test]
    fn test_try_coerce_with_fields() {
        use crate::base::database::{ColumnField, ColumnType};

        let table = owned_table::<TestScalar>([
            bigint("bigint", [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ]);

        let fields = vec![
            ColumnField::new("bigint".into(), ColumnType::BigInt),
            ColumnField::new("scalar".into(), ColumnType::Int),
        ];

        let coerced_table = table.clone().try_coerce_with_fields(fields).unwrap();

        let expected_table = owned_table::<TestScalar>([
            bigint("bigint", [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            int("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ]);

        assert_eq!(coerced_table, expected_table);
    }

    #[test]
    fn test_try_coerce_with_fields_name_mismatch() {
        use crate::base::database::{ColumnField, ColumnType};

        let table = owned_table::<TestScalar>([
            bigint("bigint", [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ]);

        let fields = vec![
            ColumnField::new("bigint".into(), ColumnType::BigInt),
            ColumnField::new("mismatch".into(), ColumnType::Int),
        ];

        let result = table.clone().try_coerce_with_fields(fields);

        assert!(matches!(result, Err(TableCoercionError::NameMismatch)));
    }

    #[test]
    fn test_try_coerce_with_fields_column_count_mismatch() {
        use crate::base::database::{ColumnField, ColumnType};

        let table = owned_table::<TestScalar>([
            bigint("bigint", [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ]);

        let fields = vec![ColumnField::new("bigint".into(), ColumnType::BigInt)];

        let result = table.clone().try_coerce_with_fields(fields);

        assert!(matches!(
            result,
            Err(TableCoercionError::ColumnCountMismatch)
        ));
    }

    #[test]
    fn test_try_coerce_with_fields_overflow() {
        use crate::base::database::{ColumnField, ColumnType};

        let table = owned_table::<TestScalar>([
            bigint("bigint", [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, i64::MAX]),
        ]);

        let fields = vec![
            ColumnField::new("bigint".into(), ColumnType::BigInt),
            ColumnField::new("scalar".into(), ColumnType::TinyInt),
        ];

        let result = table.clone().try_coerce_with_fields(fields);

        assert!(matches!(
            result,
            Err(TableCoercionError::ColumnCoercionError {
                source: ColumnCoercionError::Overflow
            })
        ));
    }
}
