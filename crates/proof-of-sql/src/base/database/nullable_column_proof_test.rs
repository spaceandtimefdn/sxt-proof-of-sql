use super::{
    owned_table_utility::*, presence_column_id, ColumnField, ColumnRef, ColumnType,
    NullableOwnedColumn, OwnedColumn, OwnedTableTestAccessor, TableRef,
};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof, math::decimal::Precision,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::VerifiableQueryResult,
        proof_exprs::{test_utility::*, DynProofExpr},
        proof_plans::test_utility::*,
    },
};
use alloc::vec::Vec;

#[test]
#[expect(clippy::too_many_lines)]
fn we_can_prove_nullable_bigint_addition_with_presence_columns() {
    let nullable_score = nullable_bigint([Some(5_i64), None, Some(9), Some(5), None]);
    let bonus_values = vec![7_i64, 12, 1, 7, 6];
    let bonus =
        NullableOwnedColumn::<TestScalar>::non_nullable(OwnedColumn::BigInt(bonus_values.clone()));
    let nullable_sum = nullable_score.element_wise_add(&bonus).unwrap();

    assert_eq!(
        nullable_sum.values(),
        &OwnedColumn::BigInt(vec![12, 0, 10, 12, 0])
    );
    assert_eq!(
        nullable_sum.presence(),
        Some([true, false, true, true, false].as_slice())
    );

    let data = owned_table([
        nullable_score.value_owned_column("score_value"),
        nullable_score
            .presence_owned_column("score_present")
            .unwrap(),
        bonus.value_owned_column("bonus_value"),
        nullable_sum.value_owned_column("sum_value"),
        nullable_sum.presence_owned_column("sum_present").unwrap(),
    ]);
    let t = TableRef::new("sxt", "nullable_scores");
    let accessor =
        OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), data, 0, ());

    let score_plus_bonus = add(
        column(&t, "score_value", &accessor),
        column(&t, "bonus_value", &accessor),
    );
    let sum_value_as_decimal = cast(
        column(&t, "sum_value", &accessor),
        ColumnType::Decimal75(Precision::new(20).unwrap(), 0),
    );
    let where_clause = and(
        column(&t, "score_present", &accessor),
        and(
            equal(
                column(&t, "score_present", &accessor),
                column(&t, "sum_present", &accessor),
            ),
            and(
                equal(score_plus_bonus.clone(), sum_value_as_decimal),
                equal(score_plus_bonus.clone(), const_decimal75(20, 0, 12_i128)),
            ),
        ),
    );
    let plan = filter(
        vec![
            aliased_plan(score_plus_bonus, "score_plus_bonus"),
            col_expr_plan(&t, "sum_value", &accessor),
            col_expr_plan(&t, "score_present", &accessor),
            col_expr_plan(&t, "sum_present", &accessor),
        ],
        table_exec(
            t.clone(),
            vec![
                column_field("score_value", ColumnType::BigInt),
                column_field("score_present", ColumnType::Boolean),
                column_field("bonus_value", ColumnType::BigInt),
                column_field("sum_value", ColumnType::BigInt),
                column_field("sum_present", ColumnType::Boolean),
            ],
        ),
        where_clause,
    );

    let mismatched_presence_plan = filter(
        vec![
            col_expr_plan(&t, "score_present", &accessor),
            col_expr_plan(&t, "sum_present", &accessor),
        ],
        table_exec(
            t.clone(),
            vec![
                column_field("score_value", ColumnType::BigInt),
                column_field("score_present", ColumnType::Boolean),
                column_field("bonus_value", ColumnType::BigInt),
                column_field("sum_value", ColumnType::BigInt),
                column_field("sum_present", ColumnType::Boolean),
            ],
        ),
        not(equal(
            column(&t, "score_present", &accessor),
            column(&t, "sum_present", &accessor),
        )),
    );
    let mismatched_presence_res = VerifiableQueryResult::<NaiveEvaluationProof>::new(
        &mismatched_presence_plan,
        &accessor,
        &(),
        &[],
    )
    .unwrap();
    let mismatched_presence_table = mismatched_presence_res
        .verify(&mismatched_presence_plan, &accessor, &(), &[])
        .unwrap()
        .table;
    assert_eq!(
        mismatched_presence_table,
        owned_table([
            boolean("score_present", core::iter::empty::<bool>()),
            boolean("sum_present", core::iter::empty::<bool>()),
        ])
    );

    let tampered_sum = NullableOwnedColumn::<TestScalar>::try_new(
        OwnedColumn::BigInt(vec![12, 0, 10, 13, 0]),
        nullable_sum.presence().map(<[bool]>::to_vec),
    )
    .unwrap();
    let tampered_sum_accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        t.clone(),
        owned_table([
            nullable_score.value_owned_column("score_value"),
            nullable_score
                .presence_owned_column("score_present")
                .unwrap(),
            bonus.value_owned_column("bonus_value"),
            tampered_sum.value_owned_column("sum_value"),
            tampered_sum.presence_owned_column("sum_present").unwrap(),
        ]),
        0,
        (),
    );
    let tampered_sum_res =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    assert!(tampered_sum_res
        .verify(&plan, &tampered_sum_accessor, &(), &[])
        .is_err());

    let tampered_sum_presence = NullableOwnedColumn::<TestScalar>::try_new(
        OwnedColumn::BigInt(vec![12, 0, 10, 0, 0]),
        Some(vec![true, false, true, false, false]),
    )
    .unwrap();
    let tampered_presence_accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        t.clone(),
        owned_table([
            nullable_score.value_owned_column("score_value"),
            nullable_score
                .presence_owned_column("score_present")
                .unwrap(),
            bonus.value_owned_column("bonus_value"),
            tampered_sum_presence.value_owned_column("sum_value"),
            tampered_sum_presence
                .presence_owned_column("sum_present")
                .unwrap(),
        ]),
        0,
        (),
    );
    let tampered_presence_res =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    assert!(tampered_presence_res
        .verify(&plan, &tampered_presence_accessor, &(), &[])
        .is_err());

    let verified_table =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
            .unwrap()
            .verify(&plan, &accessor, &(), &[])
            .unwrap()
            .table;
    assert_eq!(
        verified_table,
        owned_table([
            decimal75("score_plus_bonus", 20, 0, [12_i128, 12]),
            bigint("sum_value", [12_i64, 12]),
            boolean("score_present", [true, true]),
            boolean("sum_present", [true, true]),
        ])
    );
}

#[test]
fn we_can_prove_nullable_expressions_through_projection_and_filter() {
    let nullable_score = nullable_bigint([Some(5_i64), None, Some(9), Some(5), None]);
    let bonus = NullableOwnedColumn::<TestScalar>::non_nullable(OwnedColumn::BigInt(vec![
        7_i64, 0, 1, 7, 0,
    ]));
    let t = TableRef::new("sxt", "nullable_scores");
    let score_presence = presence_column_id(&"score".into());
    let data = owned_table([
        nullable_score.value_owned_column("score"),
        nullable_score
            .presence_owned_column(score_presence.clone())
            .unwrap(),
        bonus.value_owned_column("bonus"),
    ]);
    let accessor =
        OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), data, 0, ());
    let score = DynProofExpr::new_column(ColumnRef::new_nullable(
        t.clone(),
        "score".into(),
        ColumnType::BigInt,
    ));
    let bonus = DynProofExpr::new_column(ColumnRef::new(
        t.clone(),
        "bonus".into(),
        ColumnType::BigInt,
    ));
    let total = add(score.clone(), bonus.clone());
    let table_scan = table_exec(
        t.clone(),
        vec![
            ColumnField::new_nullable("score".into(), ColumnType::BigInt),
            ColumnField::new("bonus".into(), ColumnType::BigInt),
        ],
    );
    let projection_plan = projection(
        vec![
            aliased_plan(total.clone(), "total"),
            aliased_plan(DynProofExpr::new_is_null(score.clone()), "score_is_null"),
        ],
        table_scan.clone(),
    );
    let projection_table =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&projection_plan, &accessor, &(), &[])
            .unwrap()
            .verify(&projection_plan, &accessor, &(), &[])
            .unwrap()
            .table;
    assert_eq!(
        projection_table,
        owned_table([
            decimal75("total", 20, 0, [12_i128, 0, 10, 12, 0]),
            boolean("total__presence", [true, false, true, true, false]),
            boolean("score_is_null", [false, true, false, false, true]),
        ])
    );

    let filter_plan = filter(
        vec![aliased_plan(total.clone(), "total")],
        table_scan,
        equal(total, const_decimal75(20, 0, 12_i128)),
    );
    let filtered_table =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&filter_plan, &accessor, &(), &[])
            .unwrap()
            .verify(&filter_plan, &accessor, &(), &[])
            .unwrap()
            .table;
    assert_eq!(
        filtered_table,
        owned_table([
            decimal75("total", 20, 0, [12_i128, 12]),
            boolean("total__presence", [true, true]),
        ])
    );
}

fn nullable_bigint<const N: usize>(values: [Option<i64>; N]) -> NullableOwnedColumn<TestScalar> {
    let values = values
        .into_iter()
        .map(|value| value.map(TestScalar::from))
        .collect::<Vec<_>>();
    NullableOwnedColumn::try_from_option_scalars(&values, ColumnType::BigInt).unwrap()
}
