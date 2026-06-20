use super::{
    Column, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedColumn,
    OwnedTable, SchemaAccessor, TableRef, TestAccessor,
};
use crate::base::{
    commitment::{CommitmentEvaluationProof, VecCommitmentExt},
    map::IndexMap,
    scalar::ScalarExt,
};
use alloc::{string::String, vec::Vec};
use bumpalo::Bump;
use sqlparser::ast::Ident;
/// A test accessor that uses [`OwnedTable`] as the underlying table type.
/// Note: this is intended for testing and examples. It is not optimized for performance, so should not be used for benchmarks or production use-cases.
pub struct OwnedTableTestAccessor<'a, CP: CommitmentEvaluationProof> {
    tables: IndexMap<TableRef, (OwnedTable<CP::Scalar>, usize)>,
    alloc: Bump,
    setup: Option<CP::ProverPublicSetup<'a>>,
}

impl<CP: CommitmentEvaluationProof> Default for OwnedTableTestAccessor<'_, CP> {
    fn default() -> Self {
        Self {
            tables: IndexMap::default(),
            alloc: Bump::new(),
            setup: None,
        }
    }
}

impl<CP: CommitmentEvaluationProof> Clone for OwnedTableTestAccessor<'_, CP> {
    fn clone(&self) -> Self {
        Self {
            tables: self.tables.clone(),
            setup: self.setup,
            ..Default::default()
        }
    }
}

impl<CP: CommitmentEvaluationProof> TestAccessor<CP::Commitment>
    for OwnedTableTestAccessor<'_, CP>
{
    type Table = OwnedTable<CP::Scalar>;

    fn new_empty() -> Self {
        OwnedTableTestAccessor::default()
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
impl<CP: CommitmentEvaluationProof> DataAccessor<CP::Scalar> for OwnedTableTestAccessor<'_, CP> {
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<'_, CP::Scalar> {
        match self
            .tables
            .get(table_ref)
            .unwrap()
            .0
            .inner_table()
            .get(column_id)
            .unwrap()
        {
            OwnedColumn::Boolean(col) => Column::Boolean(col),
            OwnedColumn::TinyInt(col) => Column::TinyInt(col),
            OwnedColumn::Uint8(col) => Column::Uint8(col),
            OwnedColumn::SmallInt(col) => Column::SmallInt(col),
            OwnedColumn::Int(col) => Column::Int(col),
            OwnedColumn::BigInt(col) => Column::BigInt(col),
            OwnedColumn::Int128(col) => Column::Int128(col),
            OwnedColumn::Decimal75(precision, scale, col) => {
                Column::Decimal75(*precision, *scale, col)
            }
            OwnedColumn::Scalar(col) => Column::Scalar(col),
            OwnedColumn::VarChar(col) => {
                let col: &mut [&str] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(String::as_str));
                let scals: &mut [_] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(|s| (*s).into()));
                Column::VarChar((col, scals))
            }
            OwnedColumn::VarBinary(col) => {
                // Convert each `Vec<u8>` to `&[u8]` for the `Column::VarBinary` "string-like" part.
                let col_as_slices: &mut [&[u8]] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(Vec::as_slice));

                // Convert each `Vec<u8>` to a scalar by calling `from_le_bytes_mod_order`.
                // That is the crucial step, because there's no direct `From<&[u8]>`.
                let scals: &mut [CP::Scalar] = self.alloc.alloc_slice_fill_iter(
                    col.iter()
                        .map(|b| CP::Scalar::from_byte_slice_via_hash(b.as_slice())),
                );

                Column::VarBinary((col_as_slices, scals))
            }
            OwnedColumn::TimestampTZ(tu, tz, col) => Column::TimestampTZ(*tu, *tz, col),
        }
    }
}

///
/// # Panics
///
/// Will panic if the `table_ref` is not found in `self.tables`, or if the `column_id` is not found in the inner table for that reference,indicating that an invalid column reference was provided.
impl<CP: CommitmentEvaluationProof> CommitmentAccessor<CP::Commitment>
    for OwnedTableTestAccessor<'_, CP>
{
    fn get_commitment(&self, table_ref: &TableRef, column_id: &Ident) -> CP::Commitment {
        let (table, offset) = self.tables.get(table_ref).unwrap();
        let owned_column = table.inner_table().get(column_id).unwrap();
        Vec::<CP::Commitment>::from_columns_with_offset(
            [owned_column],
            *offset,
            self.setup.as_ref().unwrap(),
        )[0]
        .clone()
    }
}
impl<CP: CommitmentEvaluationProof> MetadataAccessor for OwnedTableTestAccessor<'_, CP> {
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
impl<CP: CommitmentEvaluationProof> SchemaAccessor for OwnedTableTestAccessor<'_, CP> {
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

impl<'a, CP: CommitmentEvaluationProof> OwnedTableTestAccessor<'a, CP> {
    /// Create a new empty test accessor with the given setup.
    pub fn new_empty_with_setup(setup: CP::ProverPublicSetup<'a>) -> Self {
        let mut res = Self::new_empty();
        res.setup = Some(setup);
        res
    }

    /// Create a new test accessor containing the provided table.
    pub fn new_from_table(
        table_ref: TableRef,
        owned_table: OwnedTable<CP::Scalar>,
        offset: usize,
        setup: CP::ProverPublicSetup<'a>,
    ) -> Self {
        let mut res = Self::new_empty_with_setup(setup);
        res.add_table(table_ref, owned_table, offset);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        commitment::{
            naive_commitment::NaiveCommitment, naive_evaluation_proof::NaiveEvaluationProof,
        },
        database::owned_table_utility::{
            bigint, boolean, decimal75, int, int128, owned_table, scalar, smallint, timestamptz,
            tinyint, uint8, varbinary, varchar,
        },
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn owned_table_test_accessor_exposes_owned_column_variants() {
        let table_ref = TableRef::new("sxt", "mixed");
        let table = owned_table::<TestScalar>([
            boolean("flag", [true, false]),
            uint8("byte", [3_u8, 4]),
            tinyint("tiny", [-1_i8, 2]),
            smallint("small", [-10_i16, 20]),
            int("int_col", [-100_i32, 200]),
            bigint("big", [-1000_i64, 2000]),
            int128("huge", [-10000_i128, 20000]),
            decimal75("amount", 12, 2, [123_i64, -456]),
            scalar("scalar_col", [7_i64, 8]),
            varchar("name", ["alice", "bob"]),
            varbinary("payload", [vec![1_u8, 2], vec![3, 4, 5]]),
            timestamptz(
                "created_at",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [1_700_000_000_i64, 1_700_000_001],
            ),
        ]);

        let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            table,
            1,
            (),
        );

        assert_eq!(accessor.get_length(&table_ref), 2);
        assert_eq!(accessor.get_offset(&table_ref), 1);
        assert_eq!(
            accessor.get_column_names(&table_ref),
            vec![
                "flag",
                "byte",
                "tiny",
                "small",
                "int_col",
                "big",
                "huge",
                "amount",
                "scalar_col",
                "name",
                "payload",
                "created_at",
            ]
        );
        assert_eq!(
            accessor.lookup_column(&table_ref, &"amount".into()),
            Some(ColumnType::Decimal75(
                crate::base::math::decimal::Precision::new(12).unwrap(),
                2,
            ))
        );
        assert_eq!(
            accessor.lookup_column(&TableRef::new("missing", "table"), &"amount".into()),
            None
        );
        assert_eq!(accessor.lookup_schema(&table_ref).len(), 12);

        assert!(matches!(
            accessor.get_column(&table_ref, &"flag".into()),
            Column::Boolean([true, false])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"byte".into()),
            Column::Uint8([3, 4])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"tiny".into()),
            Column::TinyInt([-1, 2])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"small".into()),
            Column::SmallInt([-10, 20])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"int_col".into()),
            Column::Int([-100, 200])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"big".into()),
            Column::BigInt([-1000, 2000])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"huge".into()),
            Column::Int128([-10000, 20000])
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"amount".into()),
            Column::Decimal75(_, 2, values) if values.len() == 2
        ));
        assert!(matches!(
            accessor.get_column(&table_ref, &"scalar_col".into()),
            Column::Scalar(values) if values.len() == 2
        ));
        match accessor.get_column(&table_ref, &"name".into()) {
            Column::VarChar((values, scalars)) => {
                assert_eq!(values, &["alice", "bob"]);
                assert_eq!(scalars.len(), 2);
            }
            other => panic!("unexpected column: {other:?}"),
        }
        match accessor.get_column(&table_ref, &"payload".into()) {
            Column::VarBinary((values, scalars)) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], &[1, 2]);
                assert_eq!(values[1], &[3, 4, 5]);
                assert_eq!(scalars.len(), 2);
            }
            other => panic!("unexpected column: {other:?}"),
        }
        assert!(matches!(
            accessor.get_column(&table_ref, &"created_at".into()),
            Column::TimestampTZ(PoSQLTimeUnit::Second, _, [1_700_000_000, 1_700_000_001])
        ));
        assert_eq!(
            accessor.get_commitment(&table_ref, &"big".into()),
            NaiveCommitment(vec![0_i64.into(), (-1000_i64).into(), 2000_i64.into()])
        );

        accessor.update_offset(&table_ref, 3);
        assert_eq!(accessor.get_offset(&table_ref), 3);
        let cloned = accessor.clone();
        assert_eq!(cloned.get_length(&table_ref), 2);
        assert_eq!(cloned.get_offset(&table_ref), 3);
    }
}
