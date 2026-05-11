use super::{
    Column, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor, Table,
    TableRef, TestAccessor,
};
use crate::base::{
    commitment::{CommitmentEvaluationProof, VecCommitmentExt},
    map::IndexMap,
};
use alloc::vec::Vec;
use sqlparser::ast::Ident;

/// A test accessor that uses [`Table`] as the underlying table type.
/// Note: this is intended for testing and examples. It is not optimized for performance, so should not be used for benchmarks or production use-cases.
pub struct TableTestAccessor<'a, CP: CommitmentEvaluationProof> {
    tables: IndexMap<TableRef, (Table<'a, CP::Scalar>, usize)>,
    setup: Option<CP::ProverPublicSetup<'a>>,
}

impl<CP: CommitmentEvaluationProof> Default for TableTestAccessor<'_, CP> {
    fn default() -> Self {
        Self {
            tables: IndexMap::default(),
            setup: None,
        }
    }
}

impl<CP: CommitmentEvaluationProof> Clone for TableTestAccessor<'_, CP> {
    fn clone(&self) -> Self {
        Self {
            tables: self.tables.clone(),
            setup: self.setup,
        }
    }
}

impl<'a, CP: CommitmentEvaluationProof> TestAccessor<CP::Commitment> for TableTestAccessor<'a, CP> {
    type Table = Table<'a, CP::Scalar>;

    fn new_empty() -> Self {
        TableTestAccessor::default()
    }

    fn add_table(&mut self, table_ref: TableRef, data: Self::Table, table_offset: usize) {
        self.tables.insert(table_ref, (data, table_offset));
    }
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating
    /// that an invalid reference was provided.
    fn get_column_names(&self, table_ref: &TableRef) -> Vec<&str> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .column_names()
            .map(|ident| ident.value.as_str())
            .collect()
    }

    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
    fn update_offset(&mut self, table_ref: &TableRef, new_offset: usize) {
        self.tables.get_mut(&table_ref).unwrap().1 = new_offset;
    }
}

///
/// # Panics
///
/// Will panic if the `table_ref` is not found in `self.tables`, or if
/// the `column_id` is not found in the inner table for that reference,
/// indicating that an invalid column reference was provided.
impl<'a, CP: CommitmentEvaluationProof> DataAccessor<CP::Scalar> for TableTestAccessor<'a, CP> {
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<'a, CP::Scalar> {
        *self
            .tables
            .get(table_ref)
            .unwrap()
            .0
            .inner_table()
            .get(column_id)
            .unwrap()
    }
}

///
/// # Panics
///
/// Will panic if the `table_ref` is not found in `self.tables`, or if the `column_id` is not found in the inner table for that reference,indicating that an invalid column reference was provided.
impl<CP: CommitmentEvaluationProof> CommitmentAccessor<CP::Commitment>
    for TableTestAccessor<'_, CP>
{
    fn get_commitment(&self, table_ref: &TableRef, column_id: &Ident) -> CP::Commitment {
        let (table, offset) = self.tables.get(table_ref).unwrap();
        let borrowed_column = table.inner_table().get(column_id).unwrap();
        Vec::<CP::Commitment>::from_columns_with_offset(
            [borrowed_column],
            *offset,
            self.setup.as_ref().unwrap(),
        )[0]
        .clone()
    }
}
impl<CP: CommitmentEvaluationProof> MetadataAccessor for TableTestAccessor<'_, CP> {
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
    fn get_length(&self, table_ref: &TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().0.num_rows()
    }
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
    fn get_offset(&self, table_ref: &TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().1
    }
}
impl<CP: CommitmentEvaluationProof> SchemaAccessor for TableTestAccessor<'_, CP> {
    fn lookup_column(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType> {
        Some(
            self.tables
                .get(table_ref)?
                .0
                .inner_table()
                .get(column_id)?
                .column_type(),
        )
    }
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
    fn lookup_schema(&self, table_ref: &TableRef) -> Vec<(Ident, ColumnType)> {
        self.tables
            .get(table_ref)
            .unwrap()
            .0
            .inner_table()
            .iter()
            .map(|(id, col)| (id.clone(), col.column_type()))
            .collect()
    }
}

impl<'a, CP: CommitmentEvaluationProof> TableTestAccessor<'a, CP> {
    /// Create a new empty test accessor with the given setup.
    pub fn new_empty_with_setup(setup: CP::ProverPublicSetup<'a>) -> Self {
        let mut res = Self::new_empty();
        res.setup = Some(setup);
        res
    }

    /// Create a new test accessor containing the provided table.
    pub fn new_from_table(
        table_ref: TableRef,
        table: Table<'a, CP::Scalar>,
        offset: usize,
        setup: CP::ProverPublicSetup<'a>,
    ) -> Self {
        let mut res = Self::new_empty_with_setup(setup);
        res.add_table(table_ref, table, offset);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        commitment::{
            naive_commitment::NaiveCommitment, naive_evaluation_proof::NaiveEvaluationProof,
            Commitment, CommittableColumn,
        },
        database::table_utility::*,
        map::indexset,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
    };
    use bumpalo::Bump;

    type Accessor<'a> = TableTestAccessor<'a, NaiveEvaluationProof>;

    fn sample_table<'a>(alloc: &'a Bump) -> Table<'a, TestScalar> {
        let raw_bytes = [
            alloc.alloc_slice_copy(&[1_u8, 2][..]) as &[u8],
            alloc.alloc_slice_copy(&[3_u8, 4, 5][..]) as &[u8],
        ];
        let byte_scalars = [
            TestScalar::from_byte_slice_via_hash(raw_bytes[0]),
            TestScalar::from_byte_slice_via_hash(raw_bytes[1]),
        ];
        table([
            borrowed_boolean("bool", [true, false], alloc),
            borrowed_uint8("u8", [1_u8, 2], alloc),
            borrowed_tinyint("i8", [-1_i8, 2], alloc),
            borrowed_smallint("i16", [-3_i16, 4], alloc),
            borrowed_int("i32", [-5_i32, 6], alloc),
            borrowed_bigint("i64", [-7_i64, 8], alloc),
            borrowed_int128("i128", [-9_i128, 10], alloc),
            borrowed_decimal75("decimal", 10, 2, [11, 12], alloc),
            borrowed_scalar("scalar", [13, 14], alloc),
            borrowed_varchar("text", ["a", "bc"], alloc),
            (
                "bytes".into(),
                Column::VarBinary((
                    alloc.alloc_slice_copy(&raw_bytes),
                    alloc.alloc_slice_copy(&byte_scalars),
                )),
            ),
            borrowed_timestamptz(
                "time",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [100, 200],
                alloc,
            ),
        ])
    }

    #[test]
    fn table_test_accessor_exposes_metadata_and_schema() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "borrowed");
        let accessor = Accessor::new_from_table(table_ref.clone(), sample_table(&alloc), 7, ());

        assert_eq!(accessor.get_length(&table_ref), 2);
        assert_eq!(accessor.get_offset(&table_ref), 7);
        assert_eq!(
            accessor.get_column_names(&table_ref),
            vec![
                "bool", "u8", "i8", "i16", "i32", "i64", "i128", "decimal", "scalar", "text",
                "bytes", "time"
            ]
        );
        assert_eq!(
            accessor.lookup_column(&table_ref, &"decimal".into()),
            Some(ColumnType::Decimal75(Precision::new(10).unwrap(), 2))
        );
        assert_eq!(accessor.lookup_column(&table_ref, &"missing".into()), None);
        assert_eq!(
            accessor.lookup_schema(&table_ref)[..3],
            [
                ("bool".into(), ColumnType::Boolean),
                ("u8".into(), ColumnType::Uint8),
                ("i8".into(), ColumnType::TinyInt),
            ]
        );
    }

    #[test]
    fn table_test_accessor_returns_borrowed_columns_for_each_type() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "borrowed");
        let accessor = Accessor::new_from_table(table_ref.clone(), sample_table(&alloc), 0, ());

        match accessor.get_column(&table_ref, &"bool".into()) {
            Column::Boolean(col) => assert_eq!(col, &[true, false]),
            _ => panic!("expected boolean column"),
        }
        match accessor.get_column(&table_ref, &"u8".into()) {
            Column::Uint8(col) => assert_eq!(col, &[1, 2]),
            _ => panic!("expected uint8 column"),
        }
        match accessor.get_column(&table_ref, &"i8".into()) {
            Column::TinyInt(col) => assert_eq!(col, &[-1, 2]),
            _ => panic!("expected tinyint column"),
        }
        match accessor.get_column(&table_ref, &"i16".into()) {
            Column::SmallInt(col) => assert_eq!(col, &[-3, 4]),
            _ => panic!("expected smallint column"),
        }
        match accessor.get_column(&table_ref, &"i32".into()) {
            Column::Int(col) => assert_eq!(col, &[-5, 6]),
            _ => panic!("expected int column"),
        }
        match accessor.get_column(&table_ref, &"i64".into()) {
            Column::BigInt(col) => assert_eq!(col, &[-7, 8]),
            _ => panic!("expected bigint column"),
        }
        match accessor.get_column(&table_ref, &"i128".into()) {
            Column::Int128(col) => assert_eq!(col, &[-9, 10]),
            _ => panic!("expected int128 column"),
        }
        match accessor.get_column(&table_ref, &"decimal".into()) {
            Column::Decimal75(precision, scale, col) => {
                assert_eq!(precision.value(), 10);
                assert_eq!(scale, 2);
                assert_eq!(col, &[TestScalar::from(11), TestScalar::from(12)]);
            }
            _ => panic!("expected decimal column"),
        }
        match accessor.get_column(&table_ref, &"scalar".into()) {
            Column::Scalar(col) => assert_eq!(col, &[TestScalar::from(13), TestScalar::from(14)]),
            _ => panic!("expected scalar column"),
        }
        match accessor.get_column(&table_ref, &"text".into()) {
            Column::VarChar((strings, scalars)) => {
                assert_eq!(strings, &["a", "bc"]);
                assert_eq!(scalars, &[TestScalar::from("a"), TestScalar::from("bc")]);
            }
            _ => panic!("expected varchar column"),
        }
        match accessor.get_column(&table_ref, &"bytes".into()) {
            Column::VarBinary((bytes, scalars)) => {
                assert_eq!(bytes[0], &[1_u8, 2]);
                assert_eq!(bytes[1], &[3_u8, 4, 5]);
                assert_eq!(scalars.len(), 2);
            }
            _ => panic!("expected varbinary column"),
        }
        match accessor.get_column(&table_ref, &"time".into()) {
            Column::TimestampTZ(unit, zone, col) => {
                assert_eq!(unit, PoSQLTimeUnit::Second);
                assert_eq!(zone, PoSQLTimeZone::utc());
                assert_eq!(col, &[100, 200]);
            }
            _ => panic!("expected timestamp column"),
        }
    }

    #[test]
    fn table_test_accessor_builds_tables_and_updates_offsets() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "borrowed");
        let mut accessor = Accessor::new_from_table(table_ref.clone(), sample_table(&alloc), 0, ());

        let selected = indexset! {"i64".into(), "text".into()};
        let table = accessor.get_table(&table_ref, &selected);
        assert_eq!(table.num_columns(), 2);
        assert_eq!(table.num_rows(), 2);

        let empty = accessor.get_table(&table_ref, &indexset! {});
        assert_eq!(empty.num_columns(), 0);
        assert_eq!(empty.num_rows(), 2);

        let original_commitment = accessor.get_commitment(&table_ref, &"i64".into());
        accessor.update_offset(&table_ref, 5);
        assert_eq!(accessor.get_offset(&table_ref), 5);
        assert_ne!(
            original_commitment,
            accessor.get_commitment(&table_ref, &"i64".into())
        );

        assert_eq!(
            accessor.get_commitment(&table_ref, &"i64".into()),
            NaiveCommitment::compute_commitments(
                &[CommittableColumn::from(&[-7_i64, 8][..])],
                5,
                &()
            )[0]
        );
    }

    #[test]
    fn cloned_table_test_accessor_keeps_tables_and_setup() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "borrowed");
        let accessor = Accessor::new_from_table(table_ref.clone(), sample_table(&alloc), 3, ());
        let clone = accessor.clone();

        assert_eq!(clone.get_length(&table_ref), 2);
        assert_eq!(clone.get_offset(&table_ref), 3);
        assert_eq!(
            clone.get_commitment(&table_ref, &"u8".into()),
            NaiveCommitment::compute_commitments(
                &[CommittableColumn::from(&[1_u8, 2][..])],
                3,
                &()
            )[0]
        );
    }
}
