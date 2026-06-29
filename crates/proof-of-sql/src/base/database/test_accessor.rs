use super::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor, TableRef};
use crate::base::commitment::Commitment;
use alloc::vec::Vec;

/// A trait that defines the interface for a combined metadata, schema, commitment, and data accessor for unit testing or example purposes.
pub trait TestAccessor<C: Commitment>:
    Clone
    + Default
    + MetadataAccessor
    + SchemaAccessor
    + CommitmentAccessor<C>
    + DataAccessor<C::Scalar>
{
    /// The table type that the accessor will accept in the `add_table` method, and likely the inner table type.
    type Table;

    /// Create an empty test accessor
    fn new_empty() -> Self;

    /// Add a new table to the current test accessor
    fn add_table(&mut self, table_ref: TableRef, data: Self::Table, table_offset: usize);

    /// Get the column names for a given table
    fn get_column_names(&self, table_ref: &TableRef) -> Vec<&str>;

    /// Update the table offset alongside its column commitments
    fn update_offset(&mut self, table_ref: &TableRef, new_offset: usize);
}

#[cfg(test)]
mod tests {
    use super::TestAccessor;
    use crate::base::{
        commitment::{
            naive_commitment::NaiveCommitment, naive_evaluation_proof::NaiveEvaluationProof,
        },
        database::{
            owned_table_utility::{bigint, boolean, owned_table},
            table_utility::{borrowed_bigint, borrowed_boolean, table},
            ColumnType, OwnedTable, OwnedTableTestAccessor, Table, TableRef, TableTestAccessor,
        },
        scalar::test_scalar::TestScalar,
    };
    use bumpalo::Bump;

    fn assert_test_accessor_contract<A>(mut accessor: A, table: A::Table)
    where
        A: TestAccessor<NaiveCommitment>,
    {
        let table_ref = TableRef::new("sxt", "contract");

        accessor.add_table(table_ref.clone(), table, 2);

        assert_eq!(
            accessor.get_column_names(&table_ref),
            vec!["amount", "flag"]
        );
        assert_eq!(accessor.get_length(&table_ref), 3);
        assert_eq!(accessor.get_offset(&table_ref), 2);
        assert_eq!(
            accessor.lookup_column(&table_ref, &"amount".into()),
            Some(ColumnType::BigInt)
        );
        assert_eq!(
            accessor.lookup_schema(&table_ref),
            vec![
                ("amount".into(), ColumnType::BigInt),
                ("flag".into(), ColumnType::Boolean)
            ]
        );

        accessor.update_offset(&table_ref, 5);

        assert_eq!(accessor.get_offset(&table_ref), 5);
    }

    #[test]
    fn table_test_accessor_satisfies_the_test_accessor_contract() {
        let alloc = Bump::new();
        let table: Table<'_, TestScalar> = table([
            borrowed_bigint("amount", [10, 20, 30], &alloc),
            borrowed_boolean("flag", [true, false, true], &alloc),
        ]);
        let accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());

        assert_test_accessor_contract(accessor, table);
    }

    #[test]
    fn owned_table_test_accessor_satisfies_the_test_accessor_contract() {
        let owned_table: OwnedTable<TestScalar> = owned_table([
            bigint("amount", [10, 20, 30]),
            boolean("flag", [true, false, true]),
        ]);
        let accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());

        assert_test_accessor_contract(accessor, owned_table);
    }
}
