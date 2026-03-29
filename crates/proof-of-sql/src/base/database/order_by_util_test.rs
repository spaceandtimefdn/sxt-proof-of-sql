use crate::{
    base::{
        database::{order_by_util::*, Column, ColumnType, TableOperationError},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use core::cmp::Ordering;

#[test]
fn we_can_compare_indexes_by_columns_with_no_columns() {
    let columns: &[Column<TestScalar>; 0] = &[];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 3, 2), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_bigint_columns() {
    let slice_a = &[55, 44, 66, 66, 66, 77, 66, 66, 66, 66];
    let slice_b = &[22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let column_a = Column::BigInt::<DoryScalar>(slice_a);
    let column_b = Column::BigInt::<DoryScalar>(slice_b);
    let column_c = Column::BigInt::<DoryScalar>(slice_c);

    let columns = &[column_a];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Less);
    let columns = &[column_a, column_b];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Equal);
    let columns = &[column_a, column_b, column_c];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 6, 9), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_mixed_columns() {
    let slice_a = &["55", "44", "66", "66", "66", "77", "66", "66", "66", "66"];
    let slice_b = &[22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let scals_a: Vec<TestScalar> = slice_a.iter().map(core::convert::Into::into).collect();
    let column_a = Column::VarChar((slice_a, &scals_a));
    let column_b = Column::Int128(slice_b);
    let column_c = Column::BigInt(slice_c);

    let columns = &[column_a];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Less);
    let columns = &[column_a, column_b];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Equal);
    let columns = &[column_a, column_b, column_c];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 6, 9), Ordering::Equal);
}

#[test]
fn we_can_compare_single_row_of_tables() {
    let left_slice_a = &[55, 44, 44, 66, 66, 77, 66, 66, 66, 66];
    let left_slice_b = &[22, 44, 55, 44, 33, 22, 22, 11, 22, 22];
    let left_slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let left_column_a = Column::BigInt::<TestScalar>(left_slice_a);
    let left_column_b = Column::BigInt::<TestScalar>(left_slice_b);
    let left_column_c = Column::BigInt::<TestScalar>(left_slice_c);
    let left = &[left_column_a, left_column_b, left_column_c];

    let right_slice_a = &[77, 44, 66, 44, 77, 77, 66, 66, 55, 66];
    let right_slice_b = &[22, 55, 11, 77, 33, 33, 22, 22, 22, 11];
    let right_slice_c = &[11, 55, 22, 0, 77, 11, 33, 55, 11, 22];
    let right_column_a = Column::BigInt::<TestScalar>(right_slice_a);
    let right_column_b = Column::BigInt::<TestScalar>(right_slice_b);
    let right_column_c = Column::BigInt::<TestScalar>(right_slice_c);
    let right = &[right_column_a, right_column_b, right_column_c];

    assert_eq!(
        compare_single_row_of_tables(left, right, 0, 1).unwrap(),
        Ordering::Greater
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 1, 2).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 2, 3).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 2, 1).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 5, 0).unwrap(),
        Ordering::Equal
    );
}

#[test]
fn we_cannot_compare_single_row_of_tables_if_type_mismatch() {
    let left_slice = &[55, 44, 66, 66, 66, 77, 66, 66, 66, 66];
    let right_slice = &[
        true, false, true, true, false, true, false, true, false, true,
    ];
    let left_column = Column::BigInt::<TestScalar>(left_slice);
    let right_column = Column::Boolean::<TestScalar>(right_slice);
    let left = &[left_column];
    let right = &[right_column];
    assert_eq!(
        compare_single_row_of_tables(left, right, 0, 1),
        Err(TableOperationError::JoinIncompatibleTypes {
            left_type: ColumnType::BigInt,
            right_type: ColumnType::Boolean
        })
    );
}

#[test]
fn we_can_compare_indexes_by_columns_for_scalar_columns() {
    let slice_a = &[55, 44, 66, 66, 66, 77, 66, 66, 66, 66];
    let slice_b = &[22, 44, 11, 44, 33, 22, 22, 11, 22, 22];
    let slice_c = &[11, 55, 11, 44, 77, 11, 22, 55, 11, 22];
    let scals_a: Vec<TestScalar> = slice_a.iter().map(core::convert::Into::into).collect();
    let column_a = Column::Scalar(&scals_a);
    let column_b = Column::Int128(slice_b);
    let column_c = Column::BigInt(slice_c);

    let columns = &[column_a];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 2, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 0), Ordering::Less);
    let columns = &[column_a, column_b];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Equal);
    let columns = &[column_a, column_b, column_c];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 2, 7), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 6, 9), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_varbinary_columns() {
    let raw_bytes = [
        b"foo".as_ref(),
        b"bar".as_ref(),
        b"baz".as_ref(),
        b"baz".as_ref(),
        b"bar".as_ref(),
    ];
    let scalars: Vec<TestScalar> = raw_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect();
    let col_varbinary = Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));
    let columns = &[col_varbinary];

    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater); // "foo" vs "bar"
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less); // "bar" vs "baz"
    assert_eq!(compare_indexes_by_columns(columns, 2, 3), Ordering::Equal); // "baz" vs "baz"
    assert_eq!(compare_indexes_by_columns(columns, 3, 4), Ordering::Greater); // "baz" vs "bar"
    assert_eq!(compare_indexes_by_columns(columns, 1, 4), Ordering::Equal); // "bar" vs "bar"
}

#[test]
fn we_can_compare_indexes_by_columns_for_boolean_columns() {
    let slice = &[true, false, true, false];
    let column = Column::Boolean::<TestScalar>(slice);
    let columns = &[column];
    // true > false
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Greater);
    // false < true
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    // true == true
    assert_eq!(compare_indexes_by_columns(columns, 0, 2), Ordering::Equal);
    // false == false
    assert_eq!(compare_indexes_by_columns(columns, 1, 3), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_uint8_columns() {
    let slice = &[10u8, 20, 10, 30];
    let column = Column::Uint8::<TestScalar>(slice);
    let columns = &[column];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 0, 2), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_tinyint_columns() {
    let slice = &[-5i8, 0, 5, 0];
    let column = Column::TinyInt::<TestScalar>(slice);
    let columns = &[column];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 2, 0), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 1, 3), Ordering::Equal);
}

#[test]
fn we_can_compare_indexes_by_columns_for_smallint_columns() {
    let slice = &[100i16, 200, 100, -50];
    let column = Column::SmallInt::<TestScalar>(slice);
    let columns = &[column];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 0, 2), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 3, 0), Ordering::Less);
}

#[test]
fn we_can_compare_indexes_by_columns_for_int_columns() {
    let slice = &[1000i32, 2000, 1000, -500];
    let column = Column::Int::<TestScalar>(slice);
    let columns = &[column];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 0, 2), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 3, 0), Ordering::Less);
}

#[test]
fn we_can_compare_indexes_by_columns_for_decimal75_columns() {
    let precision = Precision::new(10).unwrap();
    let scale = 2i8;
    let vals: Vec<TestScalar> = [100i64, 200, 100, 50]
        .iter()
        .map(|&v| TestScalar::from(v))
        .collect();
    let column = Column::Decimal75(precision, scale, &vals);
    let columns = &[column];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 0, 2), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 3, 0), Ordering::Less);
}

#[test]
fn we_can_compare_indexes_by_columns_for_timestamp_columns() {
    let timestamps = [1_000i64, 2_000, 1_000, 500];
    let column =
        Column::TimestampTZ::<TestScalar>(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps);
    let columns = &[column];
    assert_eq!(compare_indexes_by_columns(columns, 0, 1), Ordering::Less);
    assert_eq!(compare_indexes_by_columns(columns, 1, 2), Ordering::Greater);
    assert_eq!(compare_indexes_by_columns(columns, 0, 2), Ordering::Equal);
    assert_eq!(compare_indexes_by_columns(columns, 3, 0), Ordering::Less);
}

#[test]
fn we_can_compare_single_row_of_tables_for_timestamp_columns() {
    let left_timestamps = [1_000i64, 2_000, 3_000];
    let right_timestamps = [2_000i64, 2_000, 1_000];
    let left_col = Column::TimestampTZ::<TestScalar>(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        &left_timestamps,
    );
    let right_col = Column::TimestampTZ::<TestScalar>(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        &right_timestamps,
    );
    let left = &[left_col];
    let right = &[right_col];
    assert_eq!(
        compare_single_row_of_tables(left, right, 0, 0).unwrap(),
        Ordering::Less
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 1, 1).unwrap(),
        Ordering::Equal
    );
    assert_eq!(
        compare_single_row_of_tables(left, right, 2, 2).unwrap(),
        Ordering::Greater
    );
}

#[test]
fn we_cannot_compare_single_row_of_tables_with_incompatible_timestamp_and_bigint() {
    let left_col = Column::TimestampTZ::<TestScalar>(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        &[1_000i64],
    );
    let right_col = Column::BigInt::<TestScalar>(&[1_000i64]);
    let left = &[left_col];
    let right = &[right_col];
    assert_eq!(
        compare_single_row_of_tables(left, right, 0, 0),
        Err(TableOperationError::JoinIncompatibleTypes {
            left_type: ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
            right_type: ColumnType::BigInt,
        })
    );
}
