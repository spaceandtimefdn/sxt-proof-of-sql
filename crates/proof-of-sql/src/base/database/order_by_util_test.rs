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

fn assert_single_column_ordering(
    left_column: Column<'_, TestScalar>,
    right_column: Column<'_, TestScalar>,
    expected: Ordering,
) {
    let left = [left_column];
    let right = [right_column];
    assert_eq!(
        compare_single_row_of_tables(&left, &right, 0, 0).unwrap(),
        expected
    );
}

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
fn we_can_compare_indexes_by_columns_for_remaining_column_types() {
    let uint8_values = [1_u8, 3, 3];
    let uint8_column = Column::Uint8::<TestScalar>(&uint8_values);
    assert_eq!(
        compare_indexes_by_columns(&[uint8_column], 0, 1),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_columns(&[uint8_column], 1, 2),
        Ordering::Equal
    );

    let tinyint_values = [-2_i8, 4, 4];
    let tinyint_column = Column::TinyInt::<TestScalar>(&tinyint_values);
    assert_eq!(
        compare_indexes_by_columns(&[tinyint_column], 0, 1),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_columns(&[tinyint_column], 1, 2),
        Ordering::Equal
    );

    let smallint_values = [8_i16, -1, -1];
    let smallint_column = Column::SmallInt::<TestScalar>(&smallint_values);
    assert_eq!(
        compare_indexes_by_columns(&[smallint_column], 0, 1),
        Ordering::Greater
    );
    assert_eq!(
        compare_indexes_by_columns(&[smallint_column], 1, 2),
        Ordering::Equal
    );

    let decimal_values = [
        TestScalar::from(10),
        TestScalar::from(25),
        TestScalar::from(25),
    ];
    let decimal_column =
        Column::Decimal75(Precision::new(10).unwrap(), 2, decimal_values.as_slice());
    assert_eq!(
        compare_indexes_by_columns(&[decimal_column], 0, 1),
        Ordering::Less
    );
    assert_eq!(
        compare_indexes_by_columns(&[decimal_column], 1, 2),
        Ordering::Equal
    );
}

#[test]
fn we_can_compare_single_row_of_tables_for_remaining_column_types() {
    assert_single_column_ordering(
        Column::Uint8::<TestScalar>(&[1]),
        Column::Uint8::<TestScalar>(&[2]),
        Ordering::Less,
    );
    assert_single_column_ordering(
        Column::TinyInt::<TestScalar>(&[-1]),
        Column::TinyInt::<TestScalar>(&[-2]),
        Ordering::Greater,
    );
    assert_single_column_ordering(
        Column::SmallInt::<TestScalar>(&[5]),
        Column::SmallInt::<TestScalar>(&[5]),
        Ordering::Equal,
    );
    assert_single_column_ordering(
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[100]),
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[200]),
        Ordering::Less,
    );
    assert_single_column_ordering(
        Column::Int128::<TestScalar>(&[123_i128]),
        Column::Int128::<TestScalar>(&[45_i128]),
        Ordering::Greater,
    );

    let left_decimal = [TestScalar::from(12)];
    let right_decimal = [TestScalar::from(30)];
    assert_single_column_ordering(
        Column::Decimal75(Precision::new(10).unwrap(), 2, &left_decimal),
        Column::Decimal75(Precision::new(10).unwrap(), 2, &right_decimal),
        Ordering::Less,
    );

    let left_scalar = [TestScalar::from(9)];
    let right_scalar = [TestScalar::from(9)];
    assert_single_column_ordering(
        Column::Scalar(&left_scalar),
        Column::Scalar(&right_scalar),
        Ordering::Equal,
    );

    let left_strings = ["bbb"];
    let right_strings = ["aaa"];
    let left_string_scalars = left_strings
        .iter()
        .map(TestScalar::from)
        .collect::<Vec<_>>();
    let right_string_scalars = right_strings
        .iter()
        .map(TestScalar::from)
        .collect::<Vec<_>>();
    assert_single_column_ordering(
        Column::VarChar((left_strings.as_slice(), left_string_scalars.as_slice())),
        Column::VarChar((right_strings.as_slice(), right_string_scalars.as_slice())),
        Ordering::Greater,
    );
}

#[test]
fn we_can_compare_single_row_of_tables_for_varbinary_columns() {
    let left_bytes = [b"bar".as_ref()];
    let right_bytes = [b"foo".as_ref()];
    let left_scalars = left_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect::<Vec<_>>();
    let right_scalars = right_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect::<Vec<_>>();

    assert_single_column_ordering(
        Column::VarBinary((left_bytes.as_slice(), left_scalars.as_slice())),
        Column::VarBinary((right_bytes.as_slice(), right_scalars.as_slice())),
        Ordering::Less,
    );
}
