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
    use super::*;
    use alloc::format;

    #[test]
    fn we_can_convert_overflow_coercion_error_to_query_error() {
        let coercion_error = TableCoercionError::ColumnCoercionError {
            source: ColumnCoercionError::Overflow,
        };
        let query_error: QueryError = coercion_error.into();
        assert!(matches!(query_error, QueryError::Overflow));
    }

    #[test]
    fn we_can_convert_invalid_type_coercion_error_to_query_error() {
        let coercion_error = TableCoercionError::ColumnCoercionError {
            source: ColumnCoercionError::InvalidTypeCoercion,
        };
        let query_error: QueryError = coercion_error.into();
        assert!(matches!(query_error, QueryError::ProofError { .. }));
    }

    #[test]
    fn we_can_convert_name_mismatch_to_query_error() {
        let coercion_error = TableCoercionError::NameMismatch;
        let query_error: QueryError = coercion_error.into();
        assert!(matches!(query_error, QueryError::ProofError { .. }));
    }

    #[test]
    fn we_can_convert_column_count_mismatch_to_query_error() {
        let coercion_error = TableCoercionError::ColumnCountMismatch;
        let query_error: QueryError = coercion_error.into();
        assert!(matches!(query_error, QueryError::ProofError { .. }));
    }

    #[test]
    fn query_error_displays_correctly() {
        assert_eq!(format!("{}", QueryError::Overflow), "Overflow error");
        assert_eq!(format!("{}", QueryError::InvalidString), "String decode error");
        assert_eq!(
            format!("{}", QueryError::MiscellaneousDecodingError),
            "Miscellaneous decoding error"
        );
        assert_eq!(
            format!("{}", QueryError::MiscellaneousEvaluationError),
            "Miscellaneous evaluation error"
        );
        assert_eq!(
            format!("{}", QueryError::InvalidColumnCount),
            "Invalid number of columns"
        );
    }

    #[test]
    fn we_can_convert_proof_error_to_query_error() {
        let proof_error = ProofError::InvalidTypeCoercion;
        let query_error: QueryError = proof_error.into();
        let msg = format!("{query_error}");
        assert!(msg.contains("type mismatch"));
    }
}
