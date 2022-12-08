use crate::base::database::{Column, ColumnType};
use curve25519_dalek::ristretto::RistrettoPoint;

/// Access metadata of tables in a database.
///
/// Both Prover and Verifier use this information when processing a query.
///
/// Note: we assume that the query has already been validated so that we
/// will only be accessing information about tables that exist in the database.
pub trait MetadataAccessor {
    fn get_length(&self, table_name: &str) -> usize;
}

/// Access commitments of database columns.
///
/// Verifier uses this information to process a query.
///
/// In pseudo-code, here is a sketch of how CommitmentAccessor fits in
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
pub trait CommitmentAccessor: MetadataAccessor {
    fn get_commitment(&self, table_name: &str, column_name: &str) -> RistrettoPoint;
}

/// Access database columns of an in-memory database.
///
/// Prover uses this information to process a query.
///
/// In pseudo-code, here is a sketch of how DataAccessor fits in
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
pub trait DataAccessor: MetadataAccessor {
    fn get_column(&self, table_name: &str, column_name: &str) -> Column;
}

/// Access tables and their schemas in a database.
///
/// This accessor should be implemented by both the prover and verifier
/// and then used by the proofs code to convert an IntermediateAst
/// into a ProvableAst.
pub trait SchemaAccessor {
    /// Lookup the column's data type in the specified table
    ///
    /// Return:
    ///   - Some(type) if the column exists, where `type` is the column's data type
    ///   - None in case the column does not exist in the table
    ///
    /// Precondition 1: the table must exist and be tamperproof.
    /// Precondition 2: `table_name` and `column_name` must always be lowercase.
    fn lookup_column(&self, table_name: &str, column_name: &str) -> Option<ColumnType>;

    /// Lookup all the column names and their data types in the specified table
    ///
    /// Return:
    ///   - The list of column names with their data types
    ///
    /// Precondition 1: the table must exist and be tamperproof.
    /// Precondition 2: `table_name` must be lowercase.
    fn lookup_schema<'a>(&'a self, table_name: &str) -> Vec<(&'a str, ColumnType)>;
}
