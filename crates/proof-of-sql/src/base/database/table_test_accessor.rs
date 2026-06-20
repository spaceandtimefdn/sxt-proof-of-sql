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
        },
        database::table_utility::{borrowed_bigint, borrowed_varchar, table},
        scalar::test_scalar::TestScalar,
    };
    use bumpalo::Bump;

    #[test]
    fn table_test_accessor_exposes_data_metadata_schema_and_commitments() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "orders");
        let amount_id: Ident = "amount".into();
        let customer_id: Ident = "customer".into();
        let table = table::<TestScalar>([
            borrowed_bigint(amount_id.clone(), [10_i64, 20, -5], &alloc),
            borrowed_varchar(customer_id.clone(), ["alice", "bob", "carol"], &alloc),
        ]);

        let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            table,
            2,
            (),
        );

        assert_eq!(
            accessor.get_column_names(&table_ref),
            vec![amount_id.value.as_str(), customer_id.value.as_str()]
        );
        assert_eq!(accessor.get_length(&table_ref), 3);
        assert_eq!(accessor.get_offset(&table_ref), 2);
        assert_eq!(
            accessor.lookup_column(&table_ref, &amount_id),
            Some(ColumnType::BigInt)
        );
        assert_eq!(accessor.lookup_column(&table_ref, &"missing".into()), None);
        assert_eq!(
            accessor.lookup_schema(&table_ref),
            vec![
                (amount_id.clone(), ColumnType::BigInt),
                (customer_id, ColumnType::VarChar),
            ]
        );
        assert!(matches!(
            accessor.get_column(&table_ref, &amount_id),
            Column::BigInt([10, 20, -5])
        ));
        assert_eq!(
            accessor.get_commitment(&table_ref, &amount_id),
            NaiveCommitment(vec![
                0_i64.into(),
                0_i64.into(),
                10_i64.into(),
                20_i64.into(),
                (-5_i64).into(),
            ])
        );

        accessor.update_offset(&table_ref, 4);
        assert_eq!(accessor.get_offset(&table_ref), 4);
        let cloned = accessor.clone();
        assert_eq!(cloned.get_offset(&table_ref), 4);
        assert_eq!(cloned.get_length(&table_ref), 3);
    }
}
