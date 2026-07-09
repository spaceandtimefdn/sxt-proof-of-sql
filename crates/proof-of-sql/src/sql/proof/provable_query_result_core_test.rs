use super::{ProvableQueryResult, QueryError};
use crate::base::{
    database::{
        owned_table_utility::{
            bigint, boolean, decimal75, int, int128, owned_table, scalar, smallint, timestamptz,
            tinyint, uint8, varbinary, varchar,
        },
        table_utility::{borrowed_bigint, borrowed_varchar, table},
        Column, ColumnField, ColumnType,
    },
    math::decimal::Precision,
    polynomial::compute_evaluation_vector,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
};
use alloc::{vec, vec::Vec};
use bumpalo::Bump;

fn all_column_fields() -> Vec<ColumnField> {
    vec![
        ColumnField::new("bool_col".into(), ColumnType::Boolean),
        ColumnField::new("u8_col".into(), ColumnType::Uint8),
        ColumnField::new("i8_col".into(), ColumnType::TinyInt),
        ColumnField::new("i16_col".into(), ColumnType::SmallInt),
        ColumnField::new("i32_col".into(), ColumnType::Int),
        ColumnField::new("i64_col".into(), ColumnType::BigInt),
        ColumnField::new("i128_col".into(), ColumnType::Int128),
        ColumnField::new("varchar_col".into(), ColumnType::VarChar),
        ColumnField::new("varbinary_col".into(), ColumnType::VarBinary),
        ColumnField::new("scalar_col".into(), ColumnType::Scalar),
        ColumnField::new(
            "decimal_col".into(),
            ColumnType::Decimal75(Precision::new(12).unwrap(), 2),
        ),
        ColumnField::new(
            "timestamp_col".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        ),
    ]
}

#[test]
fn to_owned_table_decodes_every_supported_result_column_type() {
    let booleans = [true, false];
    let uints = [3_u8, 4];
    let tinyints = [-2_i8, 7];
    let smallints = [-300_i16, 450];
    let ints = [-70_000_i32, 80_000];
    let bigints = [-9_000_000_000_i64, 12_000_000_000];
    let int128s = [-12_345_678_901_234_567_890_i128, 98_765_432_109_876_543];
    let strings = ["alpha", "beta"];
    let string_scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
    let raw_bytes: [&[u8]; 2] = [b"left".as_ref(), b"right".as_ref()];
    let byte_scalars = raw_bytes
        .iter()
        .map(|bytes| TestScalar::from_byte_slice_via_hash(bytes))
        .collect::<Vec<_>>();
    let scalar_values = [TestScalar::from(21), TestScalar::from(34)];
    let decimal_values = [TestScalar::from(123_456), TestScalar::from(-654_321)];
    let timestamps = [1_700_000_001_i64, 1_700_000_002];
    let precision = Precision::new(12).unwrap();
    let timezone = PoSQLTimeZone::utc();
    let fields = all_column_fields();
    let columns = [
        Column::Boolean(&booleans),
        Column::Uint8(&uints),
        Column::TinyInt(&tinyints),
        Column::SmallInt(&smallints),
        Column::Int(&ints),
        Column::BigInt(&bigints),
        Column::Int128(&int128s),
        Column::VarChar((&strings, &string_scalars)),
        Column::VarBinary((raw_bytes.as_slice(), &byte_scalars)),
        Column::Scalar(&scalar_values),
        Column::Decimal75(precision, 2, &decimal_values),
        Column::TimestampTZ(PoSQLTimeUnit::Second, timezone, &timestamps),
    ];

    let result = ProvableQueryResult::new(2, &columns);

    assert_eq!(result.num_columns(), fields.len());
    assert_eq!(result.table_length(), 2);
    assert_eq!(
        result.to_owned_table::<TestScalar>(&fields).unwrap(),
        owned_table::<TestScalar>([
            boolean("bool_col", booleans),
            uint8("u8_col", uints),
            tinyint("i8_col", tinyints),
            smallint("i16_col", smallints),
            int("i32_col", ints),
            bigint("i64_col", bigints),
            int128("i128_col", int128s),
            varchar("varchar_col", strings),
            varbinary("varbinary_col", [b"left".to_vec(), b"right".to_vec()]),
            scalar("scalar_col", scalar_values),
            decimal75("decimal_col", 12, 2, decimal_values),
            timestamptz("timestamp_col", PoSQLTimeUnit::Second, timezone, timestamps),
        ])
    );
}

#[test]
fn evaluate_uses_field_types_to_decode_each_result_column() {
    let booleans = [true, false];
    let uints = [3_u8, 4];
    let tinyints = [-2_i8, 7];
    let smallints = [-300_i16, 450];
    let ints = [-70_000_i32, 80_000];
    let bigints = [-9_000_000_000_i64, 12_000_000_000];
    let int128s = [-12_345_678_901_234_567_890_i128, 98_765_432_109_876_543];
    let strings = ["alpha", "beta"];
    let string_scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
    let raw_bytes: [&[u8]; 2] = [b"left".as_ref(), b"right".as_ref()];
    let byte_scalars = raw_bytes
        .iter()
        .map(|bytes| TestScalar::from_byte_slice_via_hash(bytes))
        .collect::<Vec<_>>();
    let scalar_values = [TestScalar::from(21), TestScalar::from(34)];
    let decimal_values = [TestScalar::from(123_456), TestScalar::from(-654_321)];
    let timestamps = [1_700_000_001_i64, 1_700_000_002];
    let precision = Precision::new(12).unwrap();
    let timezone = PoSQLTimeZone::utc();
    let fields = all_column_fields();
    let columns = [
        Column::Boolean(&booleans),
        Column::Uint8(&uints),
        Column::TinyInt(&tinyints),
        Column::SmallInt(&smallints),
        Column::Int(&ints),
        Column::BigInt(&bigints),
        Column::Int128(&int128s),
        Column::VarChar((&strings, &string_scalars)),
        Column::VarBinary((raw_bytes.as_slice(), &byte_scalars)),
        Column::Scalar(&scalar_values),
        Column::Decimal75(precision, 2, &decimal_values),
        Column::TimestampTZ(PoSQLTimeUnit::Second, timezone, &timestamps),
    ];
    let result = ProvableQueryResult::new(2, &columns);

    let evaluation_point = [TestScalar::ZERO];
    let mut evaluation_vector = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vector, &evaluation_point);
    assert_eq!(evaluation_vector, [TestScalar::ONE, TestScalar::ZERO]);

    assert_eq!(
        result.evaluate(&evaluation_point, 2, &fields).unwrap(),
        vec![
            TestScalar::ONE,
            TestScalar::from(uints[0]),
            TestScalar::from(tinyints[0]),
            TestScalar::from(smallints[0]),
            TestScalar::from(ints[0]),
            TestScalar::from(bigints[0]),
            TestScalar::from(int128s[0]),
            TestScalar::from(strings[0]),
            TestScalar::from_byte_slice_via_hash(raw_bytes[0]),
            scalar_values[0],
            decimal_values[0],
            TestScalar::from(timestamps[0]),
        ]
    );
}

#[test]
fn malformed_result_metadata_is_rejected_before_decoding() {
    let bigints = [1_i64, 2];
    let result = ProvableQueryResult::new(2, &[Column::<TestScalar>::BigInt(&bigints)]);
    let fields = [ColumnField::new("a".into(), ColumnType::BigInt)];
    let empty_fields = [];

    assert!(matches!(
        result.evaluate(&[TestScalar::ZERO], 2, &empty_fields),
        Err(QueryError::InvalidColumnCount)
    ));
    assert!(matches!(
        result.to_owned_table::<TestScalar>(&empty_fields),
        Err(QueryError::InvalidColumnCount)
    ));

    let mut tampered_result = result.clone();
    *tampered_result.num_columns_mut() = 2;
    assert!(matches!(
        tampered_result.evaluate(&[TestScalar::ZERO], 2, &fields),
        Err(QueryError::InvalidColumnCount)
    ));
}

#[test]
fn trailing_encoded_bytes_are_reported_as_evaluation_errors() {
    let mut result = ProvableQueryResult::new_from_raw_data(0, 0, vec![1]);
    result.data_mut().push(2);

    assert!(matches!(
        result.evaluate::<TestScalar>(&[], 0, &[]),
        Err(QueryError::MiscellaneousEvaluationError)
    ));
}

#[test]
fn provable_query_result_can_be_built_from_a_borrowed_table() {
    let alloc = Bump::new();
    let table = table::<TestScalar>([
        borrowed_bigint("bigint_col", [9_i64, 10], &alloc),
        borrowed_varchar("text_col", ["left", "right"], &alloc),
    ]);
    let result = ProvableQueryResult::from(table);
    let fields = [
        ColumnField::new("bigint_col".into(), ColumnType::BigInt),
        ColumnField::new("text_col".into(), ColumnType::VarChar),
    ];

    assert_eq!(
        result.to_owned_table::<TestScalar>(&fields).unwrap(),
        owned_table::<TestScalar>([
            bigint("bigint_col", [9_i64, 10]),
            varchar("text_col", ["left", "right"]),
        ])
    );
}
