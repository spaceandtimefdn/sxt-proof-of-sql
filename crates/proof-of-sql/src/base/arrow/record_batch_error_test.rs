//! Tests for RecordBatch errors.

#[cfg(test)]
mod record_batch_error_test {
    use crate::base::arrow::record_batch_errors::{
        AppendRecordBatchTableCommitmentError, RecordBatchToColumnsError,
    };
    use crate::base::arrow::ArrowArrayToColumnConversionError;
    use crate::base::commitment::ColumnCommitmentsMismatch;
    use arrow::datatypes::DataType;

    #[test]
    fn test_record_batch_to_columns_error_arrow() {
        let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_record_batch_to_columns_error_unsupported_type() {
        let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::UnsupportedType {
                datatype: DataType::List,
            },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_record_batch_to_columns_error_debug() {
        let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("RecordBatchToColumnsError"));
    }

    #[test]
    fn test_append_record_batch_error_mismatch() {
        let err = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
            source: ColumnCommitmentsMismatch::NumColumns,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_append_record_batch_error_arrow() {
        let err = AppendRecordBatchTableCommitmentError::ArrowBatchToColumnError {
            source: RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
                source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
            },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_append_record_batch_error_debug() {
        let err = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
            source: ColumnCommitmentsMismatch::Ident {
                id_a: "col1".to_string(),
                id_b: "col2".to_string(),
            },
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("AppendRecordBatchTableCommitmentError"));
    }

    #[test]
    fn test_error_partial_eq() {
        let err1 = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
        };
        let err2 = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
        };
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_append_error_partial_eq() {
        let err1 = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
            source: ColumnCommitmentsMismatch::NumColumns,
        };
        let err2 = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
            source: ColumnCommitmentsMismatch::NumColumns,
        };
        assert_eq!(err1, err2);
    }
}