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
    use super::{filter_column_by_index, filter_columns};
    use crate::base::{database::Column, scalar::test_scalar::TestScalar};
    use bumpalo::Bump;

    type S = TestScalar;

    #[test]
    fn filter_columns_with_all_selected() {
        let alloc = Bump::new();
        let data = &[1i64, 2, 3];
        let cols: &[Column<S>] = &[Column::BigInt(data)];
        let selection = &[true, true, true];
        let (filtered, count) = filter_columns(&alloc, cols, selection);
        assert_eq!(count, 3);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn filter_columns_with_none_selected() {
        let alloc = Bump::new();
        let data = &[1i64, 2, 3];
        let cols: &[Column<S>] = &[Column::BigInt(data)];
        let selection = &[false, false, false];
        let (_, count) = filter_columns(&alloc, cols, selection);
        assert_eq!(count, 0);
    }

    #[test]
    fn filter_columns_selects_correct_rows() {
        let alloc = Bump::new();
        let data = &[10i64, 20, 30];
        let cols: &[Column<S>] = &[Column::BigInt(data)];
        let selection = &[false, true, false];
        let (filtered, count) = filter_columns(&alloc, cols, selection);
        assert_eq!(count, 1);
        if let Column::BigInt(vals) = &filtered[0] {
            assert_eq!(vals[0], 20);
        } else {
            panic!("expected BigInt column");
        }
    }

    #[test]
    fn filter_column_by_index_boolean_column() {
        let alloc = Bump::new();
        let data = &[true, false, true];
        let col = Column::<S>::Boolean(data);
        let filtered = filter_column_by_index(&alloc, &col, &[0, 2]);
        if let Column::Boolean(vals) = filtered {
            assert_eq!(vals, &[true, true]);
        } else {
            panic!("expected Boolean column");
        }
    }

    #[test]
    fn filter_column_by_index_empty_indexes() {
        let alloc = Bump::new();
        let data = &[1i64, 2, 3];
        let col = Column::<S>::BigInt(data);
        let filtered = filter_column_by_index(&alloc, &col, &[]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn filter_columns_empty_columns_list() {
        let alloc = Bump::new();
        let cols: &[Column<S>] = &[];
        let selection = &[true, false, true];
        let (filtered, count) = filter_columns(&alloc, cols, selection);
        assert_eq!(count, 2);
        assert!(filtered.is_empty());
    }
}
