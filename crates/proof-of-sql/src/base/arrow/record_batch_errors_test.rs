#[cfg(test)]
mod record_batch_errors_test {
    #[test]
    fn test_error_types_exist() {
        use crate::base::arrow::record_batch_errors::{
            RecordBatchToColumnsError, AppendRecordBatchTableCommitmentError,
        };
        // Just verify the types exist and are constructible (by pattern)
        fn _check<E: std::fmt::Debug>() {}
        _check::<RecordBatchToColumnsError>();
        _check::<AppendRecordBatchTableCommitmentError>();
    }
}
