use crate::base::{database::Column, scalar::Scalar};
use alloc::vec::Vec;
use bumpalo::Bump;

/// This function takes a selection vector and a set of columns and returns a
/// new set of columns that only contains the selected rows. The function
/// panics if the selection vector is a different length than the columns.
///
/// The function returns a tuple of the filtered columns and the number of
/// rows in the filtered columns.
/// # Panics
/// This function requires that `columns` and `selection` have the same length.
pub fn filter_columns<'a, S: Scalar>(
    alloc: &'a Bump,
    columns: &[Column<'a, S>],
    selection: &[bool],
) -> (Vec<Column<'a, S>>, usize) {
    for col in columns {
        assert_eq!(col.len(), selection.len());
    }
    let indexes: Vec<_> = selection
        .iter()
        .enumerate()
        .filter(|(_, &b)| b)
        .map(|(i, _)| i)
        .collect();
    let result_length = indexes.len();
    let filtered_result: Vec<_> = columns
        .iter()
        .map(|column| filter_column_by_index(alloc, column, &indexes))
        .collect();
    (filtered_result, result_length)
}
/// This function takes an index vector and a `Column` and returns a
/// new set of columns that only contains the selected indexes. It is assumed that
/// the indexes are valid.
pub fn filter_column_by_index<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &Column<'a, S>,
    indexes: &[usize],
) -> Column<'a, S> {
    match column {
        Column::Boolean(col) => {
            Column::Boolean(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Uint8(col) => {
            Column::Uint8(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::TinyInt(col) => {
            Column::TinyInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::SmallInt(col) => {
            Column::SmallInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Int(col) => {
            Column::Int(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::BigInt(col) => {
            Column::BigInt(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Int128(col) => {
            Column::Int128(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::VarChar((col, scals)) => Column::VarChar((
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| scals[i])),
        )),
        Column::VarBinary((col, scals)) => Column::VarBinary((
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| scals[i])),
        )),
        Column::Scalar(col) => {
            Column::Scalar(alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])))
        }
        Column::Decimal75(precision, scale, col) => Column::Decimal75(
            *precision,
            *scale,
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
        ),
        Column::TimestampTZ(tu, tz, col) => Column::TimestampTZ(
            *tu,
            *tz,
            alloc.alloc_slice_fill_iter(indexes.iter().map(|&i| col[i])),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
    };

    #[test]
    fn filter_columns_selects_rows_from_each_column() {
        let alloc = Bump::new();
        let bools = [true, false, true, false];
        let ints = [10_i32, 20, 30, 40];
        let text = ["a", "b", "c", "d"];
        let text_scalars = [
            TestScalar::from(text[0]),
            TestScalar::from(text[1]),
            TestScalar::from(text[2]),
            TestScalar::from(text[3]),
        ];
        let columns = [
            Column::Boolean(&bools),
            Column::Int(&ints),
            Column::VarChar((&text, &text_scalars)),
        ];

        let (filtered, result_length) =
            filter_columns(&alloc, &columns, &[false, true, false, true]);

        assert_eq!(result_length, 2);
        assert_eq!(filtered[0], Column::Boolean(&[false, false]));
        assert_eq!(filtered[1], Column::Int(&[20, 40]));
        assert_eq!(
            filtered[2],
            Column::VarChar((&["b", "d"], &[TestScalar::from("b"), TestScalar::from("d")]))
        );
    }

    #[test]
    fn filter_column_by_index_handles_numeric_and_time_columns() {
        let alloc = Bump::new();
        let indexes = [2, 0];

        assert_eq!(
            filter_column_by_index::<TestScalar>(&alloc, &Column::Uint8(&[1, 2, 3]), &indexes),
            Column::Uint8(&[3, 1])
        );
        assert_eq!(
            filter_column_by_index::<TestScalar>(&alloc, &Column::TinyInt(&[-1, 2, -3]), &indexes),
            Column::TinyInt(&[-3, -1])
        );
        assert_eq!(
            filter_column_by_index::<TestScalar>(
                &alloc,
                &Column::SmallInt(&[-10, 20, -30]),
                &indexes
            ),
            Column::SmallInt(&[-30, -10])
        );
        assert_eq!(
            filter_column_by_index::<TestScalar>(
                &alloc,
                &Column::BigInt(&[-100, 200, -300]),
                &indexes
            ),
            Column::BigInt(&[-300, -100])
        );
        assert_eq!(
            filter_column_by_index::<TestScalar>(
                &alloc,
                &Column::Int128(&[-1000, 2000, -3000]),
                &indexes
            ),
            Column::Int128(&[-3000, -1000])
        );

        let scalars = [
            TestScalar::from(11_u8),
            TestScalar::from(22_u8),
            TestScalar::from(33_u8),
        ];
        assert_eq!(
            filter_column_by_index::<TestScalar>(&alloc, &Column::Scalar(&scalars), &indexes),
            Column::Scalar(&[TestScalar::from(33_u8), TestScalar::from(11_u8)])
        );
        assert_eq!(
            filter_column_by_index::<TestScalar>(
                &alloc,
                &Column::Decimal75(Precision::new(9).unwrap(), 2, &scalars),
                &indexes
            ),
            Column::Decimal75(
                Precision::new(9).unwrap(),
                2,
                &[TestScalar::from(33_u8), TestScalar::from(11_u8)]
            )
        );
        assert_eq!(
            filter_column_by_index::<TestScalar>(
                &alloc,
                &Column::TimestampTZ(
                    PoSQLTimeUnit::Millisecond,
                    PoSQLTimeZone::utc(),
                    &[100_i64, 200, 300],
                ),
                &indexes
            ),
            Column::TimestampTZ(
                PoSQLTimeUnit::Millisecond,
                PoSQLTimeZone::utc(),
                &[300, 100]
            )
        );
    }

    #[test]
    fn filter_column_by_index_keeps_varbinary_values_and_scalars_together() {
        let alloc = Bump::new();
        let first = [1_u8, 2];
        let second = [3_u8];
        let third = [5_u8, 8, 13];
        let bytes: [&[u8]; 3] = [&first, &second, &third];
        let scalars = [
            TestScalar::from_byte_slice_via_hash(&first),
            TestScalar::from_byte_slice_via_hash(&second),
            TestScalar::from_byte_slice_via_hash(&third),
        ];

        let result = filter_column_by_index::<TestScalar>(
            &alloc,
            &Column::VarBinary((&bytes, &scalars)),
            &[2, 0],
        );

        assert_eq!(
            result,
            Column::VarBinary((
                &[third.as_slice(), first.as_slice()],
                &[
                    TestScalar::from_byte_slice_via_hash(&third),
                    TestScalar::from_byte_slice_via_hash(&first),
                ]
            ))
        );
    }
}
