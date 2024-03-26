use super::{
    committable_column::CommittableColumn, AppendColumnCommitmentsError, ColumnCommitments,
    ColumnCommitmentsMismatch, DuplicateIdentifiers, VecCommitmentExt,
};
use crate::base::{database::OwnedTable, scalar::Scalar};
use proofs_sql::Identifier;
use std::ops::Range;
use thiserror::Error;

/// Cannot create a [`TableCommitment`] with a negative range.
#[derive(Debug, Error)]
#[error("cannot create a TableCommitment with a negative range")]
pub struct NegativeRange;

/// Cannot create a [`TableCommitment`] from columns of mixed length.
#[derive(Debug, Error)]
#[error("cannot create a TableCommitment from columns of mixed length")]
pub struct MixedLengthColumns;

/// Errors that can occur when trying to create or extend a [`TableCommitment`] from columns.
#[derive(Debug, Error)]
pub enum TableCommitmentFromColumnsError {
    /// Cannot construct [`TableCommitment`] from columns of mixed length.
    #[error(transparent)]
    MixedLengthColumns(#[from] MixedLengthColumns),
    /// Cannot construct [`TableCommitment`] from columns with duplicate identifiers.
    #[error(transparent)]
    DuplicateIdentifiers(#[from] DuplicateIdentifiers),
}

/// Errors that can occur when attempting to append rows to a [`TableCommitment`].
#[derive(Debug, Error)]
pub enum AppendTableCommitmentError {
    /// Cannot append columns of mixed length to existing [`TableCommitment`].
    #[error(transparent)]
    MixedLengthColumns(#[from] MixedLengthColumns),
    /// Encountered error when appending internal [`ColumnCommitments`].
    #[error(transparent)]
    AppendColumnCommitments(#[from] AppendColumnCommitmentsError),
}

/// Errors that can occur when performing arithmetic on [`TableCommitment`]s.
#[derive(Debug, Error)]
pub enum TableCommitmentArithmeticError {
    /// Cannot perform arithmetic on columns with mismatched metadata.
    #[error(transparent)]
    ColumnMismatch(#[from] ColumnCommitmentsMismatch),
    /// Cannot perform TableCommitment arithmetic that would result in a negative range.
    #[error(transparent)]
    NegativeRange(#[from] NegativeRange),
    /// Cannot perform arithmetic for noncontiguous table commitments.
    #[error("cannot perform table commitment arithmetic for noncontiguous table commitments")]
    NonContiguous,
}

/// Commitment for an entire table, with column and table metadata.
///
/// Unlike [`ColumnCommitments`], all columns in this commitment must have the same length.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct TableCommitment<C>
where
    Vec<C>: VecCommitmentExt,
{
    column_commitments: ColumnCommitments<C>,
    range: Range<usize>,
}

/// Private convenience alias.
type Setup<C> = <Vec<C> as VecCommitmentExt>::CommitmentPublicSetup;

impl<C> TableCommitment<C>
where
    Vec<C>: VecCommitmentExt,
{
    /// Construct a new [`TableCommitment`].
    ///
    /// Will error if the range is "negative", i.e. if its end < start.
    pub fn try_new(
        column_commitments: ColumnCommitments<C>,
        range: Range<usize>,
    ) -> Result<Self, NegativeRange> {
        if range.start <= range.end {
            Ok(TableCommitment {
                column_commitments,
                range,
            })
        } else {
            Err(NegativeRange)
        }
    }

    /// Returns a reference to this type's internal [`ColumnCommitments`].
    pub fn column_commitments(&self) -> &ColumnCommitments<C> {
        &self.column_commitments
    }

    /// Returns a reference to the range of rows this type commits to.
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }

    /// Returns the number of columns in the committed table.
    pub fn num_columns(&self) -> usize {
        self.column_commitments.len()
    }

    /// Returns the number of rows that have been committed to.
    pub fn num_rows(&self) -> usize {
        self.range.len()
    }

    /// Returns a [`TableCommitment`] to the provided columns with the given row offset.
    ///
    /// Provided columns must have the same length and no duplicate identifiers.
    pub fn try_from_columns_with_offset<'a, COL>(
        columns: impl IntoIterator<Item = (&'a Identifier, COL)>,
        offset: usize,
        setup: &Setup<C>,
    ) -> Result<TableCommitment<C>, TableCommitmentFromColumnsError>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        let (identifiers, committable_columns): (Vec<&Identifier>, Vec<CommittableColumn>) =
            columns
                .into_iter()
                .map(|(identifier, column)| (identifier, column.into()))
                .unzip();

        let num_rows = num_rows_of_columns(&committable_columns)?;

        let column_commitments = ColumnCommitments::try_from_columns_with_offset(
            identifiers.into_iter().zip(committable_columns.into_iter()),
            offset,
            setup,
        )?;

        Ok(TableCommitment {
            column_commitments,
            range: offset..offset + num_rows,
        })
    }

    /// Returns a [`TableCommitment`] to the provided table with the given row offset.
    pub fn from_owned_table_with_offset<S>(
        owned_table: &OwnedTable<S>,
        offset: usize,
        setup: &Setup<C>,
    ) -> TableCommitment<C>
    where
        S: Scalar,
    {
        Self::try_from_columns_with_offset(owned_table.inner_table(), offset, setup)
            .expect("OwnedTables cannot have columns of mixed length or duplicate identifiers")
    }

    /// Append rows of data from the provided columns to the existing [`TableCommitment`].
    ///
    /// The row offset is assumed to be the end of the [`TableCommitment`]'s current range.
    ///
    /// Will error on a variety of mismatches, or if the provided columns have mixed length.
    pub fn try_append_rows<'a, COL>(
        &mut self,
        columns: impl IntoIterator<Item = (&'a Identifier, COL)>,
        setup: &Setup<C>,
    ) -> Result<(), AppendTableCommitmentError>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        let (identifiers, committable_columns): (Vec<&Identifier>, Vec<CommittableColumn>) =
            columns
                .into_iter()
                .map(|(identifier, column)| (identifier, column.into()))
                .unzip();

        let num_rows = num_rows_of_columns(&committable_columns)?;

        self.column_commitments.try_append_rows_with_offset(
            identifiers.into_iter().zip(committable_columns.into_iter()),
            self.range.end,
            setup,
        )?;
        self.range.end += num_rows;

        Ok(())
    }

    /// Append data of the provided table to the exiting [`TableCommitment`].
    ///
    /// Will error on a variety of mismatches.
    /// See [`ColumnCommitmentsMismatch`] for an enumeration of these errors.
    pub fn append_owned_table<S>(
        &mut self,
        owned_table: &OwnedTable<S>,
        setup: &Setup<C>,
    ) -> Result<(), ColumnCommitmentsMismatch>
    where
        S: Scalar,
    {
        self.try_append_rows(owned_table.inner_table(), setup)
            .map_err(|e| match e {
                AppendTableCommitmentError::AppendColumnCommitments(e) => match e {
                    AppendColumnCommitmentsError::Mismatch(e) => e,
                    AppendColumnCommitmentsError::DuplicateIdentifiers(_) => {
                        panic!("OwnedTables cannot have duplicate identifiers");
                    }
                },
                AppendTableCommitmentError::MixedLengthColumns(_) => {
                    panic!("OwnedTables cannot have columns of mixed length");
                }
            })
    }

    /// Add new columns to this [`TableCommitment`].
    ///
    /// Columns must have the same length as the current commitment and no duplicate identifiers.
    pub fn try_extend_columns<'a, COL>(
        &mut self,
        columns: impl IntoIterator<Item = (&'a Identifier, COL)>,
        setup: &Setup<C>,
    ) -> Result<(), TableCommitmentFromColumnsError>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        let num_rows = self.range.len();

        let (identifiers, committable_columns): (Vec<&Identifier>, Vec<CommittableColumn>) =
            columns
                .into_iter()
                .map(|(identifier, column)| (identifier, column.into()))
                .unzip();

        let num_rows_of_new_columns = num_rows_of_columns(&committable_columns)?;
        if num_rows_of_new_columns != num_rows {
            Err(MixedLengthColumns)?;
        }

        self.column_commitments.try_extend_columns_with_offset(
            identifiers.into_iter().zip(committable_columns.into_iter()),
            self.range.start,
            setup,
        )?;

        Ok(())
    }

    /// Add two [`TableCommitment`]s together.
    ///
    /// `self` must end where `other` begins, or vice versa.
    /// Otherwise, [`TableCommitmentArithmeticError::NonContiguous`] is returned.
    ///
    /// This will also error on a variety of mismatches.
    /// See [`ColumnCommitmentsMismatch`] for an enumeration of these errors.
    pub fn try_add(self, other: Self) -> Result<Self, TableCommitmentArithmeticError>
    where
        Self: Sized,
    {
        let range = if self.range.end == other.range.start {
            self.range.start..other.range.end
        } else if other.range.end == self.range.start {
            other.range.start..self.range.end
        } else {
            return Err(TableCommitmentArithmeticError::NonContiguous);
        };

        let column_commitments = self.column_commitments.try_add(other.column_commitments)?;

        Ok(TableCommitment {
            column_commitments,
            range,
        })
    }

    /// Subtract two [`TableCommitment`]s.
    ///
    /// `self` and `other` must begin at the same row number or end at the same row number.
    /// Otherwise, [`TableCommitmentArithmeticError::NonContiguous`] is returned.
    ///
    /// Furthermore, `other`'s range must be smaller or equal to `self`'s.
    /// Otherwise, [`TableCommitmentArithmeticError::NegativeRange`] is returned.
    ///
    /// This will also error on a variety of mismatches.
    /// See [`ColumnCommitmentsMismatch`] for an enumeration of these errors.
    pub fn try_sub(self, other: Self) -> Result<Self, TableCommitmentArithmeticError>
    where
        Self: Sized,
    {
        if self.range.len() < other.range.len() {
            Err(NegativeRange)?;
        }

        let range = if self.range.start == other.range.start {
            other.range.end..self.range.end
        } else if self.range.end == other.range.end {
            self.range.start..other.range.start
        } else {
            return Err(TableCommitmentArithmeticError::NonContiguous);
        };

        let column_commitments = self.column_commitments.try_sub(other.column_commitments)?;

        Ok(TableCommitment {
            column_commitments,
            range,
        })
    }
}

/// Return the number of rows for the provided columns, erroring if they have mixed length.
fn num_rows_of_columns<'a>(
    committable_columns: impl IntoIterator<Item = &'a CommittableColumn<'a>>,
) -> Result<usize, MixedLengthColumns> {
    let mut committable_columns_iter = committable_columns.into_iter().peekable();
    let num_rows = committable_columns_iter
        .peek()
        .map_or(0, |committable_column| committable_column.len());

    for committable_column in committable_columns_iter {
        if committable_column.len() != num_rows {
            return Err(MixedLengthColumns);
        }
    }

    Ok(num_rows)
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::{database::OwnedColumn, scalar::Curve25519Scalar},
        owned_table,
    };
    use curve25519_dalek::ristretto::CompressedRistretto;
    use indexmap::IndexMap;

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn we_cannot_construct_table_commitment_with_negative_range() {
        let try_new_result =
            TableCommitment::<CompressedRistretto>::try_new(ColumnCommitments::default(), 1..0);

        assert!(matches!(try_new_result, Err(NegativeRange)));
    }

    #[test]
    fn we_can_construct_table_commitment_from_columns_and_identifiers() {
        // no-columns case
        let mut empty_columns_iter: IndexMap<Identifier, OwnedColumn<Curve25519Scalar>> =
            IndexMap::new();
        let empty_table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                &empty_columns_iter,
                0,
                &(),
            )
            .unwrap();
        assert_eq!(
            empty_table_commitment.column_commitments(),
            &ColumnCommitments::try_from_columns_with_offset(&empty_columns_iter, 0, &()).unwrap()
        );
        assert_eq!(empty_table_commitment.range(), &(0..0));
        assert_eq!(empty_table_commitment.num_columns(), 0);
        assert_eq!(empty_table_commitment.num_rows(), 0);

        // no-rows case
        empty_columns_iter.insert("column_a".parse().unwrap(), OwnedColumn::BigInt(vec![]));
        let empty_table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                &empty_columns_iter,
                1,
                &(),
            )
            .unwrap();
        assert_eq!(
            empty_table_commitment.column_commitments(),
            &ColumnCommitments::try_from_columns_with_offset(&empty_columns_iter, 1, &()).unwrap()
        );
        assert_eq!(empty_table_commitment.range(), &(1..1));
        assert_eq!(empty_table_commitment.num_columns(), 1);
        assert_eq!(empty_table_commitment.num_rows(), 0);

        // nonempty case
        let owned_table = owned_table!(
            "bigint_id" => [1i64, 5, -5, 0],
            // "int128_column" => [100i128, 200, 300, 400], TODO: enable this column once blitzar
            // supports it
            "varchar_id" => ["Lorem", "ipsum", "dolor", "sit"],
            "scalar_id" => [1000, 2000, -1000, 0].map(Curve25519Scalar::from),
        );
        let table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                owned_table.inner_table(),
                2,
                &(),
            )
            .unwrap();
        assert_eq!(
            table_commitment.column_commitments(),
            &ColumnCommitments::try_from_columns_with_offset(owned_table.inner_table(), 2, &())
                .unwrap()
        );
        assert_eq!(table_commitment.range(), &(2..6));
        assert_eq!(table_commitment.num_columns(), 3);
        assert_eq!(table_commitment.num_rows(), 4);

        // matches from_owned_table constructor
        let table_commitment_from_owned_table =
            TableCommitment::from_owned_table_with_offset(&owned_table, 2, &());
        assert_eq!(table_commitment_from_owned_table, table_commitment);
    }

    #[test]
    fn we_cannot_construct_table_commitment_from_duplicate_identifiers() {
        let duplicate_identifier_a = "duplicate_identifier_a".parse().unwrap();
        let duplicate_identifier_b = "duplicate_identifier_b".parse().unwrap();
        let unique_identifier = "unique_identifier".parse().unwrap();

        let empty_column = OwnedColumn::<Curve25519Scalar>::BigInt(vec![]);

        let from_columns_result =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                [
                    (&duplicate_identifier_a, &empty_column),
                    (&unique_identifier, &empty_column),
                    (&duplicate_identifier_a, &empty_column),
                ],
                0,
                &(),
            );
        assert!(matches!(
            from_columns_result,
            Err(TableCommitmentFromColumnsError::DuplicateIdentifiers(_))
        ));

        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                [
                    (&duplicate_identifier_a, &empty_column),
                    (&unique_identifier, &empty_column),
                ],
                0,
                &(),
            )
            .unwrap();
        let column_commitments = table_commitment.column_commitments().clone();

        let extend_columns_result =
            table_commitment.try_extend_columns([(&duplicate_identifier_a, &empty_column)], &());
        assert!(matches!(
            extend_columns_result,
            Err(TableCommitmentFromColumnsError::DuplicateIdentifiers(_))
        ));

        let extend_columns_result = table_commitment.try_extend_columns(
            [
                (&duplicate_identifier_b, &empty_column),
                (&duplicate_identifier_b, &empty_column),
            ],
            &(),
        );
        assert!(matches!(
            extend_columns_result,
            Err(TableCommitmentFromColumnsError::DuplicateIdentifiers(_))
        ));

        // make sure the commitment wasn't mutated
        assert_eq!(table_commitment.num_columns(), 2);
        assert_eq!(table_commitment.column_commitments(), &column_commitments);
    }

    #[test]
    fn we_cannot_construct_table_commitment_from_columns_of_mixed_length() {
        let column_id_a = "column_a".parse().unwrap();
        let column_id_b = "column_b".parse().unwrap();
        let column_id_c = "column_c".parse().unwrap();

        let one_row_column = OwnedColumn::<Curve25519Scalar>::BigInt(vec![1]);
        let two_row_column = OwnedColumn::<Curve25519Scalar>::BigInt(vec![1, 2]);

        let from_columns_result =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                [
                    (&column_id_a, &one_row_column),
                    (&column_id_b, &two_row_column),
                ],
                0,
                &(),
            );
        assert!(matches!(
            from_columns_result,
            Err(TableCommitmentFromColumnsError::MixedLengthColumns(_))
        ));

        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                [(&column_id_a, &one_row_column)],
                0,
                &(),
            )
            .unwrap();
        let column_commitments = table_commitment.column_commitments().clone();

        let extend_columns_result =
            table_commitment.try_extend_columns([(&column_id_b, &two_row_column)], &());
        assert!(matches!(
            extend_columns_result,
            Err(TableCommitmentFromColumnsError::MixedLengthColumns(_))
        ));

        let extend_columns_result = table_commitment.try_extend_columns(
            [
                (&column_id_b, &one_row_column),
                (&column_id_c, &two_row_column),
            ],
            &(),
        );
        assert!(matches!(
            extend_columns_result,
            Err(TableCommitmentFromColumnsError::MixedLengthColumns(_))
        ));

        // make sure the commitment wasn't mutated
        assert_eq!(table_commitment.num_columns(), 1);
        assert_eq!(table_commitment.column_commitments(), &column_commitments);
    }

    #[test]
    fn we_can_append_rows_to_table_commitment() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0].map(Curve25519Scalar::from);

        let initial_columns: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[..2].to_vec(),
            varchar_id => varchar_data[..2].to_vec(),
            scalar_id => scalar_data[..2].to_vec(),
        );

        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                initial_columns.inner_table(),
                0,
                &(),
            )
            .unwrap();
        let mut table_commitment_clone = table_commitment.clone();

        let append_columns: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[2..].to_vec(),
            varchar_id => varchar_data[2..].to_vec(),
            scalar_id => scalar_data[2..].to_vec(),
        );

        table_commitment
            .try_append_rows(append_columns.inner_table(), &())
            .unwrap();

        let total_columns: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data,
            varchar_id => varchar_data,
            scalar_id => scalar_data,
        );

        let expected_table_commitment =
            TableCommitment::try_from_columns_with_offset(total_columns.inner_table(), 0, &())
                .unwrap();

        assert_eq!(table_commitment, expected_table_commitment);

        // matches append_owned_table result
        table_commitment_clone
            .append_owned_table(&append_columns, &())
            .unwrap();
        assert_eq!(table_commitment, table_commitment_clone)
    }

    #[test]
    fn we_cannot_append_mismatched_columns_to_table_commitment() {
        let base_table: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => [1i64, 2, 3, 4],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"],
        );
        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                base_table.inner_table(),
                0,
                &(),
            )
            .unwrap();
        let column_commitments = table_commitment.column_commitments().clone();

        let table_diff_type: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => ["5", "6", "7", "8"],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"],
        );
        assert!(matches!(
            table_commitment.try_append_rows(table_diff_type.inner_table(), &()),
            Err(AppendTableCommitmentError::AppendColumnCommitments(
                AppendColumnCommitmentsError::Mismatch(
                    ColumnCommitmentsMismatch::ColumnCommitmentMetadata(_)
                )
            ))
        ));

        // make sure the commitment wasn't mutated
        assert_eq!(table_commitment.num_rows(), 4);
        assert_eq!(table_commitment.column_commitments(), &column_commitments);
    }

    #[test]
    fn we_cannot_append_columns_with_duplicate_identifiers_to_table_commitment() {
        let column_id_a = "column_a".parse().unwrap();
        let column_id_b = "column_b".parse().unwrap();

        let column_data = OwnedColumn::<Curve25519Scalar>::BigInt(vec![1, 2, 3]);

        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                [(&column_id_a, &column_data), (&column_id_b, &column_data)],
                0,
                &(),
            )
            .unwrap();
        let column_commitments = table_commitment.column_commitments().clone();

        let append_column_result = table_commitment.try_append_rows(
            [
                (&column_id_a, &column_data),
                (&column_id_b, &column_data),
                (&column_id_a, &column_data),
            ],
            &(),
        );
        assert!(matches!(
            append_column_result,
            Err(AppendTableCommitmentError::AppendColumnCommitments(
                AppendColumnCommitmentsError::DuplicateIdentifiers(_)
            ))
        ));

        // make sure the commitment wasn't mutated
        assert_eq!(table_commitment.num_rows(), 3);
        assert_eq!(table_commitment.column_commitments(), &column_commitments);
    }

    #[test]
    fn we_cannot_append_columns_of_mixed_length_to_table_commitment() {
        let column_id_a: Identifier = "column_a".parse().unwrap();
        let column_id_b: Identifier = "column_b".parse().unwrap();
        let base_table: OwnedTable<Curve25519Scalar> = owned_table!(
            column_id_a => [1i64, 2, 3, 4],
            column_id_b => ["Lorem", "ipsum", "dolor", "sit"],
        );

        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                base_table.inner_table(),
                0,
                &(),
            )
            .unwrap();
        let column_commitments = table_commitment.column_commitments().clone();

        let column_a_append_data = OwnedColumn::<Curve25519Scalar>::BigInt(vec![5, 6, 7]);
        let column_b_append_data =
            OwnedColumn::VarChar(["amet", "consectetur"].map(String::from).to_vec());

        let append_result = table_commitment.try_append_rows(
            [
                (&column_id_a, &column_a_append_data),
                (&column_id_b, &column_b_append_data),
            ],
            &(),
        );
        assert!(matches!(
            append_result,
            Err(AppendTableCommitmentError::MixedLengthColumns(_))
        ));

        // make sure the commitment wasn't mutated
        assert_eq!(table_commitment.num_rows(), 4);
        assert_eq!(table_commitment.column_commitments(), &column_commitments);
    }

    #[test]
    fn we_can_extend_columns_to_table_commitment() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0].map(Curve25519Scalar::from);

        let initial_columns: OwnedTable<Curve25519Scalar> = owned_table!(
        bigint_id => bigint_data,
        varchar_id => varchar_data,
        );
        let mut table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                initial_columns.inner_table(),
                2,
                &(),
            )
            .unwrap();

        let new_columns = owned_table!(
        scalar_id => scalar_data,
        );
        table_commitment
            .try_extend_columns(new_columns.inner_table(), &())
            .unwrap();

        let expected_columns = owned_table!(
        bigint_id => bigint_data,
        varchar_id => varchar_data,
        scalar_id => scalar_data,
        );
        let expected_table_commitment =
            TableCommitment::try_from_columns_with_offset(expected_columns.inner_table(), 2, &())
                .unwrap();

        assert_eq!(table_commitment, expected_table_commitment);
    }

    #[test]
    fn we_can_add_table_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0].map(Curve25519Scalar::from);

        let columns_a: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[..2].to_vec(),
            varchar_id => varchar_data[..2].to_vec(),
            scalar_id => scalar_data[..2].to_vec(),
        );

        let table_commitment_a =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                columns_a.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let columns_b: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[2..].to_vec(),
            varchar_id => varchar_data[2..].to_vec(),
            scalar_id => scalar_data[2..].to_vec(),
        );
        let table_commitment_b =
            TableCommitment::try_from_columns_with_offset(columns_b.inner_table(), 2, &()).unwrap();

        let columns_sum: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data,
            varchar_id => varchar_data,
            scalar_id => scalar_data,
        );
        let table_commitment_sum =
            TableCommitment::try_from_columns_with_offset(columns_sum.inner_table(), 0, &())
                .unwrap();

        assert_eq!(
            table_commitment_a
                .clone()
                .try_add(table_commitment_b.clone())
                .unwrap(),
            table_commitment_sum
        );
        // commutativity
        assert_eq!(
            table_commitment_b.try_add(table_commitment_a).unwrap(),
            table_commitment_sum
        );
    }

    #[test]
    fn we_cannot_add_mismatched_table_commitments() {
        let base_table: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => [1i64, 2, 3, 4],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"],
        );
        let table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                base_table.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let table_diff_type: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => ["5", "6", "7", "8"],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"],
        );
        let table_commitment_diff_type =
            TableCommitment::try_from_columns_with_offset(table_diff_type.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            table_commitment.try_add(table_commitment_diff_type),
            Err(TableCommitmentArithmeticError::ColumnMismatch(_))
        ));
    }

    #[test]
    fn we_cannot_add_noncontiguous_table_commitments() {
        let base_table: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => [1i64, 2, 3, 4],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"],
        );
        let table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                base_table.inner_table(),
                5,
                &(),
            )
            .unwrap();

        let high_disjoint_table_commitment =
            TableCommitment::try_from_columns_with_offset(base_table.inner_table(), 10, &())
                .unwrap();
        assert!(matches!(
            table_commitment
                .clone()
                .try_add(high_disjoint_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let high_overlapping_table_commitment =
            TableCommitment::try_from_columns_with_offset(base_table.inner_table(), 7, &())
                .unwrap();
        assert!(matches!(
            table_commitment
                .clone()
                .try_add(high_overlapping_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let equal_range_table_commitment =
            TableCommitment::try_from_columns_with_offset(base_table.inner_table(), 5, &())
                .unwrap();
        assert!(matches!(
            table_commitment
                .clone()
                .try_add(equal_range_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let low_overlapping_table_commitment =
            TableCommitment::try_from_columns_with_offset(base_table.inner_table(), 3, &())
                .unwrap();
        assert!(matches!(
            table_commitment
                .clone()
                .try_add(low_overlapping_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let low_disjoint_table_commitment =
            TableCommitment::try_from_columns_with_offset(base_table.inner_table(), 0, &())
                .unwrap();
        assert!(matches!(
            table_commitment
                .clone()
                .try_add(low_disjoint_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));
    }

    #[test]
    fn we_can_sub_table_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0].map(Curve25519Scalar::from);

        let columns_low: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[..2].to_vec(),
            varchar_id => varchar_data[..2].to_vec(),
            scalar_id => scalar_data[..2].to_vec(),
        );
        let table_commitment_low =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                columns_low.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let columns_high: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[2..].to_vec(),
            varchar_id => varchar_data[2..].to_vec(),
            scalar_id => scalar_data[2..].to_vec(),
        );
        let table_commitment_high =
            TableCommitment::try_from_columns_with_offset(columns_high.inner_table(), 2, &())
                .unwrap();

        let columns_all: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data,
            varchar_id => varchar_data,
            scalar_id => scalar_data,
        );
        let table_commitment_all =
            TableCommitment::try_from_columns_with_offset(columns_all.inner_table(), 0, &())
                .unwrap();

        // case where we subtract the low commitment off the total to get the high commitment
        let high_difference = table_commitment_all
            .clone()
            .try_sub(table_commitment_low.clone())
            .unwrap();
        assert_eq!(
            high_difference.column_commitments().commitments(),
            table_commitment_high.column_commitments().commitments()
        );
        assert_eq!(high_difference.range(), table_commitment_high.range());

        // case where we subtract the high commitment off the total to get the low commitment
        let low_difference = table_commitment_all.try_sub(table_commitment_high).unwrap();
        assert_eq!(
            low_difference.column_commitments().commitments(),
            table_commitment_low.column_commitments().commitments()
        );
        assert_eq!(low_difference.range(), table_commitment_low.range());

        // subtraction for column metadata is tested more thoroughly at a lower level
    }

    #[test]
    fn we_cannot_sub_mismatched_table_commitments() {
        let base_table: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => [1i64, 2, 3, 4],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"],
        );
        let table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                base_table.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let table_diff_type: OwnedTable<Curve25519Scalar> = owned_table!(
            "column_a" => ["1", "2"],
            "column_b" => ["Lorem", "ipsum"],
        );
        let table_commitment_diff_type =
            TableCommitment::try_from_columns_with_offset(table_diff_type.inner_table(), 0, &())
                .unwrap();
        assert!(matches!(
            table_commitment.try_sub(table_commitment_diff_type),
            Err(TableCommitmentArithmeticError::ColumnMismatch(_))
        ));
    }

    #[test]
    fn we_cannot_sub_noncontiguous_table_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0].map(Curve25519Scalar::from);

        let columns_minuend: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[..].to_vec(),
            varchar_id => varchar_data[..].to_vec(),
            scalar_id => scalar_data[..].to_vec(),
        );

        let columns_subtrahend: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[..2].to_vec(),
            varchar_id => varchar_data[..2].to_vec(),
            scalar_id => scalar_data[..2].to_vec(),
        );

        let minuend_table_commitment =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                columns_minuend.inner_table(),
                4,
                &(),
            )
            .unwrap();

        let high_contiguous_table_commitment =
            TableCommitment::try_from_columns_with_offset(columns_subtrahend.inner_table(), 9, &())
                .unwrap();
        assert!(matches!(
            minuend_table_commitment
                .clone()
                .try_sub(high_contiguous_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let high_overlapping_table_commitment =
            TableCommitment::try_from_columns_with_offset(columns_subtrahend.inner_table(), 6, &())
                .unwrap();
        assert!(matches!(
            minuend_table_commitment
                .clone()
                .try_sub(high_overlapping_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let low_overlapping_table_commitment =
            TableCommitment::try_from_columns_with_offset(columns_subtrahend.inner_table(), 3, &())
                .unwrap();
        assert!(matches!(
            minuend_table_commitment
                .clone()
                .try_sub(low_overlapping_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));

        let low_contiguous_table_commitment =
            TableCommitment::try_from_columns_with_offset(columns_subtrahend.inner_table(), 2, &())
                .unwrap();
        assert!(matches!(
            minuend_table_commitment
                .clone()
                .try_sub(low_contiguous_table_commitment),
            Err(TableCommitmentArithmeticError::NonContiguous)
        ));
    }

    #[test]
    fn we_cannot_sub_commitments_with_negative_difference() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0].map(Curve25519Scalar::from);

        let columns_low: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[..2].to_vec(),
            varchar_id => varchar_data[..2].to_vec(),
            scalar_id => scalar_data[..2].to_vec(),
        );
        let table_commitment_low =
            TableCommitment::<CompressedRistretto>::try_from_columns_with_offset(
                columns_low.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let columns_high: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data[2..].to_vec(),
            varchar_id => varchar_data[2..].to_vec(),
            scalar_id => scalar_data[2..].to_vec(),
        );
        let table_commitment_high =
            TableCommitment::try_from_columns_with_offset(columns_high.inner_table(), 2, &())
                .unwrap();

        let columns_all: OwnedTable<Curve25519Scalar> = owned_table!(
            bigint_id => bigint_data,
            varchar_id => varchar_data,
            scalar_id => scalar_data,
        );
        let table_commitment_all =
            TableCommitment::try_from_columns_with_offset(columns_all.inner_table(), 0, &())
                .unwrap();

        // try to subtract the total commitment off the low to get the "negative" high commitment
        let try_negative_high_difference_result =
            table_commitment_low.try_sub(table_commitment_all.clone());
        assert!(matches!(
            try_negative_high_difference_result,
            Err(TableCommitmentArithmeticError::NegativeRange(_))
        ));

        // try to subtract the total commitment off the high to get the "negative" low commitment
        let try_negative_low_difference_result =
            table_commitment_high.try_sub(table_commitment_all);
        assert!(matches!(
            try_negative_low_difference_result,
            Err(TableCommitmentArithmeticError::NegativeRange(_))
        ));
    }
}
