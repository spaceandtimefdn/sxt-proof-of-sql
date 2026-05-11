use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, table_utility::*, Column, ColumnType, OwnedTableTestAccessor,
            TableRef, TableTestAccessor, TestAccessor,
        },
    },
    sql::{
        proof::VerifiableQueryResult,
        proof_exprs::{is_null_expr::IsNullExpr, test_utility::*, ProofExpr},
        proof_plans::test_utility::*,
        AnalyzeError,
    },
};
use bumpalo::Bump;

#[test]
fn we_can_compute_is_null_from_validity_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("score_value", [10, 0, 30], &alloc),
        borrowed_boolean("score_valid", [true, false, true], &alloc),
    ]);
    let t = TableRef::new("sxt", "nullable_scores");
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data.clone(), 0);

    let expr = is_null(column(&t, "score_valid", &accessor));
    let res = expr.first_round_evaluate(&alloc, &data, &[]).unwrap();

    assert_eq!(res, Column::Boolean(&[false, true, false]));
}

#[test]
fn we_can_prove_is_null_and_is_not_null_filters_over_validity_mask() {
    let data = owned_table([
        bigint("score_value", [10_i64, 0, 30]),
        boolean("score_valid", [true, false, true]),
    ]);
    let t = TableRef::new("sxt", "nullable_scores");
    let accessor =
        OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), data, 0, ());

    let null_rows_ast = filter(
        cols_expr_plan(&t, &["score_value", "score_valid"], &accessor),
        table_exec(
            t.clone(),
            vec![
                column_field("score_value", ColumnType::BigInt),
                column_field("score_valid", ColumnType::Boolean),
            ],
        ),
        is_null(column(&t, "score_valid", &accessor)),
    );
    let null_rows =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&null_rows_ast, &accessor, &(), &[])
            .unwrap();
    assert_eq!(
        null_rows
            .verify(&null_rows_ast, &accessor, &(), &[])
            .unwrap()
            .table,
        owned_table([
            bigint("score_value", [0_i64]),
            boolean("score_valid", [false])
        ])
    );

    let not_null_rows_ast = filter(
        cols_expr_plan(&t, &["score_value", "score_valid"], &accessor),
        table_exec(
            t.clone(),
            vec![
                column_field("score_value", ColumnType::BigInt),
                column_field("score_valid", ColumnType::Boolean),
            ],
        ),
        is_not_null(column(&t, "score_valid", &accessor)),
    );
    let not_null_rows =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&not_null_rows_ast, &accessor, &(), &[])
            .unwrap();
    assert_eq!(
        not_null_rows
            .verify(&not_null_rows_ast, &accessor, &(), &[])
            .unwrap()
            .table,
        owned_table([
            bigint("score_value", [10_i64, 30]),
            boolean("score_valid", [true, true])
        ])
    );
}

#[test]
fn we_cannot_apply_is_null_to_non_boolean_expression() {
    let alloc = Bump::new();
    let data = table([borrowed_bigint("score_value", [10_i64, 0, 30], &alloc)]);
    let t = TableRef::new("sxt", "nullable_scores");
    let accessor =
        TableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), data.clone(), 0, ());
    let expr = Box::new(column(&t, "score_value", &accessor));

    let err = IsNullExpr::try_new(expr, false).unwrap_err();

    assert!(matches!(
        err,
        AnalyzeError::InvalidDataType { expr_type: _ }
    ));
}
