use super::join_util::*;
use crate::base::{
    database::{Column, Table, TableOperationError, TableOptions},
    map::IndexMap,
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;

// cross_join
#[test]
fn we_can_cross_join_empty_tables() {
    let alloc = Bump::new();
    let left = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("a".into(), Column::Int(&[]))]),
        TableOptions::new(Some(0)),
    )
    .unwrap();
    let right = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("b".into(), Column::Int(&[]))]),
        TableOptions::new(Some(0)),
    )
    .unwrap();
    let result = cross_join(&left, &right, &alloc);
    assert_eq!(result.num_rows(), 0);
}

#[test]
fn we_can_cross_join_single_row_tables() {
    let alloc = Bump::new();
    let left = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("a".into(), Column::Int(&[1]))]),
        TableOptions::new(Some(1)),
    )
    .unwrap();
    let right = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("b".into(), Column::Int(&[2]))]),
        TableOptions::new(Some(1)),
    )
    .unwrap();
    let result = cross_join(&left, &right, &alloc);
    assert_eq!(result.num_rows(), 1);
    assert_eq!(result.column(0).unwrap().as_int().unwrap(), &[1]);
    assert_eq!(result.column(1).unwrap().as_int().unwrap(), &[2]);
}

#[test]
fn we_can_cross_join_multi_row_tables() {
    let alloc = Bump::new();
    let left = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("a".into(), Column::Int(&[1, 2]))]),
        TableOptions::new(Some(2)),
    )
    .unwrap();
    let right = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("b".into(), Column::Int(&[10, 20, 30]))]),
        TableOptions::new(Some(3)),
    )
    .unwrap();
    let result = cross_join(&left, &right, &alloc);
    assert_eq!(result.num_rows(), 6);
    // Left column repeated: [1, 2, 1, 2, 1, 2]
    assert_eq!(
        result.column(0).unwrap().as_int().unwrap(),
        &[1, 2, 1, 2, 1, 2]
    );
    // Right column element-wise repeated: [10, 10, 20, 20, 30, 30]
    assert_eq!(
        result.column(1).unwrap().as_int().unwrap(),
        &[10, 10, 20, 20, 30, 30]
    );
}

#[test]
fn we_can_cross_join_tables_without_columns() {
    let alloc = Bump::new();
    let left = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::default(),
        TableOptions::new(Some(3)),
    )
    .unwrap();
    let right = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::default(),
        TableOptions::new(Some(4)),
    )
    .unwrap();
    let result = cross_join(&left, &right, &alloc);
    assert_eq!(result.num_rows(), 12);
}

// get_columns_of_table
#[test]
fn we_can_get_columns_of_table_by_indexes() {
    let table = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![
            ("a".into(), Column::Int(&[1, 2])),
            ("b".into(), Column::BigInt(&[10, 20])),
            ("c".into(), Column::Boolean(&[true, false])),
        ]),
        TableOptions::new(Some(2)),
    )
    .unwrap();
    let columns = get_columns_of_table(&table, &[0, 2]).unwrap();
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].as_int().unwrap(), &[1, 2]);
    assert_eq!(columns[1].as_boolean().unwrap(), &[true, false]);
}

#[test]
fn we_can_get_empty_columns_from_table() {
    let table = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("a".into(), Column::Int(&[1]))]),
        TableOptions::new(Some(1)),
    )
    .unwrap();
    let columns = get_columns_of_table(&table, &[]).unwrap();
    assert!(columns.is_empty());
}

#[test]
fn we_cannot_get_out_of_bounds_column_from_table() {
    let table = Table::<'_, TestScalar>::try_new_with_options(
        IndexMap::from_iter(vec![("a".into(), Column::Int(&[1]))]),
        TableOptions::new(Some(1)),
    )
    .unwrap();
    let result = get_columns_of_table(&table, &[5]);
    assert!(matches!(
        result,
        Err(TableOperationError::ColumnIndexOutOfBounds { .. })
    ));
}

// ordered_set_union
#[test]
fn ordered_set_union_with_empty_columns_returns_empty() {
    let alloc = Bump::new();
    let result = ordered_set_union::<TestScalar>(&[], &[], &alloc).unwrap();
    assert!(result.is_empty());
}

#[test]
fn ordered_set_union_deduplicates_and_sorts() {
    let alloc = Bump::new();
    let left: Vec<Column<TestScalar>> = vec![Column::Int(&[3, 1])];
    let right: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2])];
    let result = ordered_set_union(&left, &right, &alloc).unwrap();
    assert_eq!(result.len(), 1);
    // The union of [3, 1] and [1, 2] = [3, 1, 1, 2], deduped sorted = [1, 2, 3]
    assert_eq!(result[0].as_int().unwrap(), &[1, 2, 3]);
}

#[test]
fn ordered_set_union_with_no_overlap() {
    let alloc = Bump::new();
    let left: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 3])];
    let right: Vec<Column<TestScalar>> = vec![Column::Int(&[2, 4])];
    let result = ordered_set_union(&left, &right, &alloc).unwrap();
    assert_eq!(result[0].as_int().unwrap(), &[1, 2, 3, 4]);
}

#[test]
fn ordered_set_union_with_full_overlap() {
    let alloc = Bump::new();
    let left: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 3])];
    let right: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 3])];
    let result = ordered_set_union(&left, &right, &alloc).unwrap();
    assert_eq!(result[0].as_int().unwrap(), &[1, 2, 3]);
}

// get_multiplicities
#[test]
fn get_multiplicities_empty_unique_returns_empty() {
    let alloc = Bump::new();
    let data: Vec<Column<TestScalar>> = vec![];
    let unique: Vec<Column<TestScalar>> = vec![];
    let result = get_multiplicities(&data, &unique, &alloc);
    assert!(result.is_empty());
}

#[test]
fn get_multiplicities_empty_data_returns_zeros() {
    let alloc = Bump::new();
    let data: Vec<Column<TestScalar>> = vec![];
    let unique: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 3])];
    let result = get_multiplicities(&data, &unique, &alloc);
    assert_eq!(result, &[0, 0, 0]);
}

#[test]
fn get_multiplicities_counts_correctly() {
    let alloc = Bump::new();
    // data has [1, 2, 2, 3, 3, 3], unique has [1, 2, 3]
    let data: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 2, 3, 3, 3])];
    let unique: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 3])];
    let result = get_multiplicities(&data, &unique, &alloc);
    assert_eq!(result, &[1, 2, 3]);
}

#[test]
fn get_multiplicities_with_missing_data() {
    let alloc = Bump::new();
    // data has [2, 2], unique has [1, 2, 3]
    let data: Vec<Column<TestScalar>> = vec![Column::Int(&[2, 2])];
    let unique: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 3])];
    let result = get_multiplicities(&data, &unique, &alloc);
    assert_eq!(result, &[0, 2, 0]);
}

// get_sort_merge_join_indexes
#[test]
fn sort_merge_join_with_empty_tables_returns_empty() {
    let left_on: Vec<Column<TestScalar>> = vec![Column::Int(&[])];
    let right_on: Vec<Column<TestScalar>> = vec![Column::Int(&[])];
    let result = get_sort_merge_join_indexes(&left_on, &right_on, 0, 0);
    assert!(result.is_empty());
}

#[test]
fn sort_merge_join_with_matching_single_rows() {
    let left_on: Vec<Column<TestScalar>> = vec![Column::Int(&[1])];
    let right_on: Vec<Column<TestScalar>> = vec![Column::Int(&[1])];
    let result = get_sort_merge_join_indexes(&left_on, &right_on, 1, 1);
    assert_eq!(result, vec![(0, 0)]);
}

#[test]
fn sort_merge_join_with_no_matches() {
    let left_on: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 3])];
    let right_on: Vec<Column<TestScalar>> = vec![Column::Int(&[2, 4])];
    let result = get_sort_merge_join_indexes(&left_on, &right_on, 2, 2);
    assert!(result.is_empty());
}

#[test]
fn sort_merge_join_with_multiple_matches() {
    let left_on: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 2, 2])];
    let right_on: Vec<Column<TestScalar>> = vec![Column::Int(&[2, 3])];
    let result = get_sort_merge_join_indexes(&left_on, &right_on, 3, 2);
    // Rows 1 and 2 in left (value 2) match row 0 in right (value 2)
    assert_eq!(result, vec![(1, 0), (2, 0)]);
}

#[test]
fn sort_merge_join_with_cartesian_matches() {
    let left_on: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 1])];
    let right_on: Vec<Column<TestScalar>> = vec![Column::Int(&[1, 1])];
    let result = get_sort_merge_join_indexes(&left_on, &right_on, 2, 2);
    // 2x2 cartesian product for matching rows
    assert_eq!(result.len(), 4);
}
