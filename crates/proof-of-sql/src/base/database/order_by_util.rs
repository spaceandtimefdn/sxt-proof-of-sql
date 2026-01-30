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
    use crate::base::{
        database::Column,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };
    use core::cmp::Ordering;

    #[test]
    fn we_can_compare_indexes_by_uint8_column() {
        let data: &[u8] = &[10, 20, 30, 20];
        let column: Column<TestScalar> = Column::Uint8(data);
        let columns = vec![column];

        assert_eq!(compare_indexes_by_columns(&columns, 1, 3), Ordering::Equal);

        assert_eq!(compare_indexes_by_columns(&columns, 0, 1), Ordering::Less);

        assert_eq!(
            compare_indexes_by_columns(&columns, 2, 1),
            Ordering::Greater
        );
    }

    #[test]
    fn we_can_compare_indexes_by_tinyint_column() {
        let data: &[i8] = &[-10, 0, 20, 0];
        let column: Column<TestScalar> = Column::TinyInt(data);
        let columns = vec![column];

        assert_eq!(compare_indexes_by_columns(&columns, 1, 3), Ordering::Equal);

        assert_eq!(compare_indexes_by_columns(&columns, 0, 1), Ordering::Less);

        assert_eq!(
            compare_indexes_by_columns(&columns, 2, 1),
            Ordering::Greater
        );
    }

    #[test]
    fn we_can_compare_single_row_with_uint8_columns() {
        let left_data: &[u8] = &[10, 20, 30];
        let right_data: &[u8] = &[15, 20, 25];
        let left_column: Column<TestScalar> = Column::Uint8(left_data);
        let right_column: Column<TestScalar> = Column::Uint8(right_data);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 1, 1).unwrap();
        assert_eq!(result, Ordering::Equal);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 0, 0).unwrap();
        assert_eq!(result, Ordering::Less);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 2, 2).unwrap();
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn we_can_compare_single_row_with_tinyint_columns() {
        let left_data: &[i8] = &[-10, 0, 20];
        let right_data: &[i8] = &[-5, 0, 15];
        let left_column: Column<TestScalar> = Column::TinyInt(left_data);
        let right_column: Column<TestScalar> = Column::TinyInt(right_data);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 1, 1).unwrap();
        assert_eq!(result, Ordering::Equal);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 0, 0).unwrap();
        assert_eq!(result, Ordering::Less);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 2, 2).unwrap();
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn we_can_compare_single_row_with_timestamptz_columns() {
        let left_data: &[i64] = &[100, 200, 300];
        let right_data: &[i64] = &[150, 200, 250];
        let left_column: Column<TestScalar> =
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), left_data);
        let right_column: Column<TestScalar> =
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), right_data);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 1, 1).unwrap();
        assert_eq!(result, Ordering::Equal);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 0, 0).unwrap();
        assert_eq!(result, Ordering::Less);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 2, 2).unwrap();
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn we_can_compare_single_row_with_multiple_columns() {
        let left_int_data: &[i32] = &[1, 1, 2];
        let left_varchar_data: &[&str] = &["a", "b", "c"];
        let right_int_data: &[i32] = &[1, 1, 2];
        let right_varchar_data: &[&str] = &["a", "c", "b"];

        let left_columns: Vec<Column<TestScalar>> = vec![
            Column::Int(left_int_data),
            Column::VarChar((left_varchar_data, &[])),
        ];
        let right_columns: Vec<Column<TestScalar>> = vec![
            Column::Int(right_int_data),
            Column::VarChar((right_varchar_data, &[])),
        ];

        // test equal (both columns match)
        let result = compare_single_row_of_tables(&left_columns, &right_columns, 0, 0).unwrap();
        assert_eq!(result, Ordering::Equal);

        // test less than (first column equal, second column less)
        let result = compare_single_row_of_tables(&left_columns, &right_columns, 1, 1).unwrap();
        assert_eq!(result, Ordering::Less);

        // test greater than (first column equal, second column greater)
        let result = compare_single_row_of_tables(&left_columns, &right_columns, 2, 2).unwrap();
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn we_can_detect_incompatible_column_types() {
        let left_data: &[i32] = &[1, 2, 3];
        let right_data: &[i64] = &[1, 2, 3];

        let left_columns: Vec<Column<TestScalar>> = vec![Column::Int(left_data)];
        let right_columns: Vec<Column<TestScalar>> = vec![Column::BigInt(right_data)];

        let result = compare_single_row_of_tables(&left_columns, &right_columns, 0, 0);

        assert!(matches!(
            result,
            Err(TableOperationError::JoinIncompatibleTypes { .. })
        ));
    }

    #[test]
    fn we_can_compare_indexes_with_multiple_columns() {
        let int_data: &[i32] = &[1, 1, 2, 2];
        let bool_data: &[bool] = &[true, false, true, false];

        let columns: Vec<Column<TestScalar>> =
            vec![Column::Int(int_data), Column::Boolean(bool_data)];

        assert_eq!(compare_indexes_by_columns(&columns, 0, 0), Ordering::Equal);

        assert_eq!(compare_indexes_by_columns(&columns, 1, 0), Ordering::Less);

        assert_eq!(
            compare_indexes_by_columns(&columns, 2, 1),
            Ordering::Greater
        );
    }

    #[test]
    fn we_can_compare_decimal75_columns() {
        let left_data = [
            TestScalar::from(100),
            TestScalar::from(200),
            TestScalar::from(300),
        ];
        let right_data = [
            TestScalar::from(150),
            TestScalar::from(200),
            TestScalar::from(250),
        ];

        let left_column: Column<TestScalar> =
            Column::Decimal75(Precision::new(10).unwrap(), 2, &left_data);
        let right_column: Column<TestScalar> =
            Column::Decimal75(Precision::new(10).unwrap(), 2, &right_data);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 1, 1).unwrap();
        assert_eq!(result, Ordering::Equal);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 0, 0).unwrap();
        assert_eq!(result, Ordering::Less);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 2, 2).unwrap();
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn we_can_compare_scalar_columns() {
        let left_data = [
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];
        let right_data = [
            TestScalar::from(15),
            TestScalar::from(20),
            TestScalar::from(25),
        ];

        let left_column: Column<TestScalar> = Column::Scalar(&left_data);
        let right_column: Column<TestScalar> = Column::Scalar(&right_data);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 1, 1).unwrap();
        assert_eq!(result, Ordering::Equal);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 0, 0).unwrap();
        assert_eq!(result, Ordering::Less);

        let result = compare_single_row_of_tables(&[left_column], &[right_column], 2, 2).unwrap();
        assert_eq!(result, Ordering::Greater);
    }
}
