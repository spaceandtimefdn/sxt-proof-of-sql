use super::{ProvableQueryResult, QueryError};
use crate::base::{
    database::{
        owned_table_utility::{
            bigint, boolean, decimal75, int, int128, owned_table, scalar, smallint, timestamptz,
            tinyint, uint8, varbinary, varchar,
        },
        Column, ColumnField, ColumnType,
    },
    math::decimal::Precision,
    polynomial::compute_evaluation_vector,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
};

#[test]
fn no_arrow_result_round_trips_all_column_types_to_owned_table() {
    let varchar_values = ["north", "south"];
    let varchar_scalars = varchar_values
        .iter()
        .map(|value| TestScalar::from(*value))
        .collect::<Vec<_>>();
    let binary_values = [b"left".as_slice(), b"right".as_slice()];
    let binary_scalars = binary_values
        .iter()
        .map(|value| TestScalar::from_byte_slice_via_hash(value))
        .collect::<Vec<_>>();
    let scalar_values = [TestScalar::from(11), TestScalar::from(12)];
    let decimal_values = [TestScalar::from(1234), TestScalar::from(5678)];
    let timestamp_unit = PoSQLTimeUnit::Millisecond;
    let timestamp_zone = PoSQLTimeZone::utc();
    let timestamp_values = [1_700_000_000_i64, 1_700_000_500_i64];
    let precision = Precision::new(20).unwrap();

    let columns = vec![
        Column::Boolean(&[true, false]),
        Column::Uint8(&[7, 9]),
        Column::TinyInt(&[-3, 4]),
        Column::SmallInt(&[-30, 40]),
        Column::Int(&[-300, 400]),
        Column::BigInt(&[-3_000, 4_000]),
        Column::Int128(&[-30_000, 40_000]),
        Column::VarChar((&varchar_values, &varchar_scalars)),
        Column::VarBinary((&binary_values, &binary_scalars)),
        Column::Scalar(&scalar_values),
        Column::Decimal75(precision, 2, &decimal_values),
        Column::TimestampTZ(timestamp_unit, timestamp_zone, &timestamp_values),
    ];
    let result = ProvableQueryResult::new(2, &columns);
    assert_eq!(result.num_columns(), columns.len());
    assert_eq!(result.table_length(), 2);

    let fields = vec![
        ColumnField::new("bool_col".into(), ColumnType::Boolean),
        ColumnField::new("u8_col".into(), ColumnType::Uint8),
        ColumnField::new("tiny_col".into(), ColumnType::TinyInt),
        ColumnField::new("small_col".into(), ColumnType::SmallInt),
        ColumnField::new("int_col".into(), ColumnType::Int),
        ColumnField::new("big_col".into(), ColumnType::BigInt),
        ColumnField::new("i128_col".into(), ColumnType::Int128),
        ColumnField::new("varchar_col".into(), ColumnType::VarChar),
        ColumnField::new("binary_col".into(), ColumnType::VarBinary),
        ColumnField::new("scalar_col".into(), ColumnType::Scalar),
        ColumnField::new("decimal_col".into(), ColumnType::Decimal75(precision, 2)),
        ColumnField::new(
            "timestamp_col".into(),
            ColumnType::TimestampTZ(timestamp_unit, timestamp_zone),
        ),
    ];

    let expected = owned_table::<TestScalar>(vec![
        boolean("bool_col", [true, false]),
        uint8("u8_col", [7_u8, 9]),
        tinyint("tiny_col", [-3_i8, 4]),
        smallint("small_col", [-30_i16, 40]),
        int("int_col", [-300_i32, 400]),
        bigint("big_col", [-3_000_i64, 4_000]),
        int128("i128_col", [-30_000_i128, 40_000]),
        varchar("varchar_col", varchar_values),
        varbinary("binary_col", [b"left".to_vec(), b"right".to_vec()]),
        scalar("scalar_col", scalar_values),
        decimal75("decimal_col", 20, 2, decimal_values),
        timestamptz(
            "timestamp_col",
            timestamp_unit,
            timestamp_zone,
            timestamp_values,
        ),
    ]);

    assert_eq!(result.to_owned_table(&fields).unwrap(), expected);
}

#[test]
fn no_arrow_evaluate_hashes_variable_width_columns() {
    let varchar_values = ["alpha", "beta"];
    let varchar_scalars = varchar_values
        .iter()
        .map(|value| TestScalar::from(*value))
        .collect::<Vec<_>>();
    let binary_values = [b"first".as_slice(), b"second".as_slice()];
    let binary_scalars = binary_values
        .iter()
        .map(|value| TestScalar::from_byte_slice_via_hash(value))
        .collect::<Vec<_>>();
    let columns = vec![
        Column::VarChar((&varchar_values, &varchar_scalars)),
        Column::VarBinary((&binary_values, &binary_scalars)),
    ];
    let result = ProvableQueryResult::new(2, &columns);
    let evaluation_point = [TestScalar::from(3), TestScalar::from(5)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);

    let fields = [
        ColumnField::new("varchar_col".into(), ColumnType::VarChar),
        ColumnField::new("binary_col".into(), ColumnType::VarBinary),
    ];
    let evaluations = result.evaluate(&evaluation_point, 2, &fields).unwrap();

    let expected_varchar =
        evaluation_vec[0] * varchar_scalars[0] + evaluation_vec[1] * varchar_scalars[1];
    let expected_binary =
        evaluation_vec[0] * binary_scalars[0] + evaluation_vec[1] * binary_scalars[1];
    assert_eq!(evaluations, vec![expected_varchar, expected_binary]);
}

#[test]
fn no_arrow_evaluate_reports_malformed_result_metadata() {
    let columns: Vec<Column<TestScalar>> = vec![Column::BigInt(&[10_i64, 12])];
    let mut result = ProvableQueryResult::new(2, &columns);
    let evaluation_point = [TestScalar::from(3), TestScalar::from(5)];
    let fields = [ColumnField::new("big_col".into(), ColumnType::BigInt)];

    *result.num_columns_mut() = 2;
    assert!(matches!(
        result.evaluate(&evaluation_point, 2, &fields),
        Err(QueryError::InvalidColumnCount)
    ));

    *result.num_columns_mut() = 1;
    result.data_mut().push(0);
    assert!(matches!(
        result.evaluate(&evaluation_point, 2, &fields),
        Err(QueryError::MiscellaneousEvaluationError)
    ));
}
