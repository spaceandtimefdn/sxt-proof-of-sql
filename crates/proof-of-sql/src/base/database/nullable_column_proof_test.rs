use super::{ColumnType, NullableBigIntColumn, OwnedTableTestAccessor, TableRef};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof, database::owned_table_utility::*,
        math::decimal::Precision,
    },
    sql::{
        proof::VerifiableQueryResult, proof_exprs::test_utility::*, proof_plans::test_utility::*,
    },
};

#[test]
#[expect(clippy::too_many_lines)]
fn we_can_prove_nullable_bigint_poc_with_committed_validity_mask() {
    let nullable_score = NullableBigIntColumn::from_options([Some(5_i64), None, Some(9), Some(5)]);
    let bonus = [7_i64, 12, 1, 7];
    let nullable_sum = nullable_score.try_add_bigint(bonus).unwrap();
    assert_eq!(nullable_sum.values(), &[12, 0, 10, 12]);
    assert_eq!(nullable_sum.validity(), &[true, false, true, true]);

    let data = owned_table([
        nullable_score.value_owned_column("score_value"),
        nullable_score.validity_owned_column("score_valid"),
        nullable_sum.value_owned_column("sum_value"),
        nullable_sum.validity_owned_column("sum_valid"),
        bigint("bonus", bonus),
    ]);
    let t = TableRef::new("sxt", "nullable_scores");
    let accessor =
        OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), data, 0, ());

    let score_plus_bonus = add(
        column(&t, "score_value", &accessor),
        column(&t, "bonus", &accessor),
    );
    let sum_value_as_decimal = cast(
        column(&t, "sum_value", &accessor),
        ColumnType::Decimal75(Precision::new(20).unwrap(), 0),
    );
    let where_clause = and(
        column(&t, "score_valid", &accessor),
        and(
            column(&t, "sum_valid", &accessor),
            and(
                equal(score_plus_bonus.clone(), sum_value_as_decimal),
                equal(score_plus_bonus.clone(), const_decimal75(20, 0, 12_i128)),
            ),
        ),
    );
    let expr = filter(
        vec![
            aliased_plan(score_plus_bonus, "score_plus_bonus"),
            col_expr_plan(&t, "sum_value", &accessor),
            col_expr_plan(&t, "score_valid", &accessor),
            col_expr_plan(&t, "sum_valid", &accessor),
        ],
        table_exec(
            t.clone(),
            vec![
                column_field("score_value", ColumnType::BigInt),
                column_field("score_valid", ColumnType::Boolean),
                column_field("sum_value", ColumnType::BigInt),
                column_field("sum_valid", ColumnType::Boolean),
                column_field("bonus", ColumnType::BigInt),
            ],
        ),
        where_clause,
    );

    let tampered_nullable_score = NullableBigIntColumn::try_new(
        nullable_score.values().to_vec(),
        vec![true, true, true, true],
    )
    .unwrap();
    let tampered_data = owned_table([
        tampered_nullable_score.value_owned_column("score_value"),
        tampered_nullable_score.validity_owned_column("score_valid"),
        nullable_sum.value_owned_column("sum_value"),
        nullable_sum.validity_owned_column("sum_valid"),
        bigint("bonus", bonus),
    ]);
    let tampered_accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        t.clone(),
        tampered_data,
        0,
        (),
    );

    let tamper_res =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&expr, &accessor, &(), &[]).unwrap();
    assert!(tamper_res
        .verify(&expr, &tampered_accessor, &(), &[])
        .is_err());

    let tampered_nullable_sum =
        NullableBigIntColumn::try_new(vec![12, 0, 10, 13], nullable_sum.validity().to_vec())
            .unwrap();
    let tampered_sum_data = owned_table([
        nullable_score.value_owned_column("score_value"),
        nullable_score.validity_owned_column("score_valid"),
        tampered_nullable_sum.value_owned_column("sum_value"),
        tampered_nullable_sum.validity_owned_column("sum_valid"),
        bigint("bonus", bonus),
    ]);
    let tampered_sum_accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        t.clone(),
        tampered_sum_data,
        0,
        (),
    );

    let tamper_res =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&expr, &accessor, &(), &[]).unwrap();
    assert!(tamper_res
        .verify(&expr, &tampered_sum_accessor, &(), &[])
        .is_err());

    let res =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&expr, &accessor, &(), &[]).unwrap();
    let verified_table = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected_res = owned_table([
        decimal75("score_plus_bonus", 20, 0, [12_i128, 12]),
        bigint("sum_value", [12_i64, 12]),
        boolean("score_valid", [true, true]),
        boolean("sum_valid", [true, true]),
    ]);
    assert_eq!(verified_table, expected_res);
}
