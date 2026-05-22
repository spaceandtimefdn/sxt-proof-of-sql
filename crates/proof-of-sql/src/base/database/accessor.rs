use crate::base::{
    commitment::Commitment,
    database::{Column, ColumnType, Table, TableOptions, TableRef},
    map::{IndexMap, IndexSet},
    scalar::Scalar,
};
use alloc::vec::Vec;
use sqlparser::ast::Ident;

/// Access metadata of a table span in a database.
///
/// Both Prover and Verifier use this information when processing a query.
///
/// Note: we assume that the query has already been validated so that we
/// will only be accessing information about tables that exist in the database.
pub trait MetadataAccessor {
    /// Return the data span's length in the table (not the full table length)
    fn get_length(&self, table_ref: &TableRef) -> usize;

    /// Return the data span's offset in the table
    ///
    /// If the data span has its first row starting at the ith table row,
    /// this `get_offset` should then return `i`.
    fn get_offset(&self, table_ref: &TableRef) -> usize;
}

/// Access commitments of database columns.
///
/// Verifier uses this information to process a query.
///
/// In pseudo-code, here is a sketch of how [`CommitmentAccessor`] fits in
/// with the verification workflow:
///
/// ```ignore
/// verify(proof, query, commitment_database) {
///     if(!validate_query(query, commitment_database)) {
///         // if the query references columns that don't exist
///         // we should error here before going any further
///         return invalid-query()
///     }
///     commitment_database.reader_lock()
///     // we can't be updating commitments while verifying
///     accessor <- make-commitment-accessor(commitment_database)
///     verify_result <- verify-valid-query(proof, query, accessor)
///     commitment_database.reader_unlock()
///     return verify_result
/// }
/// ```
///
/// Note: we assume that the query has already been validated so that we
/// will only be accessing information about columns that exist in the database.
pub trait CommitmentAccessor<C: Commitment>: MetadataAccessor {
    /// Return the full table column commitment
    fn get_commitment(&self, table_ref: &TableRef, column_id: &Ident) -> C;
}

/// Access database columns of an in-memory table span.
///
/// Prover uses this information to process a query.
///
/// In pseudo-code, here is a sketch of how [`DataAccessor`] fits in
/// with the prove workflow:
///
/// ```ignore
/// prove(query, database) {
///       if(!validate_query(query, database)) {
///           // if the query references columns that don't exist
///           // we should error here before going any further
///           invalid-query()
///       }
///       update-cached-columns(database, query)
///            // if the database represents an in-memory cache of an externally persisted
///            // database we should update the cache so that any column referenced in the query
///            // will be available
///       database.reader_lock()
///           // we can't be updating the database while proving
///       accessor <- make-data-accessor(database)
///       proof <- prove-valid-query(query, accessor)
///       database.reader_unlock()
///       return proof
/// }
/// ```
///
/// Note: we assume that the query has already been validated so that we
/// will only be accessing information about columns that exist in the database.
pub trait DataAccessor<S: Scalar>: MetadataAccessor {
    /// Return the data span in the table (not the full-table data)
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<'_, S>;

    /// Creates a new [`Table`] from a [`TableRef`] and [`Ident`]s.
    ///
    /// Columns are retrieved from the [`DataAccessor`] using the provided [`TableRef`] and [`Ident`]s.
    /// The only reason why [`table_ref`] is needed is because [`column_ids`] can be empty.
    /// # Panics
    /// Column length mismatches can occur in theory. In practice, this should not happen.
    fn get_table(&self, table_ref: &TableRef, column_ids: &IndexSet<Ident>) -> Table<'_, S> {
        if column_ids.is_empty() {
            let input_length = self.get_length(table_ref);
            Table::<S>::try_new_with_options(
                IndexMap::default(),
                TableOptions::new(Some(input_length)),
            )
        } else {
            Table::<S>::try_from_iter(column_ids.into_iter().map(|column_id| {
                let column = self.get_column(table_ref, column_id);
                (column_id.clone(), column)
            }))
        }
        .expect("Failed to create table from table and column references")
    }
}

/// Access tables and their schemas in a database.
///
/// This accessor should be implemented by both the prover and verifier
/// and then used by the Proof of SQL code to convert an `IntermediateAst`
/// into a [`ProofPlan`](crate::sql::proof::ProofPlan).
pub trait SchemaAccessor {
    /// Lookup the column's data type in the specified table
    ///
    /// Return:
    ///   - Some(type) if the column exists, where `type` is the column's data type
    ///   - None in case the column does not exist in the table
    ///
    /// Precondition 1: the table must exist and be tamperproof.
    /// Precondition 2: `table_ref` and `column_id` must always be lowercase.
    fn lookup_column(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType>;

    /// Lookup all the column names and their data types in the specified table
    ///
    /// Return:
    ///   - The list of column names with their data types
    ///
    /// Precondition 1: the table must exist and be tamperproof.
    /// Precondition 2: `table_name` must be lowercase.
    fn lookup_schema(&self, table_ref: &TableRef) -> Vec<(Ident, ColumnType)>;
}

/// The simplest implementation of `SchemaAccessor`.
/// This is effectively an in-memory mapping from table to the schema.
#[derive(Clone)]
pub struct SchemaAccessorImpl {
    table_schema_lookup: IndexMap<TableRef, Vec<(Ident, ColumnType)>>,
}

impl SchemaAccessorImpl {
    /// Constructs a new `SchemaAccessorImpl` implementation
    #[must_use]
    pub fn new(table_schema_lookup: IndexMap<TableRef, Vec<(Ident, ColumnType)>>) -> Self {
        Self {
            table_schema_lookup,
        }
    }
}

impl SchemaAccessor for SchemaAccessorImpl {
    /// # Panics
    ///
    /// Panics if the table does not exist
    fn lookup_column(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType> {
        self.table_schema_lookup
            .get(table_ref)
            .expect("Table does not exist in schema accessor.")
            .iter()
            .find_map(|(id, column_type)| (id == column_id).then_some(*column_type))
    }

    /// # Panics
    ///
    /// Panics if the table does not exist
    fn lookup_schema(&self, table_ref: &TableRef) -> Vec<(Ident, ColumnType)> {
        self.table_schema_lookup
            .get(table_ref)
            .expect("Table does not exist in schema accessor.")
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::ColumnField,
        map::{indexmap, indexset},
        scalar::test_scalar::TestScalar,
    };

    fn sample_schema_accessor() -> SchemaAccessorImpl {
        let table1 = TableRef::new("schema", "table1");
        let table2 = TableRef::new("schema", "table2");
        SchemaAccessorImpl::new(indexmap! {
            table1 => vec![("col1".into(), ColumnType::BigInt),
                ("col2".into(), ColumnType::VarChar)],
            table2 => vec![("col1".into(), ColumnType::BigInt)],
        })
    }

    #[test]
    fn test_lookup_column() {
        let accessor = sample_schema_accessor();
        let table1 = TableRef::new("schema", "table1");
        let table2 = TableRef::new("schema", "table2");
        assert_eq!(
            accessor.lookup_column(&table1, &"col1".into()),
            Some(ColumnType::BigInt)
        );
        assert_eq!(
            accessor.lookup_column(&table1, &"col2".into()),
            Some(ColumnType::VarChar)
        );
        assert_eq!(accessor.lookup_column(&table1, &"not_a_col".into()), None);
        assert_eq!(
            accessor.lookup_column(&table2, &"col1".into()),
            Some(ColumnType::BigInt)
        );
        assert_eq!(accessor.lookup_column(&table2, &"col2".into()), None);
    }

    #[test]
    fn test_lookup_non_existent_column_on_existing_table() {
        let accessor = sample_schema_accessor();
        let table1 = TableRef::new("schema", "table1");
        assert_eq!(accessor.lookup_column(&table1, &"col3".into()), None);
    }

    #[test]
    #[should_panic(expected = "Table does not exist in schema accessor.")]
    fn test_lookup_column_on_non_existent_table() {
        let accessor = sample_schema_accessor();
        let not_a_table = TableRef::new("schema", "not_a_table");
        accessor.lookup_column(&not_a_table, &"col1".into());
    }

    #[test]
    fn test_lookup_schema() {
        let accessor = sample_schema_accessor();
        let table1 = TableRef::new("schema", "table1");
        let table2 = TableRef::new("schema", "table2");
        assert_eq!(
            accessor.lookup_schema(&table1),
            vec![
                ("col1".into(), ColumnType::BigInt),
                ("col2".into(), ColumnType::VarChar),
            ]
        );
        assert_eq!(
            accessor.lookup_schema(&table2),
            vec![("col1".into(), ColumnType::BigInt),]
        );
    }

    #[test]
    #[should_panic(expected = "Table does not exist in schema accessor.")]
    fn test_lookup_non_existent_schema() {
        let accessor = sample_schema_accessor();
        let not_a_table = TableRef::new("schema", "not_a_table");
        accessor.lookup_schema(&not_a_table);
    }

    struct SampleDataAccessor {
        table_ref: TableRef,
        flags: Vec<bool>,
        amounts: Vec<i64>,
    }

    impl SampleDataAccessor {
        fn new() -> Self {
            Self {
                table_ref: TableRef::new("schema", "table"),
                flags: vec![true, false, true],
                amounts: vec![10, 20, 30],
            }
        }
    }

    impl MetadataAccessor for SampleDataAccessor {
        fn get_length(&self, table_ref: &TableRef) -> usize {
            assert_eq!(table_ref, &self.table_ref);
            self.amounts.len()
        }

        fn get_offset(&self, table_ref: &TableRef) -> usize {
            assert_eq!(table_ref, &self.table_ref);
            7
        }
    }

    impl DataAccessor<TestScalar> for SampleDataAccessor {
        fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<'_, TestScalar> {
            assert_eq!(table_ref, &self.table_ref);
            match column_id.value.as_str() {
                "flag" => Column::Boolean(&self.flags),
                "amount" => Column::BigInt(&self.amounts),
                _ => panic!("unexpected column id: {column_id}"),
            }
        }
    }

    #[test]
    fn get_table_preserves_row_count_for_empty_column_selection() {
        let accessor = SampleDataAccessor::new();
        let column_ids = IndexSet::default();
        let table = accessor.get_table(&accessor.table_ref, &column_ids);

        assert_eq!(table.num_columns(), 0);
        assert_eq!(table.num_rows(), accessor.amounts.len());
    }

    #[test]
    fn get_table_materializes_selected_columns_in_requested_order() {
        let accessor = SampleDataAccessor::new();
        let column_ids = indexset! { Ident::new("flag"), Ident::new("amount") };
        let table = accessor.get_table(&accessor.table_ref, &column_ids);

        assert_eq!(
            table.schema(),
            vec![
                ColumnField::new("flag".into(), ColumnType::Boolean),
                ColumnField::new("amount".into(), ColumnType::BigInt),
            ]
        );
        assert_eq!(table["flag"], Column::Boolean(&accessor.flags));
        assert_eq!(table["amount"], Column::BigInt(&accessor.amounts));
    }
}
