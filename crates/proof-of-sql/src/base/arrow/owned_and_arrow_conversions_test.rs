use super::owned_and_arrow_conversions::OwnedArrowConversionError;
use crate::base::{
    database::{owned_table_utility::*, OwnedColumn, OwnedTable},
    map::IndexMap,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use alloc::sync::Arc;
use arrow::{
    array::{
        ArrayRef, BooleanArray, Decimal128Array, Decimal256Array, Float32Array, Int16Array,
        Int32Array, Int64Array, Int8Array, LargeBinaryArray, StringArray,
        TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray,
        TimestampSecondArray, UInt8Array,
    },
    datatypes::{i256, DataType, Field, Schema},
    record_batch::RecordBatch,
};
use core::str::FromStr;
use proptest::prelude::*;

fn we_can_convert_between_owned_column_and_array_ref_impl(
    owned_column: &OwnedColumn<TestScalar>,
    array_ref: ArrayRef,
) {
    let ic_to_ar = ArrayRef::from(owned_column.clone());
    let ar_to_ic = OwnedColumn::try_from(array_ref.clone()).unwrap();

    assert!(ic_to_ar == array_ref);
    assert_eq!(*owned_column, ar_to_ic);
}

fn we_can_convert_between_varbinary_owned_column_and_array_ref_impl(data: &[Vec<u8>]) {
    let owned_col = OwnedColumn::<TestScalar>::VarBinary(data.to_owned());
    let arrow_col = Arc::new(LargeBinaryArray::from(
        data.iter()
            .map(std::vec::Vec::as_slice)
            .collect::<Vec<&[u8]>>(),
    ));
    we_can_convert_between_owned_column_and_array_ref_impl(&owned_col, arrow_col);
}

fn we_can_convert_between_boolean_owned_column_and_array_ref_impl(data: Vec<bool>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Boolean(data.clone()),
        Arc::new(BooleanArray::from(data)),
    );
}
fn we_can_convert_between_bigint_owned_column_and_array_ref_impl(data: Vec<i64>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::BigInt(data.clone()),
        Arc::new(Int64Array::from(data)),
    );
}
fn we_can_convert_between_uint8_owned_column_and_array_ref_impl(data: Vec<u8>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Uint8(data.clone()),
        Arc::new(UInt8Array::from(data)),
    );
}
fn we_can_convert_between_tinyint_owned_column_and_array_ref_impl(data: Vec<i8>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::TinyInt(data.clone()),
        Arc::new(Int8Array::from(data)),
    );
}
fn we_can_convert_between_smallint_owned_column_and_array_ref_impl(data: Vec<i16>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::SmallInt(data.clone()),
        Arc::new(Int16Array::from(data)),
    );
}
fn we_can_convert_between_int_owned_column_and_array_ref_impl(data: Vec<i32>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Int(data.clone()),
        Arc::new(Int32Array::from(data)),
    );
}
fn we_can_convert_between_int128_owned_column_and_array_ref_impl(data: Vec<i128>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Int128(data.clone()),
        Arc::new(
            Decimal128Array::from(data)
                .with_precision_and_scale(38, 0)
                .unwrap(),
        ),
    );
}
fn we_can_convert_between_varchar_owned_column_and_array_ref_impl(data: Vec<String>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::VarChar(data.clone()),
        Arc::new(StringArray::from(data)),
    );
}
fn we_can_convert_between_decimal75_owned_column_and_array_ref_impl(data: Vec<i64>) {
    let owned_col = OwnedColumn::<TestScalar>::Decimal75(
        Precision::new(20).unwrap(),
        4,
        data.iter().copied().map(TestScalar::from).collect(),
    );
    let converted = data
        .into_iter()
        .map(|value| i256::from_str(&value.to_string()).unwrap())
        .collect::<Vec<_>>();
    let arrow_col = Arc::new(
        Decimal256Array::from(converted)
            .with_precision_and_scale(20, 4)
            .unwrap(),
    );
    we_can_convert_between_owned_column_and_array_ref_impl(&owned_col, arrow_col);
}
fn we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
    time_unit: PoSQLTimeUnit,
    data: Vec<i64>,
) {
    let owned_col =
        OwnedColumn::<TestScalar>::TimestampTZ(time_unit, PoSQLTimeZone::utc(), data.clone());
    let arrow_col: ArrayRef = match time_unit {
        PoSQLTimeUnit::Second => Arc::new(TimestampSecondArray::from(data)),
        PoSQLTimeUnit::Millisecond => Arc::new(TimestampMillisecondArray::from(data)),
        PoSQLTimeUnit::Microsecond => Arc::new(TimestampMicrosecondArray::from(data)),
        PoSQLTimeUnit::Nanosecond => Arc::new(TimestampNanosecondArray::from(data)),
    };
    we_can_convert_between_owned_column_and_array_ref_impl(&owned_col, arrow_col);
}
#[test]
fn we_can_convert_between_owned_column_and_array_ref() {
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_uint8_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_tinyint_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_smallint_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_int_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_int128_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_decimal75_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Second,
        vec![],
    );
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Millisecond,
        vec![],
    );
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Microsecond,
        vec![],
    );
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Nanosecond,
        vec![],
    );
    let data = vec![true, false, true, false, true, false, true, false, true];
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(data);
    let data = vec![0_u8, 1, 2, 3, 42, u8::MAX];
    we_can_convert_between_uint8_owned_column_and_array_ref_impl(data);
    let data = vec![0_i8, 1, 2, 3, -42, i8::MIN, i8::MAX];
    we_can_convert_between_tinyint_owned_column_and_array_ref_impl(data);
    let data = vec![0_i16, 1, 2, 3, -42, i16::MIN, i16::MAX];
    we_can_convert_between_smallint_owned_column_and_array_ref_impl(data);
    let data = vec![0_i32, 1, 2, 3, -42, i32::MIN, i32::MAX];
    we_can_convert_between_int_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX];
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX];
    we_can_convert_between_int128_owned_column_and_array_ref_impl(data);
    let data = vec![0_i64, 1, -1, 12_345, -67_890];
    we_can_convert_between_decimal75_owned_column_and_array_ref_impl(data);
    let data = vec!["0", "1", "2", "3", "4", "5", "6"];
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(
        data.into_iter().map(String::from).collect(),
    );
    let data = vec![0_i64, 1, -1, 1_625_072_400];
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Second,
        data.clone(),
    );
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Millisecond,
        data.clone(),
    );
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Microsecond,
        data.clone(),
    );
    we_can_convert_between_timestamptz_owned_column_and_array_ref_impl(
        PoSQLTimeUnit::Nanosecond,
        data,
    );

    let varbin_data = vec![
        b"foo".to_vec(),
        b"bar".to_vec(),
        b"baz".to_vec(),
        vec![],
        b"some bytes".to_vec(),
    ];
    we_can_convert_between_varbinary_owned_column_and_array_ref_impl(&varbin_data);
}

#[test]
fn we_get_an_unsupported_type_error_when_trying_to_convert_from_a_float32_array_ref_to_an_owned_column(
) {
    let array_ref: ArrayRef = Arc::new(Float32Array::from(vec![0.0]));
    assert!(matches!(
        OwnedColumn::<TestScalar>::try_from(array_ref),
        Err(OwnedArrowConversionError::UnsupportedType { .. })
    ));
}

#[test]
fn we_get_a_null_not_supported_error_when_trying_to_convert_from_a_nullable_boolean_array_ref_to_an_owned_column(
) {
    let array_ref: ArrayRef = Arc::new(BooleanArray::from(vec![Some(true), None]));
    assert!(matches!(
        OwnedColumn::<TestScalar>::try_from(array_ref),
        Err(OwnedArrowConversionError::NullNotSupportedYet)
    ));
}

fn we_can_convert_between_owned_table_and_record_batch_impl(
    owned_table: &OwnedTable<TestScalar>,
    record_batch: &RecordBatch,
) {
    let it_to_rb = RecordBatch::try_from(owned_table.clone()).unwrap();
    let rb_to_it = OwnedTable::try_from(record_batch.clone()).unwrap();

    assert_eq!(it_to_rb, *record_batch);
    assert_eq!(rb_to_it, *owned_table);
}
#[test]
fn we_can_convert_between_owned_table_and_record_batch() {
    we_can_convert_between_owned_table_and_record_batch_impl(
        &OwnedTable::<TestScalar>::try_new(IndexMap::default()).unwrap(),
        &RecordBatch::new_empty(Arc::new(Schema::empty())),
    );

    let schema = Arc::new(Schema::new(vec![
        Field::new("int64", DataType::Int64, false),
        Field::new("int128", DataType::Decimal128(38, 0), false),
        Field::new("string", DataType::Utf8, false),
        Field::new("boolean", DataType::Boolean, false),
    ]));

    let batch1 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![0_i64; 0])),
            Arc::new(
                Decimal128Array::from(vec![0_i128; 0])
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            Arc::new(StringArray::from(vec!["0"; 0])),
            Arc::new(BooleanArray::from(vec![true; 0])),
        ],
    )
    .unwrap();

    we_can_convert_between_owned_table_and_record_batch_impl(
        &owned_table([
            bigint("int64", [0; 0]),
            int128("int128", [0; 0]),
            varchar("string", ["0"; 0]),
            boolean("boolean", [true; 0]),
        ]),
        &batch1,
    );

    let batch2 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                i64::MIN,
                i64::MAX,
            ])),
            Arc::new(
                Decimal128Array::from(vec![0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX])
                    .with_precision_and_scale(38, 0)
                    .unwrap(),
            ),
            Arc::new(StringArray::from(vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8",
            ])),
            Arc::new(BooleanArray::from(vec![
                true, false, true, false, true, false, true, false, true,
            ])),
        ],
    )
    .unwrap();

    we_can_convert_between_owned_table_and_record_batch_impl(
        &owned_table([
            bigint("int64", [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
            int128("int128", [0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
            varchar("string", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
            boolean(
                "boolean",
                [true, false, true, false, true, false, true, false, true],
            ),
        ]),
        &batch2,
    );

    let complex_table = owned_table::<TestScalar>([
        uint8("uint8", [0_u8, 1, 2, u8::MAX]),
        tinyint("tinyint", [0_i8, -1, 42, i8::MAX]),
        smallint("smallint", [0_i16, -1, 42, i16::MAX]),
        int("int32", [0_i32, -1, 42, i32::MAX]),
        varbinary(
            "bytes",
            [
                b"foo".to_vec(),
                b"".to_vec(),
                b"bar".to_vec(),
                vec![0, 1, 2],
            ],
        ),
        decimal75("decimal", 20, 4, [0_i64, -1, 12_345, -67_890]),
        timestamptz(
            "ts_ms",
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
            [0_i64, 1, -1, 1_625_072_400],
        ),
        timestamptz(
            "ts_ns",
            PoSQLTimeUnit::Nanosecond,
            PoSQLTimeZone::utc(),
            [0_i64, 1, -1, 1_625_072_400],
        ),
    ]);
    let complex_batch = RecordBatch::try_from(complex_table.clone()).unwrap();
    let complex_roundtrip = OwnedTable::try_from(complex_batch.clone()).unwrap();

    assert_eq!(complex_roundtrip, complex_table);
    assert_eq!(complex_batch.num_columns(), 8);
}

#[test]
#[should_panic(expected = "not implemented: Cannot convert Scalar type to arrow type")]
fn we_panic_when_converting_an_owned_table_with_a_scalar_column() {
    let owned_table = owned_table::<TestScalar>([scalar("a", [0; 0])]);
    let _ = RecordBatch::try_from(owned_table);
}

#[test]
fn we_get_a_duplicate_ident_error_when_converting_a_record_batch_with_case_conflicting_columns() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("dup", DataType::Int64, false),
        Field::new("dup", DataType::Int64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![1_i64, 2])),
            Arc::new(Int64Array::from(vec![3_i64, 4])),
        ],
    )
    .unwrap();

    assert!(matches!(
        OwnedTable::<TestScalar>::try_from(batch),
        Err(OwnedArrowConversionError::DuplicateIdents)
    ));
}

proptest! {
    #[test]
    fn we_can_roundtrip_arbitrary_owned_column(owned_column: OwnedColumn<TestScalar>) {
        let arrow = ArrayRef::from(owned_column.clone());
        let actual = OwnedColumn::try_from(arrow).unwrap();

        prop_assert_eq!(actual, owned_column);
    }
}
