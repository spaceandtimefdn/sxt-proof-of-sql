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
    use super::RecordBatchToColumnsError;
    use crate::base::arrow::arrow_array_to_column_conversion::ArrowArrayToColumnConversionError;
    use arrow::datatypes::DataType;

    #[test]
    fn record_batch_to_columns_error_debug() {
        let inner = ArrowArrayToColumnConversionError::ArrayContainsNulls;
        let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError { source: inner };
        let msg = alloc::format!("{err:?}");
        assert!(msg.contains("ArrowArrayToColumnConversionError") || msg.contains("ArrayContainsNulls"));
    }

    #[test]
    fn unsupported_type_error_propagates_through_record_batch_error() {
        let inner = ArrowArrayToColumnConversionError::UnsupportedType { datatype: DataType::Float64 };
        let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError { source: inner };
        let msg = alloc::format!("{err}");
        assert!(msg.contains("Float64") || msg.contains("unsupported"));
    }

    #[test]
    fn array_contains_nulls_error_display() {
        let inner = ArrowArrayToColumnConversionError::ArrayContainsNulls;
        let err = RecordBatchToColumnsError::ArrowArrayToColumnConversionError { source: inner };
        let msg = alloc::format!("{err}");
        assert!(msg.contains("null") || msg.contains("null"));
    }
}
