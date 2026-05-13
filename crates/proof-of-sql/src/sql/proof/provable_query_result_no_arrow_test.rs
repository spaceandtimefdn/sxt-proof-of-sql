use super::{ProvableQueryResult, QueryError};
use crate::base::{
    database::{Column, ColumnField, ColumnType, OwnedColumn, OwnedTable},
    math::decimal::Precision,
    polynomial::compute_evaluation_vector,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
};
use alloc::{vec, vec::Vec};
use sqlparser::ast::Ident;

fn field(name: &str, data_type: ColumnType) -> ColumnField {
    ColumnField::new(name.into(), data_type)
}

fn dot_pair(values: [TestScalar; 2], weights: [TestScalar; 2]) -> TestScalar {
    values[0] * weights[0] + values[1] * weights[1]
}

#[test]
fn we_can_evaluate_all_provable_result_column_types_without_arrow() {
    let precision = Precision::new(75).unwrap();
    let timezone = PoSQLTimeZone::utc();
    let boolean_values = [true, false];
    let uint8_values = [2_u8, 9];
    let tinyint_values = [-2_i8, 7];
    let smallint_values = [-30_i16, 40];
    let int_values = [-300_i32, 400];
    let bigint_values = [-3000_i64, 4000];
    let int128_values = [-30000_i128, 40000];
    let decimal_values = [TestScalar::from(11_u64), TestScalar::from(12_u64)];
    let scalar_values = [TestScalar::from(13_u64), TestScalar::from(14_u64)];
    let varchar_values = ["alpha", "beta"];
    let varchar_scalars = varchar_values
        .iter()
        .map(|value| TestScalar::from(*value))
        .collect::<Vec<_>>();
    let varbinary_values: [&[u8]; 2] = [b"aa".as_ref(), b"bb".as_ref()];
    let varbinary_scalars = varbinary_values
        .iter()
        .map(|value| TestScalar::from_byte_slice_via_hash(value))
        .collect::<Vec<_>>();
    let timestamp_values = [100_i64, 200];
    let columns: [Column<TestScalar>; 12] = [
        Column::Boolean(&boolean_values),
        Column::Uint8(&uint8_values),
        Column::TinyInt(&tinyint_values),
        Column::SmallInt(&smallint_values),
        Column::Int(&int_values),
        Column::BigInt(&bigint_values),
        Column::Int128(&int128_values),
        Column::Decimal75(precision, 2, &decimal_values),
        Column::Scalar(&scalar_values),
        Column::VarChar((varchar_values.as_slice(), varchar_scalars.as_slice())),
        Column::VarBinary((varbinary_values.as_slice(), varbinary_scalars.as_slice())),
        Column::TimestampTZ(PoSQLTimeUnit::Second, timezone, &timestamp_values),
    ];
    let column_fields = vec![
        field("boolean_col", ColumnType::Boolean),
        field("uint8_col", ColumnType::Uint8),
        field("tinyint_col", ColumnType::TinyInt),
        field("smallint_col", ColumnType::SmallInt),
        field("int_col", ColumnType::Int),
        field("bigint_col", ColumnType::BigInt),
        field("int128_col", ColumnType::Int128),
        field("decimal_col", ColumnType::Decimal75(precision, 2)),
        field("scalar_col", ColumnType::Scalar),
        field("varchar_col", ColumnType::VarChar),
        field("varbinary_col", ColumnType::VarBinary),
        field(
            "timestamp_col",
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, timezone),
        ),
    ];
    let result = ProvableQueryResult::new(2, &columns);
    let evaluation_point = [TestScalar::from(5_u64)];
    let mut weights = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut weights, &evaluation_point);

    let evaluations = result
        .evaluate(&evaluation_point, 2, &column_fields)
        .unwrap();
    let expected = vec![
        dot_pair([TestScalar::from(1_u64), TestScalar::ZERO], weights),
        dot_pair([TestScalar::from(2_u64), TestScalar::from(9_u64)], weights),
        dot_pair(
            [TestScalar::from(-2_i128), TestScalar::from(7_u64)],
            weights,
        ),
        dot_pair(
            [TestScalar::from(-30_i128), TestScalar::from(40_u64)],
            weights,
        ),
        dot_pair(
            [TestScalar::from(-300_i128), TestScalar::from(400_u64)],
            weights,
        ),
        dot_pair(
            [TestScalar::from(-3000_i128), TestScalar::from(4000_u64)],
            weights,
        ),
        dot_pair(
            [TestScalar::from(-30000_i128), TestScalar::from(40000_u64)],
            weights,
        ),
        dot_pair(decimal_values, weights),
        dot_pair(scalar_values, weights),
        dot_pair(
            [TestScalar::from("alpha"), TestScalar::from("beta")],
            weights,
        ),
        dot_pair([varbinary_scalars[0], varbinary_scalars[1]], weights),
        dot_pair(
            [TestScalar::from(100_u64), TestScalar::from(200_u64)],
            weights,
        ),
    ];
    assert_eq!(evaluations, expected);
}

#[test]
fn we_can_convert_all_provable_result_column_types_without_arrow() {
    let precision = Precision::new(75).unwrap();
    let timezone = PoSQLTimeZone::utc();
    let decimal_values = [TestScalar::from(11_u64), TestScalar::from(12_u64)];
    let scalar_values = [TestScalar::from(13_u64), TestScalar::from(14_u64)];
    let varchar_values = ["alpha", "beta"];
    let varchar_scalars = varchar_values
        .iter()
        .map(|value| TestScalar::from(*value))
        .collect::<Vec<_>>();
    let varbinary_values: [&[u8]; 2] = [b"aa".as_ref(), b"bb".as_ref()];
    let varbinary_scalars = varbinary_values
        .iter()
        .map(|value| TestScalar::from_byte_slice_via_hash(value))
        .collect::<Vec<_>>();
    let columns: [Column<TestScalar>; 12] = [
        Column::Boolean(&[true, false]),
        Column::Uint8(&[2_u8, 9]),
        Column::TinyInt(&[-2_i8, 7]),
        Column::SmallInt(&[-30_i16, 40]),
        Column::Int(&[-300_i32, 400]),
        Column::BigInt(&[-3000_i64, 4000]),
        Column::Int128(&[-30000_i128, 40000]),
        Column::Decimal75(precision, 2, &decimal_values),
        Column::Scalar(&scalar_values),
        Column::VarChar((varchar_values.as_slice(), varchar_scalars.as_slice())),
        Column::VarBinary((varbinary_values.as_slice(), varbinary_scalars.as_slice())),
        Column::TimestampTZ(PoSQLTimeUnit::Second, timezone, &[100_i64, 200]),
    ];
    let column_fields = vec![
        field("boolean_col", ColumnType::Boolean),
        field("uint8_col", ColumnType::Uint8),
        field("tinyint_col", ColumnType::TinyInt),
        field("smallint_col", ColumnType::SmallInt),
        field("int_col", ColumnType::Int),
        field("bigint_col", ColumnType::BigInt),
        field("int128_col", ColumnType::Int128),
        field("decimal_col", ColumnType::Decimal75(precision, 2)),
        field("scalar_col", ColumnType::Scalar),
        field("varchar_col", ColumnType::VarChar),
        field("varbinary_col", ColumnType::VarBinary),
        field(
            "timestamp_col",
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, timezone),
        ),
    ];
    let result = ProvableQueryResult::new(2, &columns);

    let owned_table = result.to_owned_table::<TestScalar>(&column_fields).unwrap();
    let expected = OwnedTable::try_from_iter([
        (
            Ident::new("boolean_col"),
            OwnedColumn::Boolean(vec![true, false]),
        ),
        (Ident::new("uint8_col"), OwnedColumn::Uint8(vec![2, 9])),
        (Ident::new("tinyint_col"), OwnedColumn::TinyInt(vec![-2, 7])),
        (
            Ident::new("smallint_col"),
            OwnedColumn::SmallInt(vec![-30, 40]),
        ),
        (Ident::new("int_col"), OwnedColumn::Int(vec![-300, 400])),
        (
            Ident::new("bigint_col"),
            OwnedColumn::BigInt(vec![-3000, 4000]),
        ),
        (
            Ident::new("int128_col"),
            OwnedColumn::Int128(vec![-30000, 40000]),
        ),
        (
            Ident::new("decimal_col"),
            OwnedColumn::Decimal75(precision, 2, decimal_values.to_vec()),
        ),
        (
            Ident::new("scalar_col"),
            OwnedColumn::Scalar(scalar_values.to_vec()),
        ),
        (
            Ident::new("varchar_col"),
            OwnedColumn::VarChar(vec!["alpha".to_string(), "beta".to_string()]),
        ),
        (
            Ident::new("varbinary_col"),
            OwnedColumn::VarBinary(vec![b"aa".to_vec(), b"bb".to_vec()]),
        ),
        (
            Ident::new("timestamp_col"),
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, timezone, vec![100, 200]),
        ),
    ])
    .unwrap();
    assert_eq!(owned_table, expected);
}

#[test]
fn provable_query_result_rejects_malformed_inputs_without_arrow() {
    let columns: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let result = ProvableQueryResult::new(2, &columns);
    let column_fields = [field("bigint_col", ColumnType::BigInt)];

    assert_eq!(result.num_columns(), 1);
    assert_eq!(result.table_length(), 2);
    assert!(matches!(
        result.evaluate(&[TestScalar::from(5_u64)], 2, &[]),
        Err(QueryError::InvalidColumnCount)
    ));
    assert!(matches!(
        result.to_owned_table::<TestScalar>(&[]),
        Err(QueryError::InvalidColumnCount)
    ));

    let mut extra_data = result.clone();
    extra_data.data_mut().push(3);
    assert!(matches!(
        extra_data.evaluate(&[TestScalar::from(5_u64)], 2, &column_fields),
        Err(QueryError::MiscellaneousEvaluationError)
    ));

    let mut mismatched_count = result;
    *mismatched_count.num_columns_mut() = 2;
    assert!(matches!(
        mismatched_count.evaluate(&[TestScalar::from(5_u64)], 2, &column_fields),
        Err(QueryError::InvalidColumnCount)
    ));

    let mut malformed = ProvableQueryResult::new_from_raw_data(1, 1, vec![0b1111_1111_u8; 38]);
    malformed.data_mut()[37] = 0b0000_0001_u8;
    assert!(matches!(
        malformed.evaluate(&[TestScalar::from(5_u64)], 2, &column_fields),
        Err(QueryError::Overflow)
    ));
}
