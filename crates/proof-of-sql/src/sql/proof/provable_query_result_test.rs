use super::{ProvableQueryResult, QueryError};
use crate::base::scalar::test_scalar::TestScalar;
use crate::{
    base::{
        database::{Column, ColumnField, ColumnType, OwnedColumn},
        math::decimal::Precision,
        polynomial::compute_evaluation_vector,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{Scalar, ScalarExt},
    },
    // proof_primitive::inner_product::TestScalar,
};
use alloc::sync::Arc;
use arrow::{
    array::{Decimal128Array, Decimal256Array, Int64Array, LargeBinaryArray, StringArray},
    datatypes::{i256, Field, Schema},
    record_batch::RecordBatch,
};
use num_traits::Zero;

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
fn we_can_evaluate_string_and_binary_result_columns_as_mles() {
    let string_values = ["alpha", "beta"];
    let string_scalars = string_values
        .iter()
        .map(|value| TestScalar::from(*value))
        .collect::<Vec<_>>();
    let binary_values = [b"left".as_slice(), b"right".as_slice()];
    let binary_scalars = binary_values
        .iter()
        .map(|value| TestScalar::from_byte_slice_via_hash(value))
        .collect::<Vec<_>>();
    let cols: [Column<TestScalar>; 2] = [
        Column::VarChar((&string_values, &string_scalars)),
        Column::VarBinary((&binary_values, &binary_scalars)),
    ];
    let res = ProvableQueryResult::new(2, &cols);
    let evaluation_point = [TestScalar::from(7u64), TestScalar::from(11u64)];
    let mut evaluation_vec = [TestScalar::ZERO; 2];
    compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
    let column_fields = [
        ColumnField::new("text_col".into(), ColumnType::VarChar),
        ColumnField::new("bytes_col".into(), ColumnType::VarBinary),
    ];

    let evals = res
        .evaluate(&evaluation_point, 2, &column_fields[..])
        .unwrap();

    let expected_evals = [
        string_scalars[0] * evaluation_vec[0] + string_scalars[1] * evaluation_vec[1],
        binary_scalars[0] * evaluation_vec[0] + binary_scalars[1] * evaluation_vec[1],
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

#[test]
fn we_can_convert_remaining_fixed_width_types_to_owned_table() {
    let bools = [true, false];
    let uint8s = [0_u8, u8::MAX];
    let tinyints = [i8::MIN, i8::MAX];
    let smallints = [i16::MIN, i16::MAX];
    let ints = [i32::MIN, i32::MAX];
    let timestamps = [1_717_171_717_i64, 1_818_181_818_i64];
    let time_unit = PoSQLTimeUnit::Millisecond;
    let time_zone = PoSQLTimeZone::new(3600);
    let cols: [Column<TestScalar>; 6] = [
        Column::Boolean(&bools),
        Column::Uint8(&uint8s),
        Column::TinyInt(&tinyints),
        Column::SmallInt(&smallints),
        Column::Int(&ints),
        Column::TimestampTZ(time_unit, time_zone, &timestamps),
    ];
    let res = ProvableQueryResult::new(2, &cols);
    let column_fields = vec![
        ColumnField::new("bool_col".into(), ColumnType::Boolean),
        ColumnField::new("uint8_col".into(), ColumnType::Uint8),
        ColumnField::new("tinyint_col".into(), ColumnType::TinyInt),
        ColumnField::new("smallint_col".into(), ColumnType::SmallInt),
        ColumnField::new("int_col".into(), ColumnType::Int),
        ColumnField::new(
            "timestamp_col".into(),
            ColumnType::TimestampTZ(time_unit, time_zone),
        ),
    ];

    let owned_table = res.to_owned_table::<TestScalar>(&column_fields).unwrap();

    assert_eq!(
        owned_table["bool_col"],
        OwnedColumn::Boolean(bools.to_vec())
    );
    assert_eq!(
        owned_table["uint8_col"],
        OwnedColumn::Uint8(uint8s.to_vec())
    );
    assert_eq!(
        owned_table["tinyint_col"],
        OwnedColumn::TinyInt(tinyints.to_vec())
    );
    assert_eq!(
        owned_table["smallint_col"],
        OwnedColumn::SmallInt(smallints.to_vec())
    );
    assert_eq!(owned_table["int_col"], OwnedColumn::Int(ints.to_vec()));
    assert_eq!(
        owned_table["timestamp_col"],
        OwnedColumn::TimestampTZ(time_unit, time_zone, timestamps.to_vec())
    );
}

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
