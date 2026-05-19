use super::{
    Column, ColumnField, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    NullableColumn, NullableDataAccessor, NullableOwnedTable, OwnedColumn, SchemaAccessor,
    TableRef, TestAccessor,
};
use crate::base::{
    commitment::{CommitmentEvaluationProof, VecCommitmentExt},
    map::IndexMap,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use sqlparser::ast::Ident;

/// A test accessor that uses [`NullableOwnedTable`] as the underlying table type.
///
/// Note: this is intended for testing and examples. It is not optimized for
/// performance, so should not be used for benchmarks or production use-cases.
pub struct NullableOwnedTableTestAccessor<'a, CP: CommitmentEvaluationProof> {
    tables: IndexMap<TableRef, (NullableOwnedTable<CP::Scalar>, usize)>,
    alloc: Bump,
    setup: Option<CP::ProverPublicSetup<'a>>,
}

impl<CP: CommitmentEvaluationProof> Default for NullableOwnedTableTestAccessor<'_, CP> {
    fn default() -> Self {
        Self {
            tables: IndexMap::default(),
            alloc: Bump::new(),
            setup: None,
        }
    }
}

impl<CP: CommitmentEvaluationProof> Clone for NullableOwnedTableTestAccessor<'_, CP> {
    fn clone(&self) -> Self {
        Self {
            tables: self.tables.clone(),
            setup: self.setup,
            ..Default::default()
        }
    }
}

impl<CP: CommitmentEvaluationProof> TestAccessor<CP::Commitment>
    for NullableOwnedTableTestAccessor<'_, CP>
{
    type Table = NullableOwnedTable<CP::Scalar>;

    fn new_empty() -> Self {
        NullableOwnedTableTestAccessor::default()
    }

    fn add_table(&mut self, table_ref: TableRef, data: Self::Table, table_offset: usize) {
        self.tables.insert(table_ref, (data, table_offset));
    }

    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`.
    fn get_column_names(&self, table_ref: &TableRef) -> Vec<&str> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .column_names()
            .map(|ident| ident.value.as_str())
            .collect()
    }

    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`.
    fn update_offset(&mut self, table_ref: &TableRef, new_offset: usize) {
        self.tables.get_mut(&table_ref).unwrap().1 = new_offset;
    }
}

impl<CP: CommitmentEvaluationProof> NullableDataAccessor<CP::Scalar>
    for NullableOwnedTableTestAccessor<'_, CP>
{
    /// # Panics
    ///
    /// Will panic if the table or column reference is missing.
    fn get_nullable_column(
        &self,
        table_ref: &TableRef,
        column_id: &Ident,
    ) -> NullableColumn<'_, CP::Scalar> {
        let owned_column = self
            .tables
            .get(table_ref)
            .unwrap()
            .0
            .inner_table()
            .get(column_id)
            .unwrap();
        NullableColumn::from_owned_column(owned_column, &self.alloc)
            .expect("Nullable owned columns should borrow without length mismatches")
    }
}

impl<CP: CommitmentEvaluationProof> DataAccessor<CP::Scalar>
    for NullableOwnedTableTestAccessor<'_, CP>
{
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<'_, CP::Scalar> {
        let table = &self.tables.get(table_ref).unwrap().0;
        if let Some(owned_column) = table.inner_table().get(column_id) {
            return Column::from_owned_column(owned_column.values(), &self.alloc);
        }
        let (presence, len) = presence_column(table, column_id).unwrap();
        Column::Boolean(match presence {
            Some(presence) => self.alloc.alloc_slice_copy(presence),
            None => self.alloc.alloc_slice_fill_copy(len, true),
        })
    }
}

impl<CP: CommitmentEvaluationProof> CommitmentAccessor<CP::Commitment>
    for NullableOwnedTableTestAccessor<'_, CP>
{
    fn get_commitment(&self, table_ref: &TableRef, column_id: &Ident) -> CP::Commitment {
        let (table, offset) = self.tables.get(table_ref).unwrap();
        if let Some(nullable_owned_column) = table.inner_table().get(column_id) {
            Vec::<CP::Commitment>::from_columns_with_offset(
                [nullable_owned_column.values()],
                *offset,
                self.setup.as_ref().unwrap(),
            )[0]
            .clone()
        } else {
            let (presence, len) = presence_column(table, column_id).unwrap();
            let owned_presence = OwnedColumn::<CP::Scalar>::Boolean(
                presence.map_or_else(|| alloc::vec![true; len], |presence| presence.to_vec()),
            );
            Vec::<CP::Commitment>::from_columns_with_offset(
                [&owned_presence],
                *offset,
                self.setup.as_ref().unwrap(),
            )[0]
            .clone()
        }
    }
}

impl<CP: CommitmentEvaluationProof> MetadataAccessor for NullableOwnedTableTestAccessor<'_, CP> {
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`.
    fn get_length(&self, table_ref: &TableRef) -> usize {
        self.tables.get(table_ref).unwrap().0.num_rows()
    }

    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`.
    fn get_offset(&self, table_ref: &TableRef) -> usize {
        self.tables.get(table_ref).unwrap().1
    }
}

impl<CP: CommitmentEvaluationProof> SchemaAccessor for NullableOwnedTableTestAccessor<'_, CP> {
    fn lookup_column(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType> {
        let table = &self.tables.get(table_ref)?.0;
        table
            .inner_table()
            .get(column_id)
            .map(|column| column.values().column_type())
            .or_else(|| presence_column(table, column_id).map(|_| ColumnType::Boolean))
    }

    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`.
    fn lookup_schema(&self, table_ref: &TableRef) -> Vec<(Ident, ColumnType)> {
        self.tables
            .get(table_ref)
            .unwrap()
            .0
            .inner_table()
            .iter()
            .map(|(id, col)| (id.clone(), col.values().column_type()))
            .collect()
    }

    fn lookup_column_field(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnField> {
        let table = &self.tables.get(table_ref)?.0;
        if presence_column(table, column_id).is_some() {
            return Some(ColumnField::new(column_id.clone(), ColumnType::Boolean));
        }
        let column = table.inner_table().get(column_id)?;
        Some(if column.is_nullable() {
            ColumnField::new_nullable(column_id.clone(), column.values().column_type())
        } else {
            ColumnField::new(column_id.clone(), column.values().column_type())
        })
    }

    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`.
    fn lookup_column_fields(&self, table_ref: &TableRef) -> Vec<ColumnField> {
        self.tables.get(table_ref).unwrap().0.schema()
    }
}

fn presence_column<'a, S: crate::base::scalar::Scalar>(
    table: &'a NullableOwnedTable<S>,
    presence_column_id: &Ident,
) -> Option<(Option<&'a [bool]>, usize)> {
    table.inner_table().iter().find_map(|(column_id, column)| {
        (NullableOwnedTable::<S>::presence_column_name(column_id) == *presence_column_id)
            .then(|| (column.presence(), column.len()))
    })
}

impl<'a, CP: CommitmentEvaluationProof> NullableOwnedTableTestAccessor<'a, CP> {
    /// Create a new empty test accessor with the given setup.
    pub fn new_empty_with_setup(setup: CP::ProverPublicSetup<'a>) -> Self {
        let mut res = Self::new_empty();
        res.setup = Some(setup);
        res
    }

    /// Create a new test accessor containing the provided table.
    pub fn new_from_table(
        table_ref: TableRef,
        nullable_owned_table: NullableOwnedTable<CP::Scalar>,
        offset: usize,
        setup: CP::ProverPublicSetup<'a>,
    ) -> Self {
        let mut res = Self::new_empty_with_setup(setup);
        res.add_table(table_ref, nullable_owned_table, offset);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{ColumnRef, NullableOwnedColumn, OwnedColumn},
        map::{indexmap, IndexSet},
        scalar::test_scalar::TestScalar,
    };

    fn nullable_table() -> NullableOwnedTable<TestScalar> {
        NullableOwnedTable::try_new(indexmap! {
            "id".into() => NullableOwnedColumn::new_nonnullable(
                OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3])
            ),
            "amount".into() => NullableOwnedColumn::try_new(
                OwnedColumn::<TestScalar>::BigInt(vec![10, 0, 30]),
                Some(vec![true, false, true])
            ).unwrap(),
        })
        .unwrap()
    }

    #[test]
    fn nullable_accessor_returns_values_and_presence() {
        let table_ref = TableRef::new("sxt", "nullable");
        let accessor = NullableOwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            nullable_table(),
            0,
            (),
        );

        let nullable_column = accessor.get_nullable_column(&table_ref, &"amount".into());

        assert_eq!(nullable_column.values(), Column::BigInt(&[10, 0, 30]));
        assert_eq!(nullable_column.presence(), Some(&[true, false, true][..]));
        assert_eq!(
            accessor.get_column(&table_ref, &"amount".into()),
            Column::BigInt(&[10, 0, 30])
        );
        assert_eq!(
            accessor.get_column(
                &table_ref,
                &ColumnRef::presence_column_id(&Ident::new("amount"))
            ),
            Column::Boolean(&[true, false, true])
        );
    }

    #[test]
    fn nullable_accessor_builds_nullable_tables_from_column_ids() {
        let table_ref = TableRef::new("sxt", "nullable");
        let accessor = NullableOwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            nullable_table(),
            0,
            (),
        );
        let column_ids = IndexSet::from_iter(["id".into(), "amount".into()]);

        let table = accessor.get_nullable_table(&table_ref, &column_ids);
        let amount = table.inner_table().get(&Ident::new("amount")).unwrap();

        assert_eq!(table.num_rows(), 3);
        assert_eq!(amount.values(), Column::BigInt(&[10, 0, 30]));
        assert_eq!(amount.presence(), Some(&[true, false, true][..]));
    }

    #[test]
    fn nullable_accessor_preserves_nullable_schema_fields() {
        let table_ref = TableRef::new("sxt", "nullable");
        let accessor = NullableOwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            nullable_table(),
            0,
            (),
        );

        let amount_field = accessor
            .lookup_column_field(&table_ref, &Ident::new("amount"))
            .unwrap();
        let fields = accessor.lookup_column_fields(&table_ref);

        assert_eq!(amount_field.name(), Ident::new("amount"));
        assert_eq!(amount_field.data_type(), ColumnType::BigInt);
        assert!(amount_field.is_nullable());
        assert!(!fields[0].is_nullable());
        assert!(fields[1].is_nullable());
        assert_eq!(
            accessor.lookup_schema(&table_ref),
            vec![
                ("id".into(), ColumnType::BigInt),
                ("amount".into(), ColumnType::BigInt)
            ]
        );

        let presence_field = accessor
            .lookup_column_field(
                &table_ref,
                &ColumnRef::presence_column_id(&Ident::new("amount")),
            )
            .unwrap();
        assert_eq!(
            presence_field,
            ColumnField::new(
                ColumnRef::presence_column_id(&Ident::new("amount")),
                ColumnType::Boolean
            )
        );
    }
}
