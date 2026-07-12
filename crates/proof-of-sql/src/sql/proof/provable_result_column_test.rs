use super::{ProvableResultColumn, ProvableResultElement};
use crate::base::{
    database::Column,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};

fn write_column(column: impl ProvableResultColumn, length: u64) -> Vec<u8> {
    let mut out = vec![0_u8; column.num_bytes(length)];
    let bytes_written = column.write(&mut out, length);
    assert_eq!(bytes_written, out.len());
    out
}

fn assert_column_matches_values<'a, T>(column: Column<'a, TestScalar>, values: &'a [T])
where
    T: ProvableResultElement<'a>,
{
    let length = u64::try_from(values.len()).unwrap();
    assert_eq!(column.num_bytes(length), values.num_bytes(length));
    assert_eq!(write_column(column, length), write_column(values, length));
}

#[test]
fn slice_result_columns_encode_each_element_in_order() {
    let values = [1_i64, -2_i64, 300_i64];

    let out = write_column(&values[..], values.len() as u64);
    let expected_len: usize = values
        .iter()
        .map(ProvableResultElement::required_bytes)
        .sum();

    assert_eq!(out.len(), expected_len);
    let (decoded, bytes_read) = super::decode_multiple_elements::<i64>(&out, values.len()).unwrap();
    assert_eq!(bytes_read, out.len());
    assert_eq!(decoded, values);
}

#[test]
fn array_result_columns_delegate_to_slice_encoding() {
    let values = [10_i16, -20_i16, 30_i16];

    assert_eq!(
        write_column(values, values.len() as u64),
        write_column(&values[..], values.len() as u64)
    );
}

#[test]
fn database_column_variants_delegate_to_their_backing_result_columns() {
    let booleans = [true, false, true];
    assert_column_matches_values(Column::Boolean(&booleans), &booleans);

    let uint8s = [1_u8, 2, 3];
    assert_column_matches_values(Column::Uint8(&uint8s), &uint8s);

    let tinyints = [-2_i8, 0, 7];
    assert_column_matches_values(Column::TinyInt(&tinyints), &tinyints);

    let smallints = [-300_i16, 0, 400];
    assert_column_matches_values(Column::SmallInt(&smallints), &smallints);

    let ints = [-30_000_i32, 0, 40_000];
    assert_column_matches_values(Column::Int(&ints), &ints);

    let bigints = [-3_000_000_i64, 0, 4_000_000];
    assert_column_matches_values(Column::BigInt(&bigints), &bigints);

    let int128s = [-3_000_000_000_i128, 0, 4_000_000_000];
    assert_column_matches_values(Column::Int128(&int128s), &int128s);

    let scalars = [
        TestScalar::from(11u8),
        TestScalar::from(22u8),
        TestScalar::from(33u8),
    ];
    assert_column_matches_values(Column::Scalar(&scalars), &scalars);
    assert_column_matches_values(
        Column::Decimal75(Precision::new(6).unwrap(), 2, &scalars),
        &scalars,
    );

    let timestamps = [1_700_000_000_i64, 1_700_000_001, 1_700_000_002];
    assert_column_matches_values(
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps),
        &timestamps,
    );

    let strings = ["alpha", "", "gamma"];
    let string_scalars = [
        TestScalar::from(1u8),
        TestScalar::from(2u8),
        TestScalar::from(3u8),
    ];
    assert_column_matches_values(Column::VarChar((&strings, &string_scalars)), &strings);

    let binary_values: [&[u8]; 3] = [&b"left"[..], &b""[..], &b"right"[..]];
    let binary_scalars = [
        TestScalar::from(4u8),
        TestScalar::from(5u8),
        TestScalar::from(6u8),
    ];
    assert_column_matches_values(
        Column::VarBinary((&binary_values, &binary_scalars)),
        &binary_values,
    );
}
