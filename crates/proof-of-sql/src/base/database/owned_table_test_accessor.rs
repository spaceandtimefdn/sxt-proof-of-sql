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
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::owned_table_utility::*,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
    };
    use alloc::vec;

    #[test]
    fn we_can_access_every_owned_column_variant() {
        let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
        let table_ref = TableRef::new("sxt", "variant_table");
        let table = owned_table([
            boolean("boolean", [true, false, true]),
            uint8("uint8", [1_u8, 2, 3]),
            tinyint("tinyint", [-1_i8, 0, 1]),
            smallint("smallint", [-10_i16, 0, 10]),
            int("int", [-100_i32, 0, 100]),
            bigint("bigint", [-1000_i64, 0, 1000]),
            int128("int128", [-10000_i128, 0, 10000]),
            decimal75("decimal", 12, 2, [101, 202, 303]),
            scalar("scalar", [11, 22, 33]),
            varchar("varchar", ["alpha", "beta", ""]),
            varbinary("bytes", [b"alpha".to_vec(), b"beta".to_vec(), b"".to_vec()]),
            timestamptz(
                "time",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [1_700_000_000, 1_700_000_001, 1_700_000_002],
            ),
        ]);

        accessor.add_table(table_ref.clone(), table, 4);

        assert_eq!(accessor.get_length(&table_ref), 3);
        assert_eq!(accessor.get_offset(&table_ref), 4);
        assert_eq!(
            accessor.get_column_names(&table_ref),
            vec![
                "boolean", "uint8", "tinyint", "smallint", "int", "bigint", "int128", "decimal",
                "scalar", "varchar", "bytes", "time"
            ]
        );

        match accessor.get_column(&table_ref, &"boolean".into()) {
            Column::Boolean(col) => assert_eq!(col, &[true, false, true]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"uint8".into()) {
            Column::Uint8(col) => assert_eq!(col, &[1, 2, 3]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"tinyint".into()) {
            Column::TinyInt(col) => assert_eq!(col, &[-1, 0, 1]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"smallint".into()) {
            Column::SmallInt(col) => assert_eq!(col, &[-10, 0, 10]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"int".into()) {
            Column::Int(col) => assert_eq!(col, &[-100, 0, 100]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"bigint".into()) {
            Column::BigInt(col) => assert_eq!(col, &[-1000, 0, 1000]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"int128".into()) {
            Column::Int128(col) => assert_eq!(col, &[-10000, 0, 10000]),
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"decimal".into()) {
            Column::Decimal75(precision, scale, col) => {
                assert_eq!(precision.value(), 12);
                assert_eq!(scale, 2);
                assert_eq!(col, &[101.into(), 202.into(), 303.into()]);
            }
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"scalar".into()) {
            Column::Scalar(col) => {
                assert_eq!(col, &[11.into(), 22.into(), 33.into()]);
            }
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"varchar".into()) {
            Column::VarChar((strings, scalars)) => {
                assert_eq!(strings, &["alpha", "beta", ""]);
                assert_eq!(scalars, &["alpha".into(), "beta".into(), TestScalar::ZERO]);
            }
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"bytes".into()) {
            Column::VarBinary((bytes, scalars)) => {
                assert_eq!(bytes, &[b"alpha".as_ref(), b"beta".as_ref(), b"".as_ref()]);
                assert_eq!(
                    scalars,
                    &[
                        TestScalar::from_byte_slice_via_hash(b"alpha"),
                        TestScalar::from_byte_slice_via_hash(b"beta"),
                        TestScalar::ZERO
                    ]
                );
            }
            _ => panic!("Invalid column type"),
        }
        match accessor.get_column(&table_ref, &"time".into()) {
            Column::TimestampTZ(time_unit, timezone, col) => {
                assert_eq!(time_unit, PoSQLTimeUnit::Second);
                assert_eq!(timezone, PoSQLTimeZone::utc());
                assert_eq!(col, &[1_700_000_000, 1_700_000_001, 1_700_000_002]);
            }
            _ => panic!("Invalid column type"),
        }
    }
}
