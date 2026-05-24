use crate::base::arrow::{RecordBatchToColumnsError, AppendRecordBatchTableCommitmentError};

#[test]
fn record_batch_to_columns_error_debug() {
    // This error wraps ArrowArrayToColumnConversionError
    // We test the Debug formatting
    use crate::base::arrow::ArrowArrayToColumnConversionError;
    let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
        source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
    };
    assert!(format!("{:?}").contains("RecordBatchToColumnsError"));
}

#[test]
fn append_record_batch_error_debug() {
    use crate::base::commitment::ColumnCommitmentsMismatch;
    let err = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
        source: ColumnCommitmentsMismatch {
            left_columns: 3,
            right_columns: 5,
        },
    };
    assert!(format!("{:?}").contains("AppendRecordBatchTableCommitmentError"));
}
