use super::{
    arrow_array_to_column_conversion::ArrayRefExt,
    record_batch_errors::{AppendRecordBatchTableCommitmentError, RecordBatchToColumnsError},
};
use crate::base::{
    commitment::{
        AppendColumnCommitmentsError, AppendTableCommitmentError, Commitment, TableCommitment,
        TableCommitmentFromColumnsError,
    },
    database::Column,
    scalar::Scalar,
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use sqlparser::ast::Ident;

/// This function will return an error if:
/// - The field name cannot be parsed into an [`Identifier`].
/// - The conversion of an Arrow array to a [`Column`] fails.
pub fn batch_to_columns<'a, S: Scalar + 'a>(
    batch: &'a RecordBatch,
    alloc: &'a Bump,
) -> Result<Vec<(Ident, Column<'a, S>)>, RecordBatchToColumnsError> {
    batch
        .schema()
        .fields()
        .into_iter()
        .zip(batch.columns())
        .map(|(field, array)| {
            let identifier: Ident = field.name().as_str().into();
            let column: Column<S> = array.to_column(alloc, &(0..array.len()), None)?;
            Ok((identifier, column))
        })
        .collect()
}

impl<C: Commitment> TableCommitment<C> {
    /// Append an arrow [`RecordBatch`] to the existing [`TableCommitment`].
    ///
    /// The row offset is assumed to be the end of the [`TableCommitment`]'s current range.
    ///
    /// Will error on a variety of mismatches, or if the provided columns have mixed length.
    #[expect(clippy::missing_panics_doc)]
    pub fn try_append_record_batch(
        &mut self,
        batch: &RecordBatch,
        setup: &C::PublicSetup<'_>,
    ) -> Result<(), AppendRecordBatchTableCommitmentError> {
        match self.try_append_rows(
            batch_to_columns::<C::Scalar>(batch, &Bump::new())?
                .iter()
                .map(|(a, b)| (a, b)),
            setup,
        ) {
            Ok(()) => Ok(()),
            Err(AppendTableCommitmentError::MixedLengthColumns { .. }) => {
                panic!("RecordBatches cannot have columns of mixed length")
            }
            Err(AppendTableCommitmentError::AppendColumnCommitments {
                source: AppendColumnCommitmentsError::DuplicateIdents { .. },
            }) => {
                panic!("RecordBatches cannot have duplicate identifiers")
            }
            Err(AppendTableCommitmentError::AppendColumnCommitments {
                source: AppendColumnCommitmentsError::Mismatch { source: e },
            }) => Err(e)?,
        }
    }
    /// Returns a [`TableCommitment`] to the provided arrow [`RecordBatch`].
    pub fn try_from_record_batch(
        batch: &RecordBatch,
        setup: &C::PublicSetup<'_>,
    ) -> Result<TableCommitment<C>, RecordBatchToColumnsError> {
        Self::try_from_record_batch_with_offset(batch, 0, setup)
    }

    /// Returns a [`TableCommitment`] to the provided arrow [`RecordBatch`] with the given row offset.
    #[expect(clippy::missing_panics_doc)]
    pub fn try_from_record_batch_with_offset(
        batch: &RecordBatch,
        offset: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Result<TableCommitment<C>, RecordBatchToColumnsError> {
        match Self::try_from_columns_with_offset(
            batch_to_columns::<C::Scalar>(batch, &Bump::new())?
                .iter()
                .map(|(a, b)| (a, b)),
            offset,
            setup,
        ) {
            Ok(commitment) => Ok(commitment),
            Err(TableCommitmentFromColumnsError::MixedLengthColumns { .. }) => {
                panic!("RecordBatches cannot have columns of mixed length")
            }
            Err(TableCommitmentFromColumnsError::DuplicateIdents { .. }) => {
                panic!("RecordBatches cannot have duplicate identifiers")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        arrow::arrow_array_to_column_conversion::ArrowArrayToColumnConversionError,
        commitment::{naive_commitment::NaiveCommitment, ColumnCommitmentsMismatch},
        scalar::test_scalar::TestScalar,
    };
    use arrow::{
        array::{ArrayRef, Float64Array, Int64Array, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use std::sync::Arc;

    fn batch_with_column(name: &str, array: ArrayRef) -> RecordBatch {
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                name,
                array.data_type().clone(),
                true,
            )])),
            vec![array],
        )
        .unwrap()
    }

    #[test]
    fn we_reject_record_batches_with_nulls_or_unsupported_types() {
        let invalid_columns: [(ArrayRef, ArrowArrayToColumnConversionError); 2] = [
            (
                Arc::new(Int64Array::from(vec![Some(1), None])),
                ArrowArrayToColumnConversionError::ArrayContainsNulls,
            ),
            (
                Arc::new(Float64Array::from(vec![1.0, 2.0])),
                ArrowArrayToColumnConversionError::UnsupportedType {
                    datatype: DataType::Float64,
                },
            ),
        ];
        for (array, expected) in invalid_columns {
            // Put a valid column first so conversion must propagate a later failure.
            let batch = RecordBatch::try_new(
                Arc::new(Schema::new(vec![
                    Field::new("a", DataType::Int64, false),
                    Field::new("b", array.data_type().clone(), true),
                ])),
                vec![Arc::new(Int64Array::from(vec![1, 2])), array],
            )
            .unwrap();
            let alloc = Bump::new();
            for result in [
                batch_to_columns::<TestScalar>(&batch, &alloc).map(|_| ()),
                TableCommitment::<NaiveCommitment>::try_from_record_batch(&batch, &()).map(|_| ()),
                TableCommitment::<NaiveCommitment>::try_from_record_batch_with_offset(
                    &batch,
                    7,
                    &(),
                )
                .map(|_| ()),
            ] {
                let Err(RecordBatchToColumnsError::ArrowArrayToColumnConversionError { source }) =
                    result
                else {
                    panic!("expected the array conversion error");
                };
                assert_eq!(source, expected);
            }
        }
    }

    #[test]
    fn failed_record_batch_conversion_does_not_change_the_commitment() {
        let batch = batch_with_column("a", Arc::new(Int64Array::from(vec![1, 2])));
        let mut commitment =
            TableCommitment::<NaiveCommitment>::try_from_record_batch_with_offset(&batch, 7, &())
                .unwrap();
        let before = commitment.clone();
        let invalid = batch_with_column("a", Arc::new(Int64Array::from(vec![Some(3), None])));
        assert!(matches!(
            commitment.try_append_record_batch(&invalid, &()),
            Err(
                AppendRecordBatchTableCommitmentError::ArrowBatchToColumnError {
                    source: RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
                        source: ArrowArrayToColumnConversionError::ArrayContainsNulls,
                    },
                }
            )
        ));
        assert_eq!(commitment, before);

        // A rejected append must not advance the offset of the next valid append.
        let valid = batch_with_column("a", Arc::new(Int64Array::from(vec![3, 4])));
        commitment.try_append_record_batch(&valid, &()).unwrap();
        let columns = [(&"a".into(), &Column::<TestScalar>::BigInt(&[1, 2, 3, 4]))];
        let expected =
            TableCommitment::<NaiveCommitment>::try_from_columns_with_offset(columns, 7, &())
                .unwrap();
        assert_eq!(commitment, expected);
    }

    #[test]
    fn mismatched_record_batch_schema_does_not_change_the_commitment() {
        let batch = batch_with_column("a", Arc::new(Int64Array::from(vec![1, 2])));
        let mut commitment =
            TableCommitment::<NaiveCommitment>::try_from_record_batch(&batch, &()).unwrap();
        let before = commitment.clone();

        let renamed = batch_with_column("b", Arc::new(Int64Array::from(vec![3, 4])));
        assert!(matches!(
            commitment.try_append_record_batch(&renamed, &()),
            Err(
                AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
                    source: ColumnCommitmentsMismatch::Ident { .. },
                }
            )
        ));
        assert_eq!(commitment, before);

        let retyped = batch_with_column("a", Arc::new(StringArray::from(vec!["3", "4"])));
        assert!(matches!(
            commitment.try_append_record_batch(&retyped, &()),
            Err(
                AppendRecordBatchTableCommitmentError::ColumnCommitmentsMismatch {
                    source: ColumnCommitmentsMismatch::ColumnCommitmentMetadata { .. },
                }
            )
        ));
        assert_eq!(commitment, before);
    }

    #[test]
    fn we_can_create_and_append_table_commitments_with_record_batches() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Utf8, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["1", "2", "3"])),
            ],
        )
        .unwrap();

        let b_scals = ["1".into(), "2".into(), "3".into()];

        let columns = [
            (&"a".into(), &Column::<TestScalar>::BigInt(&[1, 2, 3])),
            (
                &"b".into(),
                &Column::<TestScalar>::VarChar((&["1", "2", "3"], &b_scals)),
            ),
        ];

        let mut expected_commitment =
            TableCommitment::<NaiveCommitment>::try_from_columns_with_offset(columns, 0, &())
                .unwrap();

        let mut commitment =
            TableCommitment::<NaiveCommitment>::try_from_record_batch(&batch, &()).unwrap();

        assert_eq!(commitment, expected_commitment);

        let schema2 = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Utf8, false),
        ]));

        let batch2 = RecordBatch::try_new(
            schema2,
            vec![
                Arc::new(Int64Array::from(vec![4, 5, 6])),
                Arc::new(StringArray::from(vec!["4", "5", "6"])),
            ],
        )
        .unwrap();

        let b_scals2 = ["4".into(), "5".into(), "6".into()];

        let columns2 = [
            (&"a".into(), &Column::<TestScalar>::BigInt(&[4, 5, 6])),
            (
                &"b".into(),
                &Column::<TestScalar>::VarChar((&["4", "5", "6"], &b_scals2)),
            ),
        ];

        expected_commitment.try_append_rows(columns2, &()).unwrap();
        commitment.try_append_record_batch(&batch2, &()).unwrap();

        assert_eq!(commitment, expected_commitment);
    }
}
