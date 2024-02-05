//! Contains the utility functions for the `GroupByExpr` node.

use super::filter_column_by_index;
use crate::base::{database::Column, scalar::ArkScalar};
use bumpalo::Bump;
use core::cmp::Ordering;
use itertools::Itertools;
use rayon::prelude::ParallelSliceMut;
use thiserror::Error;

/// The output of the `aggregate_columns` function.
pub struct AggregatedColumns<'a> {
    /// The columns that are being grouped by. These are all unique and correspond to each group.
    /// This is effectively just the original group_by columns filtered by the selection.
    pub group_by_columns: Vec<Column<'a, ArkScalar>>,
    /// Resulting sums of the groups for the columns in `sum_columns_in`.
    pub sum_columns: Vec<&'a [ArkScalar]>,
    /// The number of rows in each group.
    pub count_column: &'a [i64],
}
#[derive(Error, Debug)]
pub enum AggregateColumnsError {
    #[error("Column length mismatch")]
    ColumnLengthMismatch,
}

/// This is a function that gives the result of a group by query similar to the following:
/// ```sql
///     SELECT <group_by[0]>, <group_by[1]>, ..., SUM(<sum_columns[0]>), SUM(<sum_columns[1]>), ..., COUNT(*)
///         WHERE selection GROUP BY <group_by[0]>, <group_by[1]>, ...
/// ```
///
/// This function takes a selection vector and a set of group_by and sum columns and returns
/// the given columns aggregated by the group_by columns only for the selected rows.
pub fn aggregate_columns<'a>(
    alloc: &'a Bump,
    group_by_columns_in: &[Column<'a, ArkScalar>],
    sum_columns_in: &[Column<ArkScalar>],
    selection_column_in: &[bool],
) -> Result<AggregatedColumns<'a>, AggregateColumnsError> {
    for col in group_by_columns_in {
        if col.len() != selection_column_in.len() {
            return Err(AggregateColumnsError::ColumnLengthMismatch);
        }
    }
    for col in sum_columns_in {
        if col.len() != selection_column_in.len() {
            return Err(AggregateColumnsError::ColumnLengthMismatch);
        }
    }

    // `filtered_indexes` is a vector of indexes of the rows that are selected. We sort this vector
    // so that all the rows in the same group are next to each other.
    let mut filtered_indexes = Vec::from_iter(
        selection_column_in
            .iter()
            .enumerate()
            .filter(|&(_, &b)| b)
            .map(|(i, _)| i),
    );
    filtered_indexes
        .par_sort_unstable_by(|&a, &b| compare_indexes_by_columns(group_by_columns_in, a, b));

    // `group_by_result_indexes` gives a single index for each group in `filtered_indexes`. It does
    // not matter which index is chosen for each group, so we choose the first one. This is only used
    // to extract the `group_by_columns_out`, which is the same for all elements in the group.
    let (counts, group_by_result_indexes): (Vec<_>, Vec<_>) = filtered_indexes
        .iter()
        .dedup_by_with_count(|&&a, &&b| {
            compare_indexes_by_columns(group_by_columns_in, a, b) == Ordering::Equal
        })
        .multiunzip();
    let group_by_columns_out = Vec::from_iter(
        group_by_columns_in
            .iter()
            .map(|column| filter_column_by_index(alloc, column, &group_by_result_indexes)),
    );

    // This calls the `sum_aggregate_column_by_index_counts` function on each column in `sum_columns`
    // and gives a vector of `ArkScalar` slices
    let sum_columns_out = Vec::from_iter(sum_columns_in.iter().map(|column| {
        sum_aggregate_column_by_index_counts(alloc, column, &counts, &filtered_indexes)
    }));

    // Cast the counts to something compatible with BigInt.
    let count_column_out = alloc.alloc_slice_fill_iter(counts.into_iter().map(|c| c as i64));

    Ok(AggregatedColumns {
        group_by_columns: group_by_columns_out,
        sum_columns: sum_columns_out,
        count_column: count_column_out,
    })
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped sums of `column`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `column`.
///
/// See [`sum_aggregate_slice_by_index_counts`] for an example. This is a helper wrapper around that function.
pub(super) fn sum_aggregate_column_by_index_counts<'a>(
    alloc: &'a Bump,
    column: &Column<ArkScalar>,
    counts: &[usize],
    indexes: &[usize],
) -> &'a [ArkScalar] {
    match column {
        Column::Scalar(col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::BigInt(col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),
        Column::Int128(col) => sum_aggregate_slice_by_index_counts(alloc, col, counts, indexes),

        Column::VarChar(_) => panic!("Cannot sum varchar columns"),
        Column::Decimal75(_, _, _col) => {
            todo!()
        }
    }
}

/// Returns a slice with the lifetime of `alloc` that contains the grouped sums of `slice`.
/// The `counts` slice contains the number of elements in each group and the `indexes` slice
/// contains the indexes of the elements in `slice`.
///
/// For example:
/// ```ignore
/// let slice_a = &[
///     100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
/// ];
/// let indexes = &[12, 11, 1, 10, 2, 3, 6, 14, 13, 9];
/// let counts = &[3, 3, 4];
/// let expected = &[
///     ArkScalar::from(112 + 111 + 101),
///     ArkScalar::from(110 + 102 + 103),
///     ArkScalar::from(106 + 114 + 113 + 109),
/// ];
/// let alloc = Bump::new();
/// let result = sum_aggregate_slice_by_index_counts(&alloc, slice_a, counts, indexes);
/// assert_eq!(result, expected);
/// ```
pub(super) fn sum_aggregate_slice_by_index_counts<'a, T: Copy + Into<ArkScalar>>(
    alloc: &'a Bump,
    slice: &[T],
    counts: &[usize],
    indexes: &[usize],
) -> &'a [ArkScalar] {
    let mut index = 0;
    alloc.alloc_slice_fill_iter(counts.iter().map(|&count| {
        let start = index;
        index += count;
        indexes[start..index]
            .iter()
            .map(|&i| Into::<ArkScalar>::into(slice[i]))
            .sum()
    }))
}

/// Compares the tuples (group_by[0][i], group_by[1][i], ...) and
/// (group_by[0][j], group_by[1][j], ...) in lexicographic order.
pub(super) fn compare_indexes_by_columns(
    group_by: &[Column<ArkScalar>],
    i: usize,
    j: usize,
) -> Ordering {
    group_by
        .iter()
        .map(|col| match col {
            Column::Scalar(col) => col[i].cmp(&col[j]),
            Column::BigInt(col) => col[i].cmp(&col[j]),
            Column::Int128(col) => col[i].cmp(&col[j]),
            Column::VarChar((col, _)) => col[i].cmp(col[j]),
            Column::Decimal75(_, _, _) => todo!("TODO: unimplemented"),
        })
        .find(|&ord| ord != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}
