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
    use arrow::datatypes::DataType;

    #[test]
    fn record_batch_to_columns_error_displays_arrow_conversion_source() {
        let error = RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
            source: ArrowArrayToColumnConversionError::UnsupportedType {
                datatype: DataType::Null,
            },
        };

        assert_eq!(
            error.to_string(),
            "unsupported type: attempted conversion from ArrayRef of type Null to OwnedColumn"
        );
    }

    #[test]
    fn append_record_batch_error_displays_arrow_batch_source() {
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

    #[test]
    fn append_record_batch_error_displays_commitment_mismatch_source() {
        let error = AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
            source: ColumnCommitmentsMismatch::NumColumns,
        };

        assert_eq!(
            error.to_string(),
            "commitments with different column counts cannot operate with each other"
        );
    }
}
