//! Contains the utility functions for ordering.
use crate::base::{
    database::{Column, TableOperationError, TableOperationResult},
    scalar::{Scalar, ScalarExt},
};
use core::cmp::Ordering;

/// Compares the tuples `(order_by[0][i], order_by[1][i], ...)` and
/// `(order_by[0][j], order_by[1][j], ...)` in lexicographic order.
pub(crate) fn compare_indexes_by_columns<S: Scalar>(
    order_by: &[Column<S>],
    i: usize,
    j: usize,
) -> Ordering {
    order_by
        .iter()
        .map(|col| match col {
            Column::Boolean(col) => col[i].cmp(&col[j]),
            Column::Uint8(col) => col[i].cmp(&col[j]),
            Column::TinyInt(col) => col[i].cmp(&col[j]),
            Column::SmallInt(col) => col[i].cmp(&col[j]),
            Column::Int(col) => col[i].cmp(&col[j]),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col[i].cmp(&col[j]),
            Column::Int128(col) => col[i].cmp(&col[j]),
            Column::Decimal75(_, _, col) => col[i].signed_cmp(&col[j]),
            Column::Scalar(col) => col[i].cmp(&col[j]),
            Column::VarChar((col, _)) => col[i].cmp(col[j]),
            Column::VarBinary((col, _)) => col[i].cmp(col[j]),
        })
        .find(|&ord| ord != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

/// Compares the tuples `(left[0][i], left[1][i], ...)` and
/// `(right[0][j], right[1][j], ...)` in lexicographic order.
///
/// Requires that columns in `left` and `right` have the same column types for now
///
/// # Panics
/// Panics if `left` and `right` have different number of columns
/// which should never happen since this function should only be called
/// for joins.
pub(crate) fn compare_single_row_of_tables<S: Scalar>(
    left: &[Column<S>],
    right: &[Column<S>],
    left_row_index: usize,
    right_row_index: usize,
) -> TableOperationResult<Ordering> {
    // Should never happen
    assert_eq!(left.len(), right.len());
    for (left_col, right_col) in left.iter().zip(right.iter()) {
        if left_col.column_type() != right_col.column_type() {
            return Err(TableOperationError::JoinIncompatibleTypes {
                left_type: left_col.column_type(),
                right_type: right_col.column_type(),
            });
        }

        let ordering = match (left_col, right_col) {
            (Column::Boolean(left_col), Column::Boolean(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::Uint8(left_col), Column::Uint8(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::TinyInt(left_col), Column::TinyInt(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::SmallInt(left_col), Column::SmallInt(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::Int(left_col), Column::Int(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::BigInt(left_col), Column::BigInt(right_col))
            | (Column::TimestampTZ(_, _, left_col), Column::TimestampTZ(_, _, right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::Int128(left_col), Column::Int128(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::Decimal75(_, _, left_col), Column::Decimal75(_, _, right_col)) => {
                left_col[left_row_index].signed_cmp(&right_col[right_row_index])
            }
            (Column::Scalar(left_col), Column::Scalar(right_col)) => {
                left_col[left_row_index].cmp(&right_col[right_row_index])
            }
            (Column::VarChar((left_col, _)), Column::VarChar((right_col, _))) => {
                left_col[left_row_index].cmp(right_col[right_row_index])
            }
            // Should never happen since we checked the column types
            _ => unreachable!(),
        };

        if ordering != Ordering::Equal {
            return Ok(ordering);
        }
    }

    Ok(Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn compare_bigint_equal() {
        let col = Column::<TestScalar>::BigInt(&[10, 20, 30]);
        assert_eq!(
            compare_indexes_by_columns(&[col], 0, 0),
            Ordering::Equal
        );
    }

    #[test]
    fn compare_bigint_less() {
        let col = Column::<TestScalar>::BigInt(&[10, 20, 30]);
        assert_eq!(
            compare_indexes_by_columns(&[col], 0, 1),
            Ordering::Less
        );
    }

    #[test]
    fn compare_bigint_greater() {
        let col = Column::<TestScalar>::BigInt(&[10, 20, 30]);
        assert_eq!(
            compare_indexes_by_columns(&[col], 1, 0),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_boolean() {
        let col = Column::<TestScalar>::Boolean(&[false, true]);
        assert_eq!(
            compare_indexes_by_columns(&[col], 0, 1),
            Ordering::Less
        );
        assert_eq!(
            compare_indexes_by_columns(&[col], 1, 0),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_int() {
        let col = Column::<TestScalar>::Int(&[5, 3, 8]);
        assert_eq!(
            compare_indexes_by_columns(&[col], 0, 1),
            Ordering::Greater
        );
        assert_eq!(
            compare_indexes_by_columns(&[col], 1, 0),
            Ordering::Less
        );
    }

    #[test]
    fn compare_varchar() {
        let scalars: Vec<TestScalar> = vec![];
        let col = Column::<TestScalar>::VarChar((&["apple", "banana"], &scalars));
        assert_eq!(
            compare_indexes_by_columns(&[col], 0, 1),
            Ordering::Less
        );
    }

    #[test]
    fn compare_multi_column_first_determines() {
        let col1 = Column::<TestScalar>::BigInt(&[10, 20, 10]);
        let col2 = Column::<TestScalar>::BigInt(&[100, 200, 200]);
        // First column differs: 10 vs 20 -> Less
        assert_eq!(
            compare_indexes_by_columns(&[col1, col2], 0, 1),
            Ordering::Less
        );
    }

    #[test]
    fn compare_multi_column_second_tiebreaks() {
        let col1 = Column::<TestScalar>::BigInt(&[10, 10, 20]);
        let col2 = Column::<TestScalar>::BigInt(&[100, 200, 300]);
        // First column equal (10==10), second differs: 100 vs 200 -> Less
        assert_eq!(
            compare_indexes_by_columns(&[col1, col2], 0, 1),
            Ordering::Less
        );
    }

    #[test]
    fn compare_multi_column_all_equal() {
        let col1 = Column::<TestScalar>::BigInt(&[10, 10]);
        let col2 = Column::<TestScalar>::BigInt(&[100, 100]);
        assert_eq!(
            compare_indexes_by_columns(&[col1, col2], 0, 1),
            Ordering::Equal
        );
    }

    #[test]
    fn compare_empty_columns_is_equal() {
        let cols: Vec<Column<TestScalar>> = vec![];
        assert_eq!(
            compare_indexes_by_columns(&cols, 0, 0),
            Ordering::Equal
        );
    }

    #[test]
    fn compare_single_row_of_tables_bigint() {
        let left = [Column::<TestScalar>::BigInt(&[10, 20])];
        let right = [Column::<TestScalar>::BigInt(&[10, 30])];
        assert_eq!(
            compare_single_row_of_tables(&left, &right, 0, 0).unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            compare_single_row_of_tables(&left, &right, 0, 1).unwrap(),
            Ordering::Less
        );
        assert_eq!(
            compare_single_row_of_tables(&left, &right, 1, 0).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_single_row_of_tables_boolean() {
        let left = [Column::<TestScalar>::Boolean(&[false, true])];
        let right = [Column::<TestScalar>::Boolean(&[true, false])];
        assert_eq!(
            compare_single_row_of_tables(&left, &right, 0, 0).unwrap(),
            Ordering::Less
        );
        assert_eq!(
            compare_single_row_of_tables(&left, &right, 1, 1).unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_single_row_of_tables_incompatible_types() {
        let left = [Column::<TestScalar>::BigInt(&[10])];
        let right = [Column::<TestScalar>::Boolean(&[true])];
        assert!(compare_single_row_of_tables(&left, &right, 0, 0).is_err());
    }
}
