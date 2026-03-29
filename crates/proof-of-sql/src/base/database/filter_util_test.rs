use crate::base::{
    database::{filter_util::*, Column},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;

#[test]
fn we_can_filter_columns() {
    let selection = vec![true, false, true, false, true];
    let str_scalars: [TestScalar; 5] = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
        Column::Decimal75(Precision::new(75).unwrap(), 0, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[1, 3, 5]),
            Column::Int128(&[1, 3, 5]),
            Column::VarChar((&["1", "3", "5"], &["1".into(), "3".into(), "5".into()])),
            Column::Scalar(&[1.into(), 3.into(), 5.into()]),
            Column::Decimal75(
                Precision::new(75).unwrap(),
                0,
                &[1.into(), 3.into(), 5.into()]
            )
        ]
    );
}
#[test]
fn we_can_filter_columns_with_empty_result() {
    let selection = vec![false, false, false, false, false];
    let str_scalars: [TestScalar; 5] = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let decimals = [1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    let columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[1, 2, 3, 4, 5]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &decimals),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[]),
            Column::Int128(&[]),
            Column::VarChar((&[], &[])),
            Column::Scalar(&[]),
            Column::Decimal75(Precision::new(75).unwrap(), -1, &[])
        ]
    );
}
#[test]
fn we_can_filter_empty_columns() {
    let selection = vec![];
    let columns = vec![
        Column::<TestScalar>::BigInt(&[]),
        Column::Int128(&[]),
        Column::VarChar((&[], &[])),
        Column::Scalar(&[]),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 0);
    assert_eq!(
        result,
        vec![
            Column::BigInt(&[]),
            Column::Int128(&[]),
            Column::VarChar((&[], &[])),
            Column::Scalar(&[]),
            Column::Decimal75(Precision::new(75).unwrap(), -1, &[])
        ]
    );
}

#[test]
fn we_can_filter_columns_with_varbinary() {
    let selection = vec![true, false, true, true, false];
    let raw_bytes = [b"foo".as_ref(), b"bar", b"baz", b"qux", b"quux"];
    let scalars: [TestScalar; 5] = raw_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let columns = vec![
        Column::VarBinary((&raw_bytes, &scalars)),
        Column::BigInt(&[10, 20, 30, 40, 50]),
    ];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    let filtered_bytes = [b"foo".as_ref(), b"baz", b"qux"];
    let filtered_scalars = filtered_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect::<Vec<_>>();
    assert_eq!(
        result,
        vec![
            Column::VarBinary((filtered_bytes.as_slice(), filtered_scalars.as_slice())),
            Column::BigInt(&[10, 30, 40]),
        ]
    );
}

#[test]
fn we_can_filter_boolean_columns() {
    let selection = vec![true, false, true, false, true];
    let columns = vec![Column::<TestScalar>::Boolean(&[true, false, true, true, false])];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(result, vec![Column::Boolean(&[true, true, false])]);
}

#[test]
fn we_can_filter_uint8_columns() {
    let selection = vec![true, false, true, false, true];
    let columns = vec![Column::<TestScalar>::Uint8(&[10u8, 20, 30, 40, 50])];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(result, vec![Column::Uint8(&[10u8, 30, 50])]);
}

#[test]
fn we_can_filter_tinyint_columns() {
    let selection = vec![true, false, true, false, true];
    let columns = vec![Column::<TestScalar>::TinyInt(&[-1i8, 2, -3, 4, -5])];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(result, vec![Column::TinyInt(&[-1i8, -3, -5])]);
}

#[test]
fn we_can_filter_smallint_columns() {
    let selection = vec![true, false, true, false, true];
    let columns = vec![Column::<TestScalar>::SmallInt(&[100i16, 200, 300, 400, 500])];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(result, vec![Column::SmallInt(&[100i16, 300, 500])]);
}

#[test]
fn we_can_filter_int_columns() {
    let selection = vec![true, false, true, false, true];
    let columns = vec![Column::<TestScalar>::Int(&[1000i32, 2000, 3000, 4000, 5000])];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(result, vec![Column::Int(&[1000i32, 3000, 5000])]);
}

#[test]
fn we_can_filter_timestamp_columns() {
    let selection = vec![true, false, true, false, true];
    let timestamps = [1_000i64, 2_000, 3_000, 4_000, 5_000];
    let columns = vec![Column::<TestScalar>::TimestampTZ(
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        &timestamps,
    )];
    let alloc = Bump::new();
    let (result, len) = filter_columns(&alloc, &columns, &selection);
    assert_eq!(len, 3);
    assert_eq!(
        result,
        vec![Column::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &[1_000i64, 3_000, 5_000]
        )]
    );
}

#[test]
#[should_panic]
fn we_cannot_filter_columns_with_mismatched_selection_length() {
    let columns = vec![Column::<TestScalar>::BigInt(&[1, 2, 3])];
    let selection = vec![true, false]; // length 2, column length 3
    let alloc = Bump::new();
    let _ = filter_columns(&alloc, &columns, &selection);
}
