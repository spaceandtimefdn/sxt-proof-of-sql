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
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn filter_bigint_by_selection() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::BigInt(&[10, 20, 30, 40, 50]);
        let selection = [true, false, true, false, true];
        let (filtered, len) = filter_columns(&alloc, &[column], &selection);
        assert_eq!(len, 3);
        match filtered[0] {
            Column::BigInt(col) => assert_eq!(col, &[10, 30, 50]),
            _ => panic!("Expected BigInt"),
        }
    }

    #[test]
    fn filter_boolean_by_selection() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::Boolean(&[true, false, true, false]);
        let selection = [true, true, false, false];
        let (filtered, len) = filter_columns(&alloc, &[column], &selection);
        assert_eq!(len, 2);
        match filtered[0] {
            Column::Boolean(col) => assert_eq!(col, &[true, false]),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn filter_multiple_columns() {
        let alloc = Bump::new();
        let col1 = Column::<TestScalar>::BigInt(&[1, 2, 3]);
        let col2 = Column::<TestScalar>::Boolean(&[true, false, true]);
        let selection = [false, true, true];
        let (filtered, len) = filter_columns(&alloc, &[col1, col2], &selection);
        assert_eq!(len, 2);
        match filtered[0] {
            Column::BigInt(col) => assert_eq!(col, &[2, 3]),
            _ => panic!("Expected BigInt"),
        }
        match filtered[1] {
            Column::Boolean(col) => assert_eq!(col, &[false, true]),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn filter_all_selected() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::BigInt(&[1, 2, 3]);
        let selection = [true, true, true];
        let (filtered, len) = filter_columns(&alloc, &[column], &selection);
        assert_eq!(len, 3);
        match filtered[0] {
            Column::BigInt(col) => assert_eq!(col, &[1, 2, 3]),
            _ => panic!("Expected BigInt"),
        }
    }

    #[test]
    fn filter_none_selected() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::BigInt(&[1, 2, 3]);
        let selection = [false, false, false];
        let (filtered, len) = filter_columns(&alloc, &[column], &selection);
        assert_eq!(len, 0);
        match filtered[0] {
            Column::BigInt(col) => assert!(col.is_empty()),
            _ => panic!("Expected BigInt"),
        }
    }

    #[test]
    fn filter_column_by_index_bigint() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::BigInt(&[10, 20, 30, 40]);
        let indexes = [0, 2, 3];
        let result = filter_column_by_index(&alloc, &column, &indexes);
        match result {
            Column::BigInt(col) => assert_eq!(col, &[10, 30, 40]),
            _ => panic!("Expected BigInt"),
        }
    }

    #[test]
    fn filter_column_by_index_int() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::Int(&[1, 2, 3]);
        let indexes = [1];
        let result = filter_column_by_index(&alloc, &column, &indexes);
        match result {
            Column::Int(col) => assert_eq!(col, &[2]),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn filter_column_by_index_boolean() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::Boolean(&[true, false, true]);
        let indexes = [0, 2];
        let result = filter_column_by_index(&alloc, &column, &indexes);
        match result {
            Column::Boolean(col) => assert_eq!(col, &[true, true]),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn filter_column_by_index_repeated() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::BigInt(&[10, 20, 30]);
        let indexes = [1, 1, 1];
        let result = filter_column_by_index(&alloc, &column, &indexes);
        match result {
            Column::BigInt(col) => assert_eq!(col, &[20, 20, 20]),
            _ => panic!("Expected BigInt"),
        }
    }

    #[test]
    fn filter_column_by_index_empty() {
        let alloc = Bump::new();
        let column = Column::<TestScalar>::BigInt(&[10, 20]);
        let indexes: &[usize] = &[];
        let result = filter_column_by_index(&alloc, &column, indexes);
        match result {
            Column::BigInt(col) => assert!(col.is_empty()),
            _ => panic!("Expected BigInt"),
        }
    }
}
