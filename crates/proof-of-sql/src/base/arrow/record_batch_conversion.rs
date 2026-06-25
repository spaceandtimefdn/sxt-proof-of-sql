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
        commitment::naive_commitment::NaiveCommitment, scalar::test_scalar::TestScalar,
    };
    use arrow::{
        array::{BooleanArray, Int64Array, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use std::sync::Arc;

    fn int_and_string_batch(ints: Vec<i64>, strings: Vec<&str>) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Utf8, false),
        ]));

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(ints)),
                Arc::new(StringArray::from(strings)),
            ],
        )
        .unwrap()
    }

    #[test]
    fn we_can_convert_record_batches_to_named_columns() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("is_active", DataType::Boolean, false),
            Field::new("amount", DataType::Int64, false),
            Field::new("label", DataType::Utf8, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(BooleanArray::from(vec![true, false, true])),
                Arc::new(Int64Array::from(vec![5, -8, 13])),
                Arc::new(StringArray::from(vec!["alpha", "beta", "gamma"])),
            ],
        )
        .unwrap();
        let alloc = Bump::new();
        let columns = batch_to_columns::<TestScalar>(&batch, &alloc).unwrap();

        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].0.value, "is_active");
        assert_eq!(columns[1].0.value, "amount");
        assert_eq!(columns[2].0.value, "label");
        assert_eq!(columns[0].1, Column::Boolean(&[true, false, true]));
        assert_eq!(columns[1].1, Column::BigInt(&[5, -8, 13]));

        let expected_labels = ["alpha", "beta", "gamma"];
        let expected_scalars: Vec<TestScalar> = expected_labels
            .iter()
            .map(|value| (*value).into())
            .collect();
        assert_eq!(
            columns[2].1,
            Column::VarChar((&expected_labels, expected_scalars.as_slice()))
        );
    }

    #[test]
    fn we_cannot_convert_record_batches_with_nulls_to_columns() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "amount",
            DataType::Int64,
            true,
        )]));
        let batch = RecordBatch::try_new(
            schema,
            vec![Arc::new(Int64Array::from(vec![Some(5), None, Some(13)]))],
        )
        .unwrap();
        let alloc = Bump::new();
        let err = batch_to_columns::<TestScalar>(&batch, &alloc).unwrap_err();

        assert!(matches!(
            err,
            RecordBatchToColumnsError::ArrowArrayToColumnConversionError {
                source: ArrowArrayToColumnConversionError::ArrayContainsNulls
            }
        ));
    }

    #[test]
    fn record_batch_table_commitments_respect_offsets_and_append_rows() {
        let batch = int_and_string_batch(vec![1, 2], vec!["1", "2"]);
        let mut commitment =
            TableCommitment::<NaiveCommitment>::try_from_record_batch_with_offset(&batch, 2, &())
                .unwrap();

        assert_eq!(commitment.range(), &(2..4));
        assert_eq!(commitment.num_rows(), 2);

        let next_batch = int_and_string_batch(vec![3, 4], vec!["3", "4"]);
        commitment
            .try_append_record_batch(&next_batch, &())
            .unwrap();

        let a_ident = "a".into();
        let b_ident = "b".into();
        let a_values = [1, 2, 3, 4];
        let b_values = ["1", "2", "3", "4"];
        let b_scalars: Vec<TestScalar> = b_values.iter().map(|value| (*value).into()).collect();
        let a_column = Column::<TestScalar>::BigInt(&a_values);
        let b_column = Column::VarChar((&b_values, b_scalars.as_slice()));
        let expected_columns = [(&a_ident, &a_column), (&b_ident, &b_column)];
        let expected_commitment = TableCommitment::<NaiveCommitment>::try_from_columns_with_offset(
            expected_columns,
            2,
            &(),
        )
        .unwrap();

        assert_eq!(commitment, expected_commitment);
    }

    #[test]
    fn we_can_create_and_append_table_commitments_with_record_batches() {
        let batch = int_and_string_batch(vec![1, 2, 3], vec!["1", "2", "3"]);

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

        let batch2 = int_and_string_batch(vec![4, 5, 6], vec!["4", "5", "6"]);

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
