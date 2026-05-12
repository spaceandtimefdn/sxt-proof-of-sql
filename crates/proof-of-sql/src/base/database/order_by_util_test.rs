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
fn we_can_compare_indexes_by_columns_for_more_column_types() {
    let bools = &[false, true, true];
    let uint8s = &[3_u8, 1, 1];
    let tinyints = &[3_i8, 1, 1];
    let smallints = &[3_i16, 1, 1];
    let ints = &[3_i32, 1, 1];
    let decimals: Vec<TestScalar> = [3, 1, 1].iter().map(core::convert::Into::into).collect();

    assert_eq!(
        compare_indexes_by_columns(&[Column::Boolean::<TestScalar>(bools)], 0, 1),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_columns(&[Column::Uint8::<TestScalar>(uint8s)], 0, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_columns(&[Column::TinyInt::<TestScalar>(tinyints)], 1, 0),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_columns(&[Column::SmallInt::<TestScalar>(smallints)], 1, 2),
        Ordering::Equal
    );
    assert_eq!(
        compare_indexes_by_columns(&[Column::Int::<TestScalar>(ints)], 0, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_columns(
            &[Column::Decimal75(Precision::new(10).unwrap(), 2, &decimals)],
            0,
            1
        ),
        Ordering::Greater
    );
}

#[test]
fn we_can_compare_single_row_of_tables_for_more_column_types() {
    let left_bools = &[false];
    let right_bools = &[true];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::Boolean::<TestScalar>(left_bools)],
            &[Column::Boolean::<TestScalar>(right_bools)],
            0,
            0
        )
        .unwrap(),
        Ordering::Less
    );

    let left_uint8s = &[1_u8];
    let right_uint8s = &[2_u8];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::Uint8::<TestScalar>(left_uint8s)],
            &[Column::Uint8::<TestScalar>(right_uint8s)],
            0,
            0
        )
        .unwrap(),
        Ordering::Less
    );

    let left_tinyints = &[3_i8];
    let right_tinyints = &[2_i8];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::TinyInt::<TestScalar>(left_tinyints)],
            &[Column::TinyInt::<TestScalar>(right_tinyints)],
            0,
            0
        )
        .unwrap(),
        Ordering::Greater
    );

    let left_smallints = &[4_i16];
    let right_smallints = &[4_i16];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::SmallInt::<TestScalar>(left_smallints)],
            &[Column::SmallInt::<TestScalar>(right_smallints)],
            0,
            0
        )
        .unwrap(),
        Ordering::Equal
    );

    let left_ints = &[5_i32];
    let right_ints = &[4_i32];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::Int::<TestScalar>(left_ints)],
            &[Column::Int::<TestScalar>(right_ints)],
            0,
            0
        )
        .unwrap(),
        Ordering::Greater
    );

    let left_timestamps = &[5_i64];
    let right_timestamps = &[6_i64];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::<TestScalar>::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                left_timestamps
            )],
            &[Column::<TestScalar>::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                right_timestamps
            )],
            0,
            0
        )
        .unwrap(),
        Ordering::Less
    );

    let left_int128s = &[7_i128];
    let right_int128s = &[6_i128];
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::Int128::<TestScalar>(left_int128s)],
            &[Column::Int128::<TestScalar>(right_int128s)],
            0,
            0
        )
        .unwrap(),
        Ordering::Greater
    );

    let left_decimals: Vec<TestScalar> = [8].iter().map(core::convert::Into::into).collect();
    let right_decimals: Vec<TestScalar> = [8].iter().map(core::convert::Into::into).collect();
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::Decimal75(
                Precision::new(10).unwrap(),
                2,
                &left_decimals
            )],
            &[Column::Decimal75(
                Precision::new(10).unwrap(),
                2,
                &right_decimals
            )],
            0,
            0
        )
        .unwrap(),
        Ordering::Equal
    );

    let left_scalars: Vec<TestScalar> = [9].iter().map(core::convert::Into::into).collect();
    let right_scalars: Vec<TestScalar> = [10].iter().map(core::convert::Into::into).collect();
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::Scalar(left_scalars.as_slice())],
            &[Column::Scalar(right_scalars.as_slice())],
            0,
            0
        )
        .unwrap(),
        Ordering::Less
    );

    let left_strings = &["apple"];
    let right_strings = &["banana"];
    let left_string_scalars: Vec<TestScalar> =
        left_strings.iter().map(core::convert::Into::into).collect();
    let right_string_scalars: Vec<TestScalar> = right_strings
        .iter()
        .map(core::convert::Into::into)
        .collect();
    assert_eq!(
        compare_single_row_of_tables(
            &[Column::VarChar((left_strings, &left_string_scalars))],
            &[Column::VarChar((right_strings, &right_string_scalars))],
            0,
            0
        )
        .unwrap(),
        Ordering::Less
    );
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
