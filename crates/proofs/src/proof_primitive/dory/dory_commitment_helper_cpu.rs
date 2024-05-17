use super::{DoryCommitment, DoryProverPublicSetup, DoryScalar, G1};
use crate::base::commitment::CommittableColumn;
use ark_ec::{pairing::Pairing, ScalarMul, VariableBaseMSM};
use core::iter::once;

fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment
where
    &'a T: Into<DoryScalar>,
{
    // Compute offsets for the matrix.
    let num_columns = 1 << setup.sigma();
    let first_row_offset = offset % num_columns;
    let rows_offset = offset / num_columns;
    let first_row_len = column.len().min(num_columns - first_row_offset);
    let remaining_elements_len = column.len() - first_row_len;
    let remaining_row_count = (remaining_elements_len + num_columns - 1) / num_columns;

    // Break column into rows.
    let (first_row, remaining_elements) = column.split_at(first_row_len);
    let remaining_rows = remaining_elements.chunks(num_columns);

    // Compute commitments for the rows.
    let first_row_commit = G1::msm_unchecked(
        &ScalarMul::batch_convert_to_mul_base(
            &setup.public_parameters().Gamma_1[first_row_offset..num_columns],
        ),
        &Vec::from_iter(first_row.iter().map(|s| s.into().0)),
    );
    let remaining_row_commits = remaining_rows.map(|row| {
        G1::msm_unchecked(
            &ScalarMul::batch_convert_to_mul_base(
                &setup.public_parameters().Gamma_1[..num_columns],
            ),
            &Vec::from_iter(row.iter().map(|s| s.into().0)),
        )
    });

    // Compute the commitment for the entire matrix.
    DoryCommitment(Pairing::multi_pairing(
        once(first_row_commit).chain(remaining_row_commits),
        &setup.public_parameters().Gamma_2[rows_offset..(rows_offset + remaining_row_count + 1)],
    ))
}

fn compute_dory_commitment(
    committable_column: &CommittableColumn,
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment {
    match committable_column {
        CommittableColumn::Scalar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::VarChar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
    }
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    committable_columns
        .iter()
        .map(|column| compute_dory_commitment(column, offset, setup))
        .collect()
}
