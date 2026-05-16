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
    use super::{compare_indexes_by_columns, compare_single_row_of_tables};
    use crate::base::{
        database::{Column, ColumnType, TableOperationError},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, Scalar},
    };
    use core::cmp::Ordering;

    fn assert_single_row_order(
        left: Column<'_, TestScalar>,
        right: Column<'_, TestScalar>,
        expected: Ordering,
    ) {
        assert_eq!(
            compare_single_row_of_tables(&[left], &[right], 0, 0).unwrap(),
            expected
        );
    }

    fn assert_index_order(column: Column<'_, TestScalar>, expected: Ordering) {
        assert_eq!(compare_indexes_by_columns(&[column], 0, 1), expected);
    }

    #[test]
    fn compare_indexes_uses_lexicographic_column_order() {
        let ids = [1_i64, 2, 1];
        let names = ["b", "a", "a"];
        let name_scalars = [TestScalar::ZERO; 3];
        let order_by = [
            Column::<TestScalar>::BigInt(&ids),
            Column::VarChar((&names, &name_scalars)),
        ];

        assert_eq!(compare_indexes_by_columns(&order_by, 0, 1), Ordering::Less);
        assert_eq!(
            compare_indexes_by_columns(&order_by, 0, 2),
            Ordering::Greater
        );
        assert_eq!(compare_indexes_by_columns(&order_by, 2, 2), Ordering::Equal);

        let raw_bytes = [b"aa".as_slice(), b"ab".as_slice()];
        let byte_scalars = [TestScalar::ZERO; 2];
        let order_by = [Column::VarBinary((&raw_bytes, &byte_scalars))];
        assert_eq!(compare_indexes_by_columns(&order_by, 0, 1), Ordering::Less);
    }

    #[test]
    fn compare_indexes_handles_column_variants() {
        let bools = [false, true];
        assert_index_order(Column::Boolean(&bools), Ordering::Less);

        let uint8s = [9_u8, 4];
        assert_index_order(Column::Uint8(&uint8s), Ordering::Greater);

        let tinyints = [-8_i8, 8];
        assert_index_order(Column::TinyInt(&tinyints), Ordering::Less);

        let smallints = [7_i16, 7];
        assert_index_order(Column::SmallInt(&smallints), Ordering::Equal);

        let ints = [20_i32, 10];
        assert_index_order(Column::Int(&ints), Ordering::Greater);

        let timestamps = [100_i64, 200];
        assert_index_order(
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps),
            Ordering::Less,
        );

        let decimal_precision = Precision::new(10).unwrap();
        let decimals = [TestScalar::from(3), TestScalar::from(8)];
        assert_index_order(
            Column::Decimal75(decimal_precision, 0, &decimals),
            Ordering::Less,
        );
    }

    #[test]
    fn compare_single_row_handles_matching_column_types() {
        let bool_left = [false];
        let bool_right = [true];
        assert_single_row_order(
            Column::Boolean(&bool_left),
            Column::Boolean(&bool_right),
            Ordering::Less,
        );

        let uint8_left = [3_u8];
        let uint8_right = [2_u8];
        assert_single_row_order(
            Column::Uint8(&uint8_left),
            Column::Uint8(&uint8_right),
            Ordering::Greater,
        );

        let tiny_left = [-2_i8];
        let tiny_right = [2_i8];
        assert_single_row_order(
            Column::TinyInt(&tiny_left),
            Column::TinyInt(&tiny_right),
            Ordering::Less,
        );

        let small_left = [7_i16];
        let small_right = [7_i16];
        assert_single_row_order(
            Column::SmallInt(&small_left),
            Column::SmallInt(&small_right),
            Ordering::Equal,
        );

        let int_left = [9_i32];
        let int_right = [3_i32];
        assert_single_row_order(
            Column::Int(&int_left),
            Column::Int(&int_right),
            Ordering::Greater,
        );

        let bigint_left = [11_i64];
        let bigint_right = [12_i64];
        assert_single_row_order(
            Column::BigInt(&bigint_left),
            Column::BigInt(&bigint_right),
            Ordering::Less,
        );

        let int128_left = [10_i128];
        let int128_right = [10_i128];
        assert_single_row_order(
            Column::Int128(&int128_left),
            Column::Int128(&int128_right),
            Ordering::Equal,
        );

        let decimal_precision = Precision::new(10).unwrap();
        let decimal_left = [TestScalar::from(8)];
        let decimal_right = [TestScalar::from(4)];
        assert_single_row_order(
            Column::Decimal75(decimal_precision, 0, &decimal_left),
            Column::Decimal75(decimal_precision, 0, &decimal_right),
            Ordering::Greater,
        );

        let scalar_left = [TestScalar::from(1)];
        let scalar_right = [TestScalar::from(2)];
        assert_single_row_order(
            Column::Scalar(&scalar_left),
            Column::Scalar(&scalar_right),
            Ordering::Less,
        );

        let text_left = ["same"];
        let text_right = ["same"];
        let text_scalars = [TestScalar::ZERO];
        assert_single_row_order(
            Column::VarChar((&text_left, &text_scalars)),
            Column::VarChar((&text_right, &text_scalars)),
            Ordering::Equal,
        );

        let ts_left = [100_i64];
        let ts_right = [50_i64];
        assert_single_row_order(
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &ts_left),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &ts_right),
            Ordering::Greater,
        );
    }

    #[test]
    fn compare_single_row_rejects_mismatched_column_types() {
        let left = [1_i64];
        let right = [1_i32];
        assert!(matches!(
            compare_single_row_of_tables(
                &[Column::<TestScalar>::BigInt(&left)],
                &[Column::Int(&right)],
                0,
                0,
            ),
            Err(TableOperationError::JoinIncompatibleTypes {
                left_type: ColumnType::BigInt,
                right_type: ColumnType::Int,
            })
        ));
    }
}
