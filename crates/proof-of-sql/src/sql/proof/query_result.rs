use crate::base::{
    database::{ColumnCoercionError, OwnedTable, OwnedTableError, TableCoercionError},
    proof::ProofError,
    scalar::Scalar,
};
use snafu::Snafu;

/// Verifiable query errors
#[derive(Snafu, Debug)]
pub enum QueryError {
    /// The query result overflowed. This does not mean that the verification failed.
    /// This just means that the database was supposed to respond with a result that was too large.
    #[snafu(display("Overflow error"))]
    Overflow,
    /// The query result string could not be decoded. This does not mean that the verification failed.
    /// This just means that the database was supposed to respond with a string that was not valid UTF-8.
    #[snafu(display("String decode error"))]
    InvalidString,
    /// Decoding errors other than overflow and invalid string.
    #[snafu(display("Miscellaneous decoding error"))]
    MiscellaneousDecodingError,
    /// Miscellaneous evaluation error.
    #[snafu(display("Miscellaneous evaluation error"))]
    MiscellaneousEvaluationError,
    /// The proof failed to verify.
    #[snafu(transparent)]
    ProofError {
        /// The underlying source error
        source: ProofError,
    },
    /// The table data was invalid. This should never happen because this should get caught by the verifier before reaching this point.
    #[snafu(transparent)]
    InvalidTable {
        /// The underlying source error
        source: OwnedTableError,
    },
    /// The number of columns in the table was invalid.
    #[snafu(display("Invalid number of columns"))]
    InvalidColumnCount,
}

impl From<TableCoercionError> for QueryError {
    fn from(error: TableCoercionError) -> Self {
        match error {
            TableCoercionError::ColumnCoercionError {
                source: ColumnCoercionError::Overflow,
            } => QueryError::Overflow,
            TableCoercionError::ColumnCoercionError {
                source: ColumnCoercionError::InvalidTypeCoercion,
            } => ProofError::InvalidTypeCoercion.into(),
            TableCoercionError::NameMismatch => ProofError::FieldNamesMismatch.into(),
            TableCoercionError::ColumnCountMismatch => ProofError::FieldCountMismatch.into(),
        }
    }
}

/// The verified results of a query along with metadata produced by verification
pub struct QueryData<S: Scalar> {
    /// We use Apache Arrow's [`RecordBatch`] to represent a table
    /// result so as to allow for easy interoperability with
    /// Apache Arrow Flight.
    ///
    /// See `<https://voltrondata.com/blog/apache-arrow-flight-primer/>`
    pub table: OwnedTable<S>,
    /// Additionally, there is a 32-byte verification hash that is included with this table.
    /// This hash provides evidence that the verification has been run.
    pub verification_hash: [u8; 32],
}

/// The result of a query -- either an error or a table.
pub type QueryResult<S> = Result<QueryData<S>, QueryError>;

#[cfg(test)]
mod tests {
    use super::QueryError;
    use crate::base::{
        database::{ColumnCoercionError, TableCoercionError},
        proof::ProofError,
    };

    #[test]
    fn overflow_display() {
        let e = QueryError::Overflow;
        assert_eq!(alloc::format!("{e}"), "Overflow error");
    }

    #[test]
    fn invalid_string_display() {
        let e = QueryError::InvalidString;
        assert_eq!(alloc::format!("{e}"), "String decode error");
    }

    #[test]
    fn miscellaneous_decoding_error_display() {
        let e = QueryError::MiscellaneousDecodingError;
        assert_eq!(alloc::format!("{e}"), "Miscellaneous decoding error");
    }

    #[test]
    fn miscellaneous_evaluation_error_display() {
        let e = QueryError::MiscellaneousEvaluationError;
        assert_eq!(alloc::format!("{e}"), "Miscellaneous evaluation error");
    }

    #[test]
    fn invalid_column_count_display() {
        let e = QueryError::InvalidColumnCount;
        assert_eq!(alloc::format!("{e}"), "Invalid number of columns");
    }

    #[test]
    fn debug_overflow_contains_variant_name() {
        let e = QueryError::Overflow;
        assert!(alloc::format!("{e:?}").contains("Overflow"));
    }

    #[test]
    fn from_table_coercion_overflow_produces_query_overflow() {
        let tc = TableCoercionError::ColumnCoercionError {
            source: ColumnCoercionError::Overflow,
        };
        let qe: QueryError = tc.into();
        assert!(matches!(qe, QueryError::Overflow));
    }

    #[test]
    fn from_table_coercion_invalid_type_produces_proof_error() {
        let tc = TableCoercionError::ColumnCoercionError {
            source: ColumnCoercionError::InvalidTypeCoercion,
        };
        let qe: QueryError = tc.into();
        assert!(matches!(qe, QueryError::ProofError { .. }));
    }

    #[test]
    fn from_table_coercion_name_mismatch_produces_proof_error() {
        let tc = TableCoercionError::NameMismatch;
        let qe: QueryError = tc.into();
        assert!(matches!(qe, QueryError::ProofError { .. }));
    }

    #[test]
    fn from_table_coercion_count_mismatch_produces_proof_error() {
        let tc = TableCoercionError::ColumnCountMismatch;
        let qe: QueryError = tc.into();
        assert!(matches!(qe, QueryError::ProofError { .. }));
    }

    #[test]
    fn from_proof_error_transparent() {
        let pe = ProofError::InvalidTypeCoercion;
        let qe: QueryError = pe.into();
        assert!(matches!(qe, QueryError::ProofError { .. }));
    }
}
