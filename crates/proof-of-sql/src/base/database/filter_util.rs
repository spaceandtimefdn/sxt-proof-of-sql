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
    use crate::base::database::Column;
    use crate::base::scalar::test_scalar::TestScalar;
    use bumpalo::Bump;

    #[test]
    fn filter_columns_with_empty_selection_returns_empty() {
        let alloc = Bump::new();
        let data = [1i64, 2, 3];
        let cols = [Column::<TestScalar>::BigInt(&data)];
        let selection = [false, false, false];
        let (result, count) = filter_columns(&alloc, &cols, &selection);
        assert_eq!(count, 0);
        assert_eq!(result.len(), 1);
        if let Column::BigInt(r) = &result[0] { assert_eq!(r.len(), 0); }
    }

    #[test]
    fn filter_columns_with_all_selected_returns_all() {
        let alloc = Bump::new();
        let data = [10i64, 20, 30];
        let cols = [Column::<TestScalar>::BigInt(&data)];
        let selection = [true, true, true];
        let (result, count) = filter_columns(&alloc, &cols, &selection);
        assert_eq!(count, 3);
        if let Column::BigInt(r) = &result[0] { assert_eq!(*r, [10, 20, 30]); }
    }

    #[test]
    fn filter_columns_selects_specific_rows() {
        let alloc = Bump::new();
        let data = [1i64, 2, 3, 4, 5];
        let cols = [Column::<TestScalar>::BigInt(&data)];
        let selection = [true, false, true, false, true];
        let (result, count) = filter_columns(&alloc, &cols, &selection);
        assert_eq!(count, 3);
        if let Column::BigInt(r) = &result[0] { assert_eq!(*r, [1, 3, 5]); }
    }

    #[test]
    fn filter_columns_multiple_columns() {
        let alloc = Bump::new();
        let bools = [true, false, true];
        let ints = [10i64, 20, 30];
        let cols = [
            Column::<TestScalar>::Boolean(&bools),
            Column::<TestScalar>::BigInt(&ints),
        ];
        let selection = [true, false, true];
        let (result, count) = filter_columns(&alloc, &cols, &selection);
        assert_eq!(count, 2);
        assert_eq!(result.len(), 2);
        if let Column::Boolean(r) = &result[0] { assert_eq!(*r, [true, true]); }
        if let Column::BigInt(r) = &result[1] { assert_eq!(*r, [10, 30]); }
    }

    #[test]
    fn filter_column_by_index_bool() {
        let alloc = Bump::new();
        let data = [false, true, false, true];
        let col = Column::<TestScalar>::Boolean(&data);
        let indexes = [1usize, 3];
        let result = filter_column_by_index(&alloc, &col, &indexes);
        if let Column::Boolean(r) = result { assert_eq!(*r, [true, true]); }
    }

    #[test]
    fn filter_column_by_index_bigint() {
        let alloc = Bump::new();
        let data = [100i64, 200, 300, 400];
        let col = Column::<TestScalar>::BigInt(&data);
        let indexes = [0usize, 2, 3];
        let result = filter_column_by_index(&alloc, &col, &indexes);
        if let Column::BigInt(r) = result { assert_eq!(*r, [100, 300, 400]); }
    }

    #[test]
    fn filter_column_by_index_empty_indexes() {
        let alloc = Bump::new();
        let data = [1i64, 2, 3];
        let col = Column::<TestScalar>::BigInt(&data);
        let result = filter_column_by_index(&alloc, &col, &[]);
        if let Column::BigInt(r) = result { assert_eq!(r.len(), 0); }
    }

    #[test]
    fn filter_column_by_index_uint8() {
        let alloc = Bump::new();
        let data = [5u8, 10, 15, 20];
        let col = Column::<TestScalar>::Uint8(&data);
        let indexes = [1usize, 3];
        let result = filter_column_by_index(&alloc, &col, &indexes);
        if let Column::Uint8(r) = result { assert_eq!(*r, [10, 20]); }
    }

    #[test]
    fn filter_column_by_index_int128() {
        let alloc = Bump::new();
        let data = [1i128, -2, 3];
        let col = Column::<TestScalar>::Int128(&data);
        let indexes = [0usize, 2];
        let result = filter_column_by_index(&alloc, &col, &indexes);
        if let Column::Int128(r) = result { assert_eq!(*r, [1, 3]); }
    }
}
