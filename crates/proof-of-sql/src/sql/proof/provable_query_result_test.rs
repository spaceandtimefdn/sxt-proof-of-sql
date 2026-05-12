use super::{ProvableQueryResult, QueryError};
use crate::base::scalar::test_scalar::TestScalar;
use crate::{
    base::{
        database::{owned_table_utility, table_utility, Column, ColumnField, ColumnType},
        math::decimal::Precision,
        polynomial::compute_evaluation_vector,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{Scalar, ScalarExt},
    },
    // proof_primitive::inner_product::TestScalar,
};
#[cfg(feature = "arrow")]
use alloc::sync::Arc;
#[cfg(feature = "arrow")]
use arrow::{
    array::{Decimal128Array, Decimal256Array, Int64Array, LargeBinaryArray, StringArray},
    datatypes::{i256, Field, Schema},
    record_batch::RecordBatch,
};
use bumpalo::Bump;
use num_traits::Zero;

#[cfg(feature = "arrow")]
#[test]
fn we_can_convert_an_empty_provable_result_to_a_final_result() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[0_i64; 0])];
    let res = ProvableQueryResult::new(0, &cols);
    let column_fields = vec![ColumnField::new("a1".into(), ColumnType::BigInt)];
    let res =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_evaluate_result_columns_as_mles() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, -12])];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);

    let column_fields = vec![ColumnField::new("a".into(), ColumnType::BigInt); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();
    let expected_evals = [
        TestScalar::from(10u64) * evaluation_vec[0] - TestScalar::from(12u64) * evaluation_vec[1]
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_result_columns_with_no_rows() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[0; 0])];
    let res = ProvableQueryResult::new(0, &cols);
    let evaluation_point = [];
    let mut evaluation_vec = [TestScalar::ZERO; 0];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::BigInt); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 0, &column_fields[..])
        .unwrap();
    let expected_evals = [TestScalar::zero()];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles() {
    let cols: [Column<TestScalar>; 2] = [Column::BigInt(&[10, 12]), Column::BigInt(&[5, 9])];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::BigInt); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();
    let expected_evals = [
        TestScalar::from(10u64) * evaluation_vec[0] + TestScalar::from(12u64) * evaluation_vec[1],
        TestScalar::from(5u64) * evaluation_vec[0] + TestScalar::from(9u64) * evaluation_vec[1],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_128_bits() {
    let cols: [Column<TestScalar>; 2] = [Column::Int128(&[10, 12]), Column::Int128(&[5, 9])];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::Int128); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();
    let expected_evals = [
        TestScalar::from(10u64) * evaluation_vec[0] + TestScalar::from(12u64) * evaluation_vec[1],
        TestScalar::from(5u64) * evaluation_vec[0] + TestScalar::from(9u64) * evaluation_vec[1],
    ];
    assert_eq!(evals, expected_evals);
}

#[expect(clippy::similar_names)]
#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_scalar_columns() {
    let col0 = [10, 12]
        .iter()
        .map(|v| TestScalar::from(*v))
        .collect::<Vec<_>>();
    let col1 = [5, 9]
        .iter()
        .map(|v| TestScalar::from(*v))
        .collect::<Vec<_>>();
    let cols: [Column<TestScalar>; 2] = [Column::Scalar(&col0), Column::Scalar(&col1)];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::Scalar); cols.len()];
    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();
    let expected_evals = [
        TestScalar::from(10u64) * evaluation_vec[0] + TestScalar::from(12u64) * evaluation_vec[1],
        TestScalar::from(5u64) * evaluation_vec[0] + TestScalar::from(9u64) * evaluation_vec[1],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_multiple_result_columns_as_mles_with_mixed_data_types() {
    let cols: [Column<TestScalar>; 2] = [Column::BigInt(&[10, 12]), Column::Int128(&[5, 9])];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = [
        ColumnField::new("a".into(), ColumnType::BigInt),
        ColumnField::new("a".into(), ColumnType::Int128),
    ];
    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();
    let expected_evals = [
        TestScalar::from(10u64) * evaluation_vec[0] + TestScalar::from(12u64) * evaluation_vec[1],
        TestScalar::from(5u64) * evaluation_vec[0] + TestScalar::from(9u64) * evaluation_vec[1],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_evaluate_non_numeric_result_columns_as_mles() {
    let boolean_values = [true, false];
    let uint8_values = [1_u8, 2];
    let tinyint_values = [3_i8, 4];
    let smallint_values = [5_i16, 6];
    let varchar_values = ["alpha", "beta"];
    let varchar_scalars = varchar_values
        .iter()
        .map(|v| TestScalar::from(*v))
        .collect::<Vec<_>>();
    let varbinary_values = [b"foo".as_ref(), b"bar".as_ref()];
    let varbinary_scalars = varbinary_values
        .iter()
        .map(|v| TestScalar::from_byte_slice_via_hash(v))
        .collect::<Vec<_>>();
    let timestamp_values = [7_i64, 8];
    let cols: [Column<TestScalar>; 7] = [
        Column::Boolean(&boolean_values),
        Column::Uint8(&uint8_values),
        Column::TinyInt(&tinyint_values),
        Column::SmallInt(&smallint_values),
        Column::VarChar((&varchar_values, &varchar_scalars)),
        Column::VarBinary((&varbinary_values, &varbinary_scalars)),
        Column::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &timestamp_values,
        ),
    ];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = [
        ColumnField::new("boolean_col".into(), ColumnType::Boolean),
        ColumnField::new("uint8_col".into(), ColumnType::Uint8),
        ColumnField::new("tinyint_col".into(), ColumnType::TinyInt),
        ColumnField::new("smallint_col".into(), ColumnType::SmallInt),
        ColumnField::new("varchar_col".into(), ColumnType::VarChar),
        ColumnField::new("varbinary_col".into(), ColumnType::VarBinary),
        ColumnField::new(
            "timestamp_col".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        ),
    ];
    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();
    let expected_evals = [
        TestScalar::from(1u64) * evaluation_vec[0] + TestScalar::ZERO * evaluation_vec[1],
        TestScalar::from(1u64) * evaluation_vec[0] + TestScalar::from(2u64) * evaluation_vec[1],
        TestScalar::from(3u64) * evaluation_vec[0] + TestScalar::from(4u64) * evaluation_vec[1],
        TestScalar::from(5u64) * evaluation_vec[0] + TestScalar::from(6u64) * evaluation_vec[1],
        varchar_scalars[0] * evaluation_vec[0] + varchar_scalars[1] * evaluation_vec[1],
        varbinary_scalars[0] * evaluation_vec[0] + varbinary_scalars[1] * evaluation_vec[1],
        TestScalar::from(7u64) * evaluation_vec[0] + TestScalar::from(8u64) * evaluation_vec[1],
    ];
    assert_eq!(evals, expected_evals);
}

#[test]
fn evaluation_fails_if_extra_data_is_included() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let mut res = ProvableQueryResult::new(2, &cols);
    res.data_mut().push(3u8);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::BigInt); cols.len()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 2, &column_fields[..]),
        Err(QueryError::MiscellaneousEvaluationError)
    ));
}

#[test]
fn evaluation_fails_if_the_result_cant_be_decoded() {
    let mut res = ProvableQueryResult::new_from_raw_data(1, 1, vec![0b1111_1111_u8; 38]);
    res.data_mut()[37] = 0b0000_0001_u8;
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::BigInt); res.num_columns()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 2, &column_fields[..]),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn evaluation_fails_if_integer_overflow_happens() {
    let binding = [i64::from(i32::MAX) + 1_i64, 12];
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&binding)];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::Int); res.num_columns()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 2, &column_fields[..]),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn evaluation_fails_if_data_is_missing() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let mut res = ProvableQueryResult::new(2, &cols);
    *res.num_columns_mut() = 3;
    let evaluation_point = [TestScalar::from(10u64), TestScalar::from(100u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = vec![ColumnField::new("a".into(), ColumnType::BigInt); res.num_columns()];
    assert!(matches!(
        res.evaluate(&evaluation_point, 2, &column_fields[..]),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn evaluation_fails_if_column_count_differs() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let res = ProvableQueryResult::new(2, &cols);
    assert!(matches!(
        res.evaluate(&[TestScalar::from(1), TestScalar::from(2)], 2, &[]),
        Err(QueryError::InvalidColumnCount)
    ));
}

#[test]
fn to_owned_table_fails_if_column_count_differs() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let res = ProvableQueryResult::new(2, &cols);
    assert!(matches!(
        res.to_owned_table::<TestScalar>(&[]),
        Err(QueryError::InvalidColumnCount)
    ));
}

#[test]
fn to_owned_table_fails_if_data_is_missing() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let mut res = ProvableQueryResult::new(2, &cols);
    res.data_mut().pop();
    let column_fields = [ColumnField::new("a".into(), ColumnType::BigInt)];
    assert!(matches!(
        res.to_owned_table::<TestScalar>(&column_fields),
        Err(QueryError::Overflow)
    ));
}

#[test]
fn we_can_convert_all_supported_types_to_owned_table_without_arrow() {
    let boolean_values = [true, false];
    let uint8_values = [1_u8, 2];
    let tinyint_values = [-1_i8, 2];
    let smallint_values = [-3_i16, 4];
    let int_values = [-5_i32, 6];
    let bigint_values = [-7_i64, 8];
    let int128_values = [-9_i128, 10];
    let varchar_values = ["alpha", "beta"];
    let varchar_scalars = varchar_values
        .iter()
        .map(|v| TestScalar::from(*v))
        .collect::<Vec<_>>();
    let varbinary_values = [b"foo".as_ref(), b"bar".as_ref()];
    let varbinary_scalars = varbinary_values
        .iter()
        .map(|v| TestScalar::from_le_bytes_mod_order(v))
        .collect::<Vec<_>>();
    let scalar_values = [TestScalar::from(11), TestScalar::from(12)];
    let decimal_values = [TestScalar::from(13), TestScalar::from(14)];
    let timestamp_values = [15_i64, 16];

    let cols: [Column<TestScalar>; 12] = [
        Column::Boolean(&boolean_values),
        Column::Uint8(&uint8_values),
        Column::TinyInt(&tinyint_values),
        Column::SmallInt(&smallint_values),
        Column::Int(&int_values),
        Column::BigInt(&bigint_values),
        Column::Int128(&int128_values),
        Column::VarChar((&varchar_values, &varchar_scalars)),
        Column::VarBinary((&varbinary_values, &varbinary_scalars)),
        Column::Scalar(&scalar_values),
        Column::Decimal75(Precision::new(75).unwrap(), -2, &decimal_values),
        Column::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &timestamp_values,
        ),
    ];
    let res = ProvableQueryResult::new(2, &cols);
    let column_fields = vec![
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
            ColumnType::Decimal75(Precision::new(75).unwrap(), -2),
        ),
        ColumnField::new(
            "timestamp_col".into(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        ),
    ];
    let table = res.to_owned_table::<TestScalar>(&column_fields).unwrap();
    let expected = owned_table_utility::owned_table::<TestScalar>([
        owned_table_utility::boolean("bool_col", [true, false]),
        owned_table_utility::uint8("u8_col", [1_u8, 2]),
        owned_table_utility::tinyint("i8_col", [-1_i8, 2]),
        owned_table_utility::smallint("i16_col", [-3_i16, 4]),
        owned_table_utility::int("i32_col", [-5_i32, 6]),
        owned_table_utility::bigint("i64_col", [-7_i64, 8]),
        owned_table_utility::int128("i128_col", [-9_i128, 10]),
        owned_table_utility::varchar("varchar_col", ["alpha", "beta"]),
        owned_table_utility::varbinary("varbinary_col", [b"foo".to_vec(), b"bar".to_vec()]),
        owned_table_utility::scalar("scalar_col", [TestScalar::from(11), TestScalar::from(12)]),
        owned_table_utility::decimal75(
            "decimal_col",
            75,
            -2,
            [TestScalar::from(13), TestScalar::from(14)],
        ),
        owned_table_utility::timestamptz(
            "timestamp_col",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [15_i64, 16],
        ),
    ]);

    assert_eq!(res.table_length(), 2);
    assert_eq!(table, expected);
}

#[test]
fn we_can_form_a_provable_result_from_a_table() {
    let alloc = Bump::new();
    let table = table_utility::table::<TestScalar>([
        table_utility::borrowed_bigint("bigint", [1_i64, -2], &alloc),
        table_utility::borrowed_varchar("varchar", ["alpha", "beta"], &alloc),
    ]);
    let fields = table.schema();
    let res = ProvableQueryResult::from(table);
    let table = res.to_owned_table::<TestScalar>(&fields).unwrap();
    let expected = owned_table_utility::owned_table::<TestScalar>([
        owned_table_utility::bigint("bigint", [1_i64, -2]),
        owned_table_utility::varchar("varchar", ["alpha", "beta"]),
    ]);

    assert_eq!(table, expected);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_convert_a_provable_result_to_a_final_result() {
    let cols: [Column<TestScalar>; 1] = [Column::BigInt(&[10, 12])];
    let res = ProvableQueryResult::new(2, &cols);
    let column_fields = vec![ColumnField::new("a1".into(), ColumnType::BigInt)];
    let res =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![10, 12]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_128_bits() {
    let cols: [Column<TestScalar>; 1] = [Column::Int128(&[10, i128::MAX])];
    let res = ProvableQueryResult::new(2, &cols);
    let column_fields = vec![ColumnField::new("a1".into(), ColumnType::Int128)];
    let res =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res = RecordBatch::try_new(
        schema,
        vec![Arc::new(
            Decimal128Array::from(vec![10, i128::MAX])
                .with_precision_and_scale(38, 0)
                .unwrap(),
        )],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_252_bits() {
    let values = [TestScalar::from(10), TestScalar::MAX_SIGNED];

    let cols: [Column<TestScalar>; 1] = [Column::Scalar(&values)];
    let res = ProvableQueryResult::new(2, &cols);
    let column_fields = vec![ColumnField::new(
        "a1".into(),
        ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
    )];
    let res =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));

    let expected_res = RecordBatch::try_new(
        schema,
        vec![Arc::new(
            Decimal256Array::from([i256::from(10), TestScalar::MAX_SIGNED.into()].to_vec())
                .with_precision_and_scale(75, 0)
                .unwrap(),
        )],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_mixed_data_types() {
    let values1: [i64; 2] = [6, i64::MAX];
    let values2: [i128; 2] = [10, i128::MAX];
    let values3 = ["abc", "de"];
    let scalars3 = values3
        .iter()
        .map(|v| TestScalar::from(*v))
        .collect::<Vec<_>>();
    let values4 = [TestScalar::from(10), TestScalar::MAX_SIGNED];

    let cols: [Column<TestScalar>; 4] = [
        Column::BigInt(&values1),
        Column::Int128(&values2),
        Column::VarChar((&values3, &scalars3)),
        Column::Scalar(&values4),
    ];
    let res = ProvableQueryResult::new(2, &cols);
    let column_fields = vec![
        ColumnField::new("a1".into(), ColumnType::BigInt),
        ColumnField::new("a2".into(), ColumnType::Int128),
        ColumnField::new("a3".into(), ColumnType::VarChar),
        ColumnField::new(
            "a4".into(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
        ),
    ];
    let res =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![6, i64::MAX])),
            Arc::new(
                Decimal128Array::from(vec![10, i128::MAX])
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            Arc::new(StringArray::from(vec!["abc", "de"])),
            Arc::new(
                Decimal256Array::from(vec![i256::from(10), TestScalar::MAX_SIGNED.into()])
                    .with_precision_and_scale(75, 0)
                    .unwrap(),
            ),
        ],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_convert_a_provable_result_to_a_final_result_with_varbinary() {
    let raw_bytes = [b"foo".as_ref(), b"bar"];
    let scalars: Vec<TestScalar> = raw_bytes
        .iter()
        .map(|b| TestScalar::from_le_bytes_mod_order(b))
        .collect();
    let col = Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));
    let res = ProvableQueryResult::new(2, &[col]);
    let column_fields = vec![ColumnField::new("vb_col".into(), ColumnType::VarBinary)];

    let record_batch =
        RecordBatch::try_from(res.to_owned_table::<TestScalar>(&column_fields).unwrap()).unwrap();

    let schema = Arc::new(Schema::new(vec![Field::new(
        "vb_col",
        arrow::datatypes::DataType::LargeBinary,
        false,
    )]));
    let expected = RecordBatch::try_new(
        schema,
        vec![Arc::new(LargeBinaryArray::from_vec(vec![
            b"foo".as_slice(),
            b"bar".as_slice(),
        ]))],
    )
    .unwrap();

    assert_eq!(record_batch, expected);
}
