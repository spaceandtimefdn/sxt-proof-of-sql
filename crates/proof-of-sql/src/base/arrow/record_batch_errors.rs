use super::arrow_array_to_column_conversion::ArrowArrayToColumnConversionError;
use crate::base::commitment::ColumnCommitmentsMismatch;
use snafu::Snafu;

/// Errors that can occur when trying to create or extend a [`TableCommitment`] from a record batch.
#[derive(Debug, Snafu)]
pub enum RecordBatchToColumnsError {
    /// Error converting from arrow array
    #[snafu(transparent)]
    ArrowArrayToColumnConversionError {
        /// The underlying source error
        source: ArrowArrayToColumnConversionError,
    },
}

/// Errors that can occur when attempting to append a record batch to a [`TableCommitment`].
#[derive(Debug, Snafu)]
pub enum AppendRecordBatchTableCommitmentError {
    /// During commitment operation, metadata indicates that operand tables cannot be the same.
    #[snafu(transparent)]
    ColumnCommitmentsMismatch {
        /// The underlying source error
        source: ColumnCommitmentsMismatch,
    },
    /// Error converting from arrow array
    #[snafu(transparent)]
    ArrowBatchToColumnError {
        /// The underlying source error
        source: RecordBatchToColumnsError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn record_batch_to_columns_error_forwards_arrow_conversion_display() {
        let error = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
        };

        assert_eq!(error.to_string(), "arrow array must not contain nulls");
    }

    #[test]
    fn append_record_batch_error_forwards_column_commitment_mismatch_display() {
        let error = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
            source: ColumnCommitmentsMismatch::NumColumns,
        };

        assert_eq!(
            error.to_string(),
            "commitments with different column counts cannot operate with each other"
        );
    }

    #[test]
    fn append_record_batch_error_forwards_batch_to_column_display() {
        let error = AppendRecordBatchTableCommitmentError::ArrowBatchToColumnError {
            source: RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
                source: ArrowArrayToColumnConversionError::IndexOutOfBounds { len: 2, index: 3 },
            },
        };

        assert_eq!(
            error.to_string(),
            "index out of bounds: the len is 2 but the index is 3"
        );
    }
}
