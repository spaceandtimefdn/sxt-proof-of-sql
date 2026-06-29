//! Tests for QueryError.

#[cfg(test)]
mod query_error_tests {
    use crate::sql::proof::{QueryData, QueryError, QueryResult};
    use crate::base::proof::ProofError;
    use crate::base::database::OwnedTableError;
    use crate::base::scalar::Scalar;
    use alloc::string::ToString;

    #[test]
    fn test_error_overflow() {
        let err = QueryError::Overflow;
        assert_eq!(err.to_string(), "Overflow error");
    }

    #[test]
    fn test_error_invalid_string() {
        let err = QueryError::InvalidString;
        assert_eq!(err.to_string(), "String decode error");
    }

    #[test]
    fn test_error_miscellaneous_decoding_error() {
        let err = QueryError::MiscellaneousDecodingError;
        assert_eq!(err.to_string(), "Miscellaneous decoding error");
    }

    #[test]
    fn test_error_miscellaneous_evaluation_error() {
        let err = QueryError::MiscellaneousEvaluationError;
        assert_eq!(err.to_string(), "Miscellaneous evaluation error");
    }

    #[test]
    fn test_error_proof_error() {
        let err = QueryError::ProofError {
            source: ProofError::InternalError,
        };
        let s = err.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_error_invalid_table() {
        #[derive(Debug)]
        struct MockTableError;
        impl core::fmt::Display for MockTableError {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "mock error")
            }
        }
        let _err = QueryError::InvalidTable {
            source: OwnedTableError::ColumnNotFound("test".to_string()),
        };
    }

    #[test]
    fn test_error_invalid_column_count() {
        let err = QueryError::InvalidColumnCount;
        assert_eq!(err.to_string(), "Invalid number of columns");
    }

    #[test]
    fn test_error_partial_eq() {
        let err1 = QueryError::Overflow;
        let err2 = QueryError::Overflow;
        let err3 = QueryError::InvalidString;
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_debug() {
        let err = QueryError::Overflow;
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_result_type() {
        let result: QueryResult<blitzar::proof::InnerProductProof> = Err(QueryError::Overflow);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), QueryError::Overflow);
    }

    #[test]
    fn test_error_description() {
        let errors = vec![
            QueryError::Overflow,
            QueryError::InvalidString,
            QueryError::MiscellaneousDecodingError,
            QueryError::MiscellaneousEvaluationError,
            QueryError::InvalidColumnCount,
        ];
        for err in errors {
            let s = err.to_string();
            assert!(!s.is_empty(), "Error should have a description");
        }
    }
}