use super::owned_and_arrow_conversions::{
    null_presence_column_id, owned_table_from_record_batch_with_nulls, OwnedArrowConversionError,
};
use crate::base::{
    commitment::naive_evaluation_proof::NaiveEvaluationProof,
    database::{
        owned_table_utility::*, ColumnField, ColumnType, OwnedColumn, OwnedTable,
        OwnedTableTestAccessor, TableRef, TestAccessor,
    },
    map::IndexMap,
    scalar::test_scalar::TestScalar,
};
use crate::sql::{
    proof::VerifiableQueryResult, proof_exprs::test_utility::*, proof_plans::test_utility::*,
};
use alloc::sync::Arc;
use arrow::{
    array::{
        ArrayRef, BooleanArray, Decimal128Array, Float32Array, Int64Array, LargeBinaryArray,
        StringArray,
    },
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use proptest::prelude::*;
use sqlparser::ast::Ident;

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
#[test]
fn we_can_convert_between_owned_column_and_array_ref() {
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_int128_owned_column_and_array_ref_impl(vec![]);
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(vec![]);
    let data = vec![true, false, true, false, true, false, true, false, true];
    we_can_convert_between_boolean_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX];
    we_can_convert_between_bigint_owned_column_and_array_ref_impl(data);
    let data = vec![0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX];
    we_can_convert_between_int128_owned_column_and_array_ref_impl(data);
    let data = vec!["0", "1", "2", "3", "4", "5", "6"];
    we_can_convert_between_varchar_owned_column_and_array_ref_impl(
        data.into_iter().map(String::from).collect(),
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
fn we_can_materialize_nullable_arrow_arrays_as_value_and_presence_columns() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("score", DataType::Int64, true),
        Field::new("flag", DataType::Boolean, true),
        Field::new("name", DataType::Utf8, true),
        Field::new("bonus", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![Some(5), None, Some(9)])),
            Arc::new(BooleanArray::from(vec![Some(true), None, Some(false)])),
            Arc::new(StringArray::from(vec![Some("ready"), None, Some("done")])),
            Arc::new(Int64Array::from(vec![7, 7, 1])),
        ],
    )
    .unwrap();

    let actual = owned_table_from_record_batch_with_nulls::<TestScalar>(batch).unwrap();

    assert_eq!(
        null_presence_column_id(&Ident::new("score")),
        "score__presence".into()
    );
    assert_eq!(
        actual,
        owned_table([
            bigint("score", [5_i64, 0, 9]),
            boolean("score__presence", [true, false, true]),
            boolean("flag", [true, false, false]),
            boolean("flag__presence", [true, false, true]),
            varchar("name", ["ready", "", "done"]),
            boolean("name__presence", [true, false, true]),
            bigint("bonus", [7_i64, 7, 1]),
        ])
    );
}

#[test]
fn we_get_duplicate_idents_when_nullable_arrow_presence_column_already_exists() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("score", DataType::Int64, true),
        Field::new("score__presence", DataType::Boolean, false),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![Some(5), None])),
            Arc::new(BooleanArray::from(vec![true, false])),
        ],
    )
    .unwrap();

    assert!(matches!(
        owned_table_from_record_batch_with_nulls::<TestScalar>(batch),
        Err(OwnedArrowConversionError::DuplicateIdents)
    ));
}

#[test]
fn we_can_prove_over_materialized_nullable_arrow_columns() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("score", DataType::Int64, true),
        Field::new("bonus", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![
                Some(5),
                None,
                Some(9),
                Some(5),
                None,
            ])),
            Arc::new(Int64Array::from(vec![7, 99, 1, 7, 6])),
        ],
    )
    .unwrap();
    let table = owned_table_from_record_batch_with_nulls::<TestScalar>(batch).unwrap();

    let t = TableRef::new("sxt", "nullable_scores");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), table, 0);

    let score_plus_bonus = add(
        column(&t, "score", &accessor),
        column(&t, "bonus", &accessor),
    );
    let where_clause = and(
        column(&t, "score__presence", &accessor),
        equal(score_plus_bonus.clone(), const_decimal75(20, 0, 12_i128)),
    );
    let plan = filter(
        vec![
            aliased_plan(score_plus_bonus, "score_plus_bonus"),
            col_expr_plan(&t, "score", &accessor),
            col_expr_plan(&t, "score__presence", &accessor),
        ],
        table_exec(
            t.clone(),
            vec![
                ColumnField::new("score".into(), ColumnType::BigInt),
                ColumnField::new("score__presence".into(), ColumnType::Boolean),
                ColumnField::new("bonus".into(), ColumnType::BigInt),
            ],
        ),
        where_clause,
    );

    let verifiable_result =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let result = verifiable_result
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    assert_eq!(
        result,
        owned_table([
            decimal75("score_plus_bonus", 20, 0, [12_i64, 12]),
            bigint("score", [5_i64, 5]),
            boolean("score__presence", [true, true]),
        ])
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
