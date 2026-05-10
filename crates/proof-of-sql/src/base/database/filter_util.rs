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
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn we_can_filter_remaining_column_variants_by_index() {
        let alloc = Bump::new();
        let indexes = [3, 0, 2];

        let column = Column::<TestScalar>::Boolean(&[true, false, true, false]);
        assert_eq!(
            filter_column_by_index(&alloc, &column, &indexes),
            Column::Boolean(&[false, true, true])
        );

        let column = Column::<TestScalar>::Uint8(&[10_u8, 20, 30, 40]);
        assert_eq!(
            filter_column_by_index(&alloc, &column, &indexes),
            Column::Uint8(&[40_u8, 10, 30])
        );

        let column = Column::<TestScalar>::TinyInt(&[-1_i8, 2, -3, 4]);
        assert_eq!(
            filter_column_by_index(&alloc, &column, &indexes),
            Column::TinyInt(&[4_i8, -1, -3])
        );

        let column = Column::<TestScalar>::SmallInt(&[-10_i16, 20, -30, 40]);
        assert_eq!(
            filter_column_by_index(&alloc, &column, &indexes),
            Column::SmallInt(&[40_i16, -10, -30])
        );

        let column = Column::<TestScalar>::Int(&[-100_i32, 200, -300, 400]);
        assert_eq!(
            filter_column_by_index(&alloc, &column, &indexes),
            Column::Int(&[400_i32, -100, -300])
        );

        let timezone = PoSQLTimeZone::utc();
        let column = Column::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            timezone,
            &[1_000, 2_000, 3_000, 4_000],
        );
        assert_eq!(
            filter_column_by_index(&alloc, &column, &indexes),
            Column::TimestampTZ(PoSQLTimeUnit::Second, timezone, &[4_000, 1_000, 3_000])
        );
    }
}
