use super::{G1Affine, F};
use crate::{
    base::{commitment::CommittableColumn, database::ColumnType},
    proof_primitive::dory::offset_to_bytes::OffsetToBytes,
};
use ark_bls12_381::G1Projective;
use ark_ff::MontFp;
use ark_std::ops::Mul;
use rayon::prelude::*;

const BYTE_SIZE: usize = 8;
const OFFSET_SIZE: usize = 2;

/// Returns a bit table vector related to each of the committable columns data size.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_matrix_commitment_columns` - The number of generators used for msm.
fn output_bit_table(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_matrix_commitment_columns: usize,
) -> Vec<u32> {
    committable_columns
        .iter()
        .flat_map(|column| {
            let bit_size = column.column_type().bit_size();
            itertools::repeat_n(
                bit_size,
                num_sub_commits_update(column, offset, num_matrix_commitment_columns),
            )
        })
        .collect()
}

/// Returns the number of sub commitments needed for
/// each full commitment in the packed_msm function.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_matrix_commitment_columns` - The number of generators used for msm.
pub fn num_sub_commits_update(
    column: &CommittableColumn,
    offset: usize,
    num_matrix_commitment_columns: usize,
) -> usize {
    (column.len() + offset + num_matrix_commitment_columns - 1) / num_matrix_commitment_columns
}

/// Returns the minimum value of a column as F.
///
/// # Arguments
///
/// * `column_type` - The type of a committable column.
const fn min_as_f(column_type: ColumnType) -> F {
    match column_type {
        ColumnType::SmallInt => MontFp!("-32768"),
        ColumnType::Int => MontFp!("-2147483648"),
        ColumnType::BigInt | ColumnType::TimestampTZ(_, _) => MontFp!("-9223372036854775808"),
        ColumnType::Int128 => MontFp!("-170141183460469231731687303715884105728"),
        ColumnType::Decimal75(_, _)
        | ColumnType::Scalar
        | ColumnType::VarChar
        | ColumnType::Boolean => MontFp!("0"),
    }
}

const fn is_signed(column_type: ColumnType) -> bool {
    match column_type {
        ColumnType::SmallInt
        | ColumnType::Int
        | ColumnType::BigInt
        | ColumnType::Int128
        | ColumnType::TimestampTZ(_, _) => true,
        ColumnType::Decimal75(_, _)
        | ColumnType::Scalar
        | ColumnType::VarChar
        | ColumnType::Boolean => false,
    }
}

/// Packs bits of a committable column into the packed scalars array.
/// Will offset signed values by the minimum of the data type.
///
/// # Arguments
///
/// * `column` - A reference to the committable column to be packed.
/// * `packed_scalars` - A mutable reference to the array where the packed scalars will be stored.
/// * `current_bit_table_sum` - The current sum of the bit table up to the current sub commit.
/// * `offset` - The offset to the data.
/// * `current_byte_size` - The current byte size of the column.
/// * `bit_table_sum_in_bytes` - The full bit table size in bytes.
/// * `num_columns` - The number of columns in a matrix commitment.
fn pack_bit<const LEN: usize, T: OffsetToBytes<LEN>>(
    column: &[T],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    offset: usize,
    current_byte_size: usize,
    bit_table_sum_in_bytes: usize,
    num_columns: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    column.iter().enumerate().for_each(|(i, value)| {
        let row_offset = ((i + offset) % num_columns) * bit_table_sum_in_bytes;
        let col_offset = current_byte_size * ((i + offset) / num_columns);
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx..offset_idx + current_byte_size]
            .copy_from_slice(&value.offset_to_bytes()[..]);
    });
}

/// Packs the offset bits of a committable column into the packed scalars at the end of the array.
/// The offsets are 8-bit values used to handle the signed values.
///
/// # Arguments
///
/// * `column` -  A reference to a signed committable column that needs offsets calculated.
/// * `packed_scalars` - A mutable reference to the array where the packed scalars will be stored.
/// * `current_bit_table_sum` - The current sum of the bit table up to the current column.
/// * `offset` - The offset to the data.
/// * `bit_table_sum_in_bytes` - The full bit table size in bytes.
/// * `num_columns` - The number of columns in a matrix commitment.
fn pack_offset_bit<const LEN: usize, T: OffsetToBytes<LEN>>(
    column: &[T],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    offset: usize,
    bit_table_sum_in_bytes: usize,
    num_columns: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    column.iter().enumerate().for_each(|(i, _)| {
        let row_offset = ((i + offset) % num_columns) * bit_table_sum_in_bytes;
        let col_offset = (i + offset) / num_columns;
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx] = 1_u8;
    });
}

/// Returns the bit table and packed scalar array to be used in Blitzar's packed_msm function.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_columns` - The number of columns in a matrix commitment.
/// * `num_sub_commits_per_full_commit` - The number of sub commits needed for
///                                       each full commit for the packed_msm function.
///
/// # Example
///
/// ```ignore
/// let committable_columns = [
///     CommittableColumn::SmallInt(&[0, 1, 2]),
///     CommittableColumn::SmallInt(&[3, 4, 5, 6, 7]),
/// ];
/// let offset = 1;
/// let num_columns = 3;
///
/// let num_sub_commits_per_full_commit = sub_commits_per_full_commit(&committable_columns, offset, num_columns);
///
/// let (bit_table, packed_scalars) = bit_table_and_scalars_for_packed_msm(
///     &committable_columns,
///     offset,
///     num_columns,
///     num_sub_commits_per_full_commit,
/// );
///
/// assert_eq!(num_of_commits, 2);
/// assert_eq!(bit_table, [16, 16, 16, 16, 8, 8, 8, 8]);
/// assert_eq!(packed_scalars.len(), 36); // num_columns * bit_table_sum / BYTE_SIZE
/// assert_eq!(packed_scalars, [0,   0, 2, 128, 0,   0, 5, 128, 0, 1, 0, 1,
///                             0, 128, 0,   0, 3, 128, 6, 128, 1, 0, 1, 1,
///                             1, 128, 0,   0, 4, 128, 7, 128, 1, 0, 1, 1]);
/// ```
#[tracing::instrument(
    name = "pack_scalars::bit_table_and_scalars_for_packed_msm_OLD",
    level = "debug",
    skip_all
)]
pub fn bit_table_and_scalars_for_packed_msm_OLD(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_columns: usize,
    num_sub_commits_per_full_commit: usize,
) -> (Vec<u32>, Vec<u8>) {
    // Get a bit table that represented each of the committable columns bit size.
    let bit_table_full_commits = output_bit_table(committable_columns);

    // Repeat the bit table to account for the appropriate number of sub commitments per full commit.
    let mut bit_table = repeat_bit_table(bit_table_full_commits, num_sub_commits_per_full_commit);
    let bit_table_sub_commits_sum = bit_table.iter().sum::<u32>() as usize;

    // Double the bit table to handle handle the BYTE_SIZE offsets.
    bit_table.extend(std::iter::repeat(BYTE_SIZE as u32).take(bit_table.len()));
    let bit_table_sum_in_bytes = bit_table.iter().sum::<u32>() as usize / BYTE_SIZE;

    // Create the packed_scalar vector.
    let mut packed_scalars = vec![0_u8; bit_table_sum_in_bytes * num_columns];

    // For each committable column, pack the data into the packed_scalar array.
    committable_columns
        .iter()
        .enumerate()
        .for_each(|(i, column)| {
            // Get the running sum of the bit table for the signed values.
            let current_bit_table_sum = if i > 0 {
                bit_table
                    .iter()
                    .take(i * num_sub_commits_per_full_commit)
                    .sum::<u32>() as usize
            } else {
                0
            };

            // Get the running sum of the bit table for the offsets.
            let bit_table_offset_sum =
                bit_table_sub_commits_sum + i * BYTE_SIZE * num_sub_commits_per_full_commit;

            // Get the byte size of the column of data.
            let byte_size = committable_columns[i].column_type().byte_size();

            // Pack the signed bits and offset bits into the packed_scalars array.
            match column {
                CommittableColumn::SmallInt(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit_old(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Int(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit_old(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::BigInt(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit_old(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Int128(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit_old(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::TimestampTZ(_, _, column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit_old(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Boolean(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Decimal75(_, _, column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Scalar(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::VarChar(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
            }
        });

    (bit_table, packed_scalars)
}


















































/// Modifies the signed matrix commitment columns by adding the offset to the matrix commitment columns.
///
/// # Arguments
///
/// * `sub_commits` - A reference to the signed sub-commits.
/// * `bit_table` - A reference to the bit table used by the packed_msm function.
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_matrix_commitment_columns` - The number of sub-commits needed for
///                                     each full commit for the packed_msm function.
#[tracing::instrument(name = "pack_scalars::modify_commits", level = "debug", skip_all)]
pub fn modify_commits(
    sub_commits: &[G1Affine],
    bit_table: &[u32],
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_matrix_commitment_columns: usize,
) -> Vec<G1Affine> {
    // Set parameters
    let num_offset_commits = OFFSET_SIZE + committable_columns.len();
    let num_sub_commits_in_full_commit = bit_table.len() - num_offset_commits;
    assert_eq!(
        num_sub_commits_in_full_commit + num_offset_commits,
        sub_commits.len()
    );

    // Spit the sub-commits and offset commits.
    let (signed_sub_commits, offset_sub_commits) =
        sub_commits.split_at(num_sub_commits_in_full_commit);
    assert_eq!(signed_sub_commits.len(), num_sub_commits_in_full_commit);
    assert_eq!(offset_sub_commits.len(), num_offset_commits);

    let mut modifed_commits: Vec<G1Projective> =
        vec![G1Projective::default(); signed_sub_commits.len()];

    let mut k = 0;
    for (i, column) in committable_columns.iter().enumerate() {
        let num_sub_commits = num_sub_commits_update(column, offset, num_matrix_commitment_columns);
        if is_signed(column.column_type()) {
            let min = min_as_f(column.column_type());
            let offset_sub_commit_first = offset_sub_commits[0].mul(min);
            let offset_sub_commit_middle = offset_sub_commits[1].mul(min);
            let offset_sub_commit_last = offset_sub_commits[OFFSET_SIZE + i].mul(min);

            match num_sub_commits {
                1 => {
                    modifed_commits[k] = signed_sub_commits[k] + offset_sub_commit_last;
                    k += 1;
                }
                2 => {
                    modifed_commits[k] = signed_sub_commits[k] + offset_sub_commit_first;
                    k += 1;
                    modifed_commits[k] = signed_sub_commits[k] + offset_sub_commit_last;
                    k += 1;
                }
                _ => {
                    modifed_commits[k] = signed_sub_commits[k] + offset_sub_commit_first;
                    k += 1;
                    (1..num_sub_commits - 1).for_each(|_| {
                        modifed_commits[k] = signed_sub_commits[k] + offset_sub_commit_middle;
                        k += 1;
                    });
                    modifed_commits[k] = signed_sub_commits[k] + offset_sub_commit_last;
                    k += 1;
                }
            }
        } else {
            (0..num_sub_commits).for_each(|_| {
                modifed_commits[k] = signed_sub_commits[k].into();
                k += 1;
            });
        }
    }

    modifed_commits.into_iter().map(Into::into).collect()
}

/// Packs the offset bits of a committable column into the packed scalars at the end of the array.
/// The offsets are 8-bit values used to handle the signed values.
///
/// # Arguments
///
/// * `offset_column` -  A reference to the offset column, should be 0 or 1.
/// * `packed_scalars` - A mutable reference to the array where the packed scalars will be stored.
/// * `current_bit_table_sum` - The current sum of the bit table up to the current column.
/// * `offset` - The offset to the data.
/// * `bit_table_sum_in_bytes` - The full bit table size in bytes.
/// * `num_columns` - The number of columns in a matrix commitment.
fn pack_offset_bit(
    offset_column: &[u8],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    bit_table_sum_in_bytes: usize,
    num_columns: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    offset_column.iter().enumerate().for_each(|(i, value)| {
        let row_offset = (i % num_columns) * bit_table_sum_in_bytes;
        let col_offset = i / num_columns;
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx] = *value;
    });
}

fn offset_column(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_columns: usize,
) -> Vec<u8> {
    assert!(
        offset < num_columns,
        "offset {} must be less than the number of columns {}",
        offset,
        num_columns
    );

    let mut offset_column = vec![0_u8; (OFFSET_SIZE + committable_columns.len()) * num_columns];

    let has_at_least_one_signed_column = committable_columns
        .par_iter()
        .any(|column| is_signed(column.column_type()));

    if has_at_least_one_signed_column {
        // Set the offset and the column of all ones in a single loop
        (0..num_columns).for_each(|i| {
            if i >= offset {
                offset_column[i] = 1_u8;
            }
            offset_column[num_columns + i] = 1_u8;
        });

        // Set the remaining columns
        for (j, column) in committable_columns.iter().enumerate() {
            if is_signed(column.column_type()) {
                let column_len = column.len();
                let first_value = if (offset + column_len) <= num_columns {
                    offset
                } else {
                    0
                };
                let last_value = if (offset + column_len) % num_columns == 0 {
                    num_columns
                } else {
                    (offset + column_len) % num_columns
                };

                (first_value..last_value).for_each(|i| {
                    offset_column[((2 + j) * num_columns) + i] = 1_u8;
                });
            }
        }
    }

    offset_column
}

/// Returns the bit table and packed scalar array to be used in Blitzar's packed_msm function.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_columns` - The number of columns in a matrix commitment.
///
/// # Example
///
/// ```ignore
/// let committable_columns = [
///     CommittableColumn::SmallInt(&[0, 1, 2]),
///     CommittableColumn::SmallInt(&[3, 4, 5, 6, 7]),
/// ];
/// let offset = 1;
/// let num_columns = 3;
///
/// let (bit_table, packed_scalars) = bit_table_and_scalars_for_packed_msm(
///     &committable_columns,
///     offset,
///     num_columns,
/// );
///
/// assert_eq!(num_of_commits, 2);
/// assert_eq!(bit_table, [16, 16, 16, 16, 8, 8, 8, 8]);
/// assert_eq!(packed_scalars.len(), 36); // num_columns * bit_table_sum / BYTE_SIZE
/// assert_eq!(packed_scalars, [0,   0, 2, 128, 0,   0, 5, 128, 0, 1, 0, 1,
///                             0, 128, 0,   0, 3, 128, 6, 128, 1, 0, 1, 1,
///                             1, 128, 0,   0, 4, 128, 7, 128, 1, 0, 1, 1]);
/// ```
#[tracing::instrument(
    name = "pack_scalars::bit_table_and_scalars_for_packed_msm",
    level = "debug",
    skip_all
)]
pub fn bit_table_and_scalars_for_packed_msm(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_columns: usize,
) -> (Vec<u32>, Vec<u8>) {
    // Get the bit table to account for the appropriate number of sub commitments per full commit.
    let mut bit_table = output_bit_table(committable_columns, offset, num_columns);
    let bit_table_sub_commits_sum = bit_table.iter().sum::<u32>() as usize;

    // Add offsets to the bit table.
    bit_table
        .extend(std::iter::repeat(BYTE_SIZE as u32).take(OFFSET_SIZE + committable_columns.len()));
    let bit_table_sum_in_bytes = bit_table.iter().sum::<u32>() as usize / BYTE_SIZE;

    // Create the packed_scalar vector.
    let mut packed_scalars = vec![0_u8; bit_table_sum_in_bytes * num_columns];

    // Pack the offsets, used to handed signed values, into the packed_scalars array.
    let offset_column = offset_column(committable_columns, offset, num_columns);
    pack_offset_bit(
        &offset_column,
        &mut packed_scalars,
        bit_table_sub_commits_sum,
        bit_table_sum_in_bytes,
        num_columns,
    );

    // For each committable column, pack the data into the packed_scalar array.
    let mut num_sub_commits_completed = 0;
    committable_columns
        .iter()
        .enumerate()
        .for_each(|(i, column)| {
            // Get the running sum of the bit table for the signed values.
            let current_bit_table_sum = bit_table
                .iter()
                .take(num_sub_commits_completed)
                .sum::<u32>() as usize;

            num_sub_commits_completed += num_sub_commits_update(column, offset, num_columns);

            // Get the byte size of the column of data.
            let byte_size = committable_columns[i].column_type().byte_size();

            // Pack the signed bits and offset bits into the packed_scalars array.
            match column {
                CommittableColumn::SmallInt(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Int(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::BigInt(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Int128(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::TimestampTZ(_, _, column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Boolean(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Decimal75(_, _, column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Scalar(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::VarChar(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
            }
        });

    (bit_table, packed_scalars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    /*
    #[test]
    fn we_can_get_max_committable_column_length_of_the_same_type() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4]]),
        ];

        let max_column_length = max_committable_column_length(&committable_columns);
        assert_eq!(max_column_length, 2);
    }
    */

    /*
    #[test]
    fn we_can_get_max_committable_column_length_of_different_types() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3]),
            CommittableColumn::Int(&[1, 2, 3]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4, 5, 6]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 0, 0, 0], [2, 0, 0, 0]],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let max_column_length = max_committable_column_length(&committable_columns);
        assert_eq!(max_column_length, 6);
    }
    */

    #[test]
    fn we_can_get_a_bit_table() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let offset = 0;
        let num_matrix_commitment_columns = 1;
        let bit_table: Vec<u32> =
            output_bit_table(&committable_columns, offset, num_matrix_commitment_columns);
        let expected = [
            16, 32, 32, 64, 64, 64, 128, 128, 128, 128, 256, 256, 256, 256, 256, 256, 256, 256,
            256, 256, 256, 256, 8, 8, 64,
        ];
        assert_eq!(bit_table, expected);
    }

    #[test]
    fn we_can_get_a_bit_table_with_offset() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let offset = 1;
        let num_matrix_commitment_columns = 1;
        let bit_table: Vec<u32> =
            output_bit_table(&committable_columns, offset, num_matrix_commitment_columns);
        let expected = [
            16, 16, 32, 32, 32, 64, 64, 64, 64, 128, 128, 128, 128, 128, 256, 256, 256, 256, 256,
            256, 256, 256, 256, 256, 256, 256, 256, 256, 256, 8, 8, 8, 64, 64,
        ];
        assert_eq!(bit_table, expected);
    }

    #[test]
    fn we_can_get_the_num_of_sub_commits_with_more_rows_than_cols_update() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let offset = 0;
        let num_matrix_commitment_columns = 1;
        let expected: Vec<usize> = vec![1, 2, 3, 4, 5, 4, 3, 2, 1];
        let num_sub_commits: Vec<usize> = (0..committable_columns.len())
            .map(|i| {
                num_sub_commits_update(
                    &committable_columns[i],
                    offset,
                    num_matrix_commitment_columns,
                )
            })
            .collect();

        assert_eq!(num_sub_commits, expected);
    }

    #[test]
    fn we_can_get_the_num_of_sub_commits_offset_with_more_rows_than_cols_update() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let offset = 2;
        let num_matrix_commitment_columns = 1;
        let expected: Vec<usize> = vec![3, 4, 5, 6, 7, 6, 5, 4, 3];
        let num_sub_commits: Vec<usize> = (0..committable_columns.len())
            .map(|i| {
                num_sub_commits_update(
                    &committable_columns[i],
                    offset,
                    num_matrix_commitment_columns,
                )
            })
            .collect();

        assert_eq!(num_sub_commits, expected);
    }

    #[test]
    fn we_can_get_the_num_of_sub_commits_with_less_rows_than_cols_update() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let offset = 0;
        let num_matrix_commitment_columns = 4;
        let expected: Vec<usize> = vec![1, 1, 1, 1, 2, 1, 1, 1, 1];
        let num_sub_commits: Vec<usize> = (0..committable_columns.len())
            .map(|i| {
                num_sub_commits_update(
                    &committable_columns[i],
                    offset,
                    num_matrix_commitment_columns,
                )
            })
            .collect();

        assert_eq!(num_sub_commits, expected);
    }

    #[test]
    fn we_can_get_the_num_of_sub_commits_offset_with_less_rows_than_cols_update() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let offset = 1;
        let num_matrix_commitment_columns = 4;
        let expected: Vec<usize> = vec![1, 1, 1, 2, 2, 2, 1, 1, 1];
        let num_sub_commits: Vec<usize> = (0..committable_columns.len())
            .map(|i| {
                num_sub_commits_update(
                    &committable_columns[i],
                    offset,
                    num_matrix_commitment_columns,
                )
            })
            .collect();

        assert_eq!(num_sub_commits, expected);
    }

    /*
    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_less_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0]]),
        ];

        let offset = 0;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 1);
    }
     */

    /*
    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_offset_and_less_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0]]),
        ];

        let offset = 5;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);
    }
     */

    /*
    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_more_rows_than_generators() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        ];

        let offset = 0;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 5);
    }
     */

    /*
    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_offset_and_more_rows_than_generators() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        ];

        let offset = 1;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 6);
    }
     */

    /*
    #[test]
    fn we_can_create_a_mixed_packed_scalar_with_more_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            ]),
            CommittableColumn::Int(&[
                19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
            ]),
            CommittableColumn::SmallInt(&[
                38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
            ]),
        ];

        let num_columns = 1 << 2;
        let offset = 0;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 5);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_bit_table = [
            16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8,
        ];

        let expected_packed_scalar = [
            0, 128, 4, 128, 8, 128, 12, 128, 16, 128, 19, 0, 0, 128, 23, 0, 0, 128, 27, 0, 0, 128,
            31, 0, 0, 128, 35, 0, 0, 128, 38, 128, 42, 128, 46, 128, 50, 128, 54, 128, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 128, 5, 128, 9, 128, 13, 128, 17, 128, 20, 0, 0,
            128, 24, 0, 0, 128, 28, 0, 0, 128, 32, 0, 0, 128, 36, 0, 0, 128, 39, 128, 43, 128, 47,
            128, 51, 128, 55, 128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 128, 6, 128, 10,
            128, 14, 128, 18, 128, 21, 0, 0, 128, 25, 0, 0, 128, 29, 0, 0, 128, 33, 0, 0, 128, 37,
            0, 0, 128, 40, 128, 44, 128, 48, 128, 52, 128, 56, 128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 3, 128, 7, 128, 11, 128, 15, 128, 0, 0, 22, 0, 0, 128, 26, 0, 0, 128,
            30, 0, 0, 128, 34, 0, 0, 128, 0, 0, 0, 0, 41, 128, 45, 128, 49, 128, 53, 128, 0, 0, 1,
            1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0,
        ];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */

    /*
    #[test]
    fn we_can_create_a_mixed_packed_scalar_with_offset_and_same_num_of_rows_and_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[0, 1, 2, 3]),
            CommittableColumn::Int(&[4, 5, 6, 7]),
            CommittableColumn::SmallInt(&[8, 9, 10, 11]),
        ];

        let num_columns = 1 << 2;
        let offset = 5;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 3);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_bit_table = [
            16, 16, 16, 32, 32, 32, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        ];

        let expected_packed_scalar = [
            0, 0, 0, 0, 3, 128, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 128, 0, 0, 0, 0, 11, 128, 0, 0, 1,
            0, 0, 1, 0, 0, 1, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 4, 0, 0, 128, 0, 0, 0, 0, 0, 0, 8,
            128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 128, 0, 0, 0, 0, 0, 0, 5, 0, 0, 128, 0,
            0, 0, 0, 0, 0, 9, 128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 2, 128, 0, 0, 0, 0, 0, 0,
            6, 0, 0, 128, 0, 0, 0, 0, 0, 0, 10, 128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
        ];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */

    /*
    #[test]
    fn we_can_pack_empty_scalars() {
        let committable_columns = [];

        let (bit_table, packed_scalar) =
            bit_table_and_scalars_for_packed_msm_OLD(&committable_columns, 0, 1, 0);

        assert!(bit_table.is_empty());
        assert!(packed_scalar.is_empty());
    }
     */

    /*
    #[test]
    fn we_can_pack_scalars_with_one_full_row() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 0;
        let num_columns = 1 << 1;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 1);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 1, 1, 2, 0, 0, 0, 0, 0, 0, 128, 4,
            0, 0, 0, 0, 0, 0, 128, 1, 1,
        ];

        let expected_bit_table = [64, 64, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */

    #[test]
    fn we_can_pack_scalars_with_one_full_row_update() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 0;
        let num_columns = 1 << 1;

        let (bit_table, packed_scalar) =
            bit_table_and_scalars_for_packed_msm(&committable_columns, offset, num_columns);

        let expected_packed_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0, 0,
            128, 4, 0, 0, 0, 0, 0, 0, 128, 1, 1, 1, 1,
        ];

        let expected_bit_table = [64, 64, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_pack_scalars_with_offset_one_full_row_update() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 1;
        let num_columns = 1 << 1;

        let (bit_table, packed_scalar) =
            bit_table_and_scalars_for_packed_msm(&committable_columns, offset, num_columns);

        let expected_packed_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
            0, 0, 0, 128, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_get_simple_offset_columns() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        // No offset
        let offset = 0;
        let num_columns = 1 << 1;

        let expected: Vec<u8> = vec![1, 1, 1, 1, 1, 1, 1, 1];

        let offset_columns = offset_column(&committable_columns, offset, num_columns);

        assert_eq!(offset_columns, expected);

        // Offset
        let offset = 1;
        let num_columns = 1 << 1;

        let expected: Vec<u8> = vec![0, 1, 1, 1, 1, 0, 1, 0];

        let offset_columns = offset_column(&committable_columns, offset, num_columns);

        assert_eq!(offset_columns, expected);

        // One column
        let offset = 0;
        let num_columns = 1;

        let expected: Vec<u8> = vec![1, 1, 1, 1];

        let offset_columns = offset_column(&committable_columns, offset, num_columns);

        assert_eq!(offset_columns, expected);
    }

    #[test]
    #[should_panic(expected = "offset 1 must be less than the number of columns 1")]
    fn we_can_panic_when_offset_is_not_less_than_num_of_columns_when_getting_offsets() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 1;
        let num_columns = 1;
        let _ = offset_column(&committable_columns, offset, num_columns);
    }

    #[test]
    fn we_can_get_complex_offset_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6]),
            CommittableColumn::BigInt(&[1, 2, 3, 4, 5, 6, 7]),
            CommittableColumn::Int128(&[1, 2, 3, 4, 5, 6, 7, 8]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false, true, false, true]),
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                &[1, 2, 3, 4, 5],
            ),
        ];

        let offset = 0;
        let num_columns = 1 << 2;
        let offset_column: Vec<u8> = offset_column(&committable_columns, offset, num_columns);
        let expected: Vec<u8> = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
        ];
        assert_eq!(offset_column, expected);
    }

    #[test]
    fn we_can_get_complex_offset_columns_with_offset() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6]),
            CommittableColumn::BigInt(&[1, 2, 3, 4, 5, 6, 7]),
            CommittableColumn::Int128(&[1, 2, 3, 4, 5, 6, 7, 8]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false, true, false, true]),
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                &[1, 2, 3, 4, 5],
            ),
        ];

        let offset = 3;
        let num_columns = 1 << 2;
        let offset_column: Vec<u8> = offset_column(&committable_columns, offset, num_columns);
        let expected: Vec<u8> = vec![
            0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
        ];
        assert_eq!(offset_column, expected);
    }

    /*
    #[test]
    fn we_can_pack_scalars_with_more_than_one_row() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 0;
        let num_columns = 1 << 0;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0,
            0, 0, 0, 0, 128, 1, 1, 1, 1,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */

    /*
    #[test]
    fn we_can_pack_scalars_with_one_full_row_with_offset() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 1;
        let num_columns = 1 << 1;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
            0, 0, 0, 128, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */

    /*
    #[test]
    fn we_can_pack_scalars_with_offset_and_more_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 1;
        let num_columns = 1 << 0;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 3);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0,
            0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128, 0, 1, 1, 0, 1, 1,
        ];

        let expected_bit_table = [64, 64, 64, 64, 64, 64, 8, 8, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */

    /*
    #[test]
    fn we_can_create_a_mixed_packed_scalar_with_offset_and_more_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[0, 1, 2, 3, 4, 5]),
            CommittableColumn::Int(&[6, 7, 8, 9]),
            CommittableColumn::Scalar(vec![[10, 0, 0, 0], [11, 0, 0, 0], [12, 0, 0, 0]]),
        ];

        let offset = 0;
        let num_columns = 3;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm_OLD(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            0, 128, 3, 128, 6, 0, 0, 128, 9, 0, 0, 128, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1,
            128, 4, 128, 7, 0, 0, 128, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 2, 128, 5,
            128, 8, 0, 0, 128, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0,
        ];

        let expected_bit_table = [16, 16, 32, 32, 64 * 4, 64 * 4, 8, 8, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
     */
}
