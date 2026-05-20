use super::owned_and_arrow_conversions::OwnedArrowConversionError;
use crate::base::{
    database::{owned_table_utility::*, OwnedColumn, OwnedTable},
    map::IndexMap,
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
fn we_can_convert_between_bigint_owned_column_and_array_ref_impl(data: Vec<i64>) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::BigInt(data.clone()),
        Arc::new(Int64Array::from(data)),
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
fn we_can_convert_between_decimal75_owned_column_and_array_ref_impl(
    precision: u8,
    scale: i8,
    data: Vec<i64>,
) {
    we_can_convert_between_owned_column_and_array_ref_impl(
        &OwnedColumn::<TestScalar>::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            data.iter().copied().map(TestScalar::from).collect(),
        ),
        Arc::new(
            Decimal256Array::from(data.into_iter().map(i256::from).collect::<Vec<_>>())
                .with_precision_and_scale(precision, scale)
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
fn we_can_convert_between_timestamp_owned_column_and_array_ref_impl(
    time_unit: crate::base::posql_time::PoSQLTimeUnit,
    data: Vec<i64>,
) {
    let timezone = crate::base::posql_time::PoSQLTimeZone::utc();
    let owned_column = OwnedColumn::<TestScalar>::TimestampTZ(time_unit, timezone, data.clone());
    let array_ref: ArrayRef = match time_unit {
        crate::base::posql_time::PoSQLTimeUnit::Second => {
            Arc::new(TimestampSecondArray::from(data))
        }
        crate::base::posql_time::PoSQLTimeUnit::Millisecond => {
            Arc::new(TimestampMillisecondArray::from(data))
        }
        crate::base::posql_time::PoSQLTimeUnit::Microsecond => {
            Arc::new(TimestampMicrosecondArray::from(data))
        }
        crate::base::posql_time::PoSQLTimeUnit::Nanosecond => {
            Arc::new(TimestampNanosecondArray::from(data))
        }
    };
    we_can_convert_between_owned_column_and_array_ref_impl(&owned_column, array_ref);
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
    we_can_convert_between_decimal75_owned_column_and_array_ref_impl(10, 2, vec![]);
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_timestamp_owned_column_and_array_ref_impl(
        crate::base::posql_time::PoSQLTimeUnit::Second,
        vec![],
    );
    let data = vec![true, false, true, false, true, false, true, false, true];
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(data);
    we_can_convert_between_uint8_owned_column_and_array_ref_impl(vec![0, 1, 2, u8::MAX]);
    we_can_convert_between_tinyint_owned_column_and_array_ref_impl(vec![
        0,
        1,
        -1,
        i8::MIN,
        i8::MAX,
    ]);
    we_can_convert_between_smallint_owned_column_and_array_ref_impl(vec![
        0,
        1,
        -1,
        i16::MIN,
        i16::MAX,
    ]);
    we_can_convert_between_int_owned_column_and_array_ref_impl(vec![0, 1, -1, i32::MIN, i32::MAX]);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX];
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX];
    we_can_convert_between_int128_owned_column_and_array_ref_impl(data);
    we_can_convert_between_decimal75_owned_column_and_array_ref_impl(10, 2, vec![0, 1, -1, 42]);
    let data = vec!["0", "1", "2", "3", "4", "5", "6"];
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(
        data.into_iter().map(String::from).collect(),
    );
    let timestamp_data = vec![0, 1_625_072_400, i64::MAX / 2];
    for time_unit in [
        crate::base::posql_time::PoSQLTimeUnit::Second,
        crate::base::posql_time::PoSQLTimeUnit::Millisecond,
        crate::base::posql_time::PoSQLTimeUnit::Microsecond,
        crate::base::posql_time::PoSQLTimeUnit::Nanosecond,
    ] {
        we_can_convert_between_timestamp_owned_column_and_array_ref_impl(
            time_unit,
            timestamp_data.clone(),
        );
    }

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
}

#[test]
#[should_panic(expected = "not implemented: Cannot convert Scalar type to arrow type")]
fn we_panic_when_converting_an_owned_table_with_a_scalar_column() {
    let owned_table = owned_table::<TestScalar>([scalar("a", [0; 0])]);
    let _ = RecordBatch::try_from(owned_table);
}

proptest! {
    #[test]
    fn we_can_roundtrip_arbitrary_owned_column(owned_column: OwnedColumn<TestScalar>) {
        let arrow = ArrayRef::from(owned_column.clone());
        let actual = OwnedColumn::try_from(arrow).unwrap();

        prop_assert_eq!(actual, owned_column);
    }
}
