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
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<S>;

    /// Creates a new [`Table`] from a [`TableRef`] and [`Ident`]s.
    ///
    /// Columns are retrieved from the [`DataAccessor`] using the provided [`TableRef`] and [`Ident`]s.
    /// The only reason why [`table_ref`] is needed is because [`column_ids`] can be empty.
    /// # Panics
    /// Column length mismatches can occur in theory. In practice, this should not happen.
    fn get_table(&self, table_ref: &TableRef, column_ids: &IndexSet<Ident>) -> Table<S> {
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
