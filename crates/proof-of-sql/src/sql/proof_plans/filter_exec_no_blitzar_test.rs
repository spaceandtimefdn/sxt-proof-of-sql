use super::{test_utility::*, FilterExec};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{owned_table_utility::*, ColumnField, ColumnRef, ColumnType, TableRef},
        map::IndexSet,
    },
    sql::{
        proof::{ProofPlan, VerifiableQueryResult},
        proof_exprs::{test_utility::*, ColumnExpr, DynProofExpr},
    },
};

fn projected_column(table_ref: &TableRef, name: &str, column_type: ColumnType) -> DynProofExpr {
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
        table_ref.clone(),
        name.into(),
        column_type,
    )))
}

#[test]
fn we_can_verify_filter_exec_with_naive_proof_backend() {
    let table_ref = TableRef::new("sxt", "orders");
    let data = owned_table([
        bigint("order_id", [1_i64, 2, 3, 4, 5]),
        bigint("amount", [10_i64, 20, 30, 40, 50]),
        int128("region", [7_i128, 8, 9, 10, 11]),
    ]);
    let accessor =
        crate::base::database::OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            data,
            0,
            (),
        );
    let plan = filter(
        cols_expr_plan(&table_ref, &["amount", "region"], &accessor),
        table_exec(
            table_ref.clone(),
            vec![
                column_field("order_id", ColumnType::BigInt),
                column_field("amount", ColumnType::BigInt),
                column_field("region", ColumnType::Int128),
            ],
        ),
        and(
            gte(column(&table_ref, "order_id", &accessor), const_int128(2)),
            lt(column(&table_ref, "amount", &accessor), const_int128(50)),
        ),
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
            bigint("amount", [20_i64, 30, 40]),
            int128("region", [8_i128, 9, 10])
        ])
    );
}

#[test]
fn we_can_verify_filter_exec_over_projected_input_aliases() {
    let table_ref = TableRef::new("sxt", "inventory");
    let data = owned_table([
        bigint("sku", [1_i64, 2, 3, 4]),
        bigint("quantity", [5_i64, 8, 13, 21]),
    ]);
    let accessor =
        crate::base::database::OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            data,
            0,
            (),
        );
    let projected_input = projection(
        vec![
            aliased_plan(column(&table_ref, "sku", &accessor), "projected_sku"),
            aliased_plan(
                column(&table_ref, "quantity", &accessor),
                "projected_quantity",
            ),
        ],
        table_exec(
            table_ref.clone(),
            vec![
                column_field("sku", ColumnType::BigInt),
                column_field("quantity", ColumnType::BigInt),
            ],
        ),
    );
    let plan = filter(
        vec![aliased_plan(
            projected_column(&table_ref, "projected_quantity", ColumnType::BigInt),
            "quantity",
        )],
        projected_input,
        gt(
            projected_column(&table_ref, "projected_sku", ColumnType::BigInt),
            const_int128(2),
        ),
    );

    let verifiable_result =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let result = verifiable_result
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    assert_eq!(result, owned_table([bigint("quantity", [13_i64, 21])]));
}

#[test]
fn filter_exec_reports_result_fields_and_input_references() {
    let table_ref = TableRef::new("sxt", "events");
    let schema = vec![
        column_field("event_id", ColumnType::BigInt),
        column_field("visible", ColumnType::Boolean),
    ];
    let input = table_exec(table_ref.clone(), schema);
    let where_clause = projected_column(&table_ref, "visible", ColumnType::Boolean);
    let results = vec![aliased_plan(
        projected_column(&table_ref, "event_id", ColumnType::BigInt),
        "id",
    )];
    let plan = FilterExec::new(
        results.clone(),
        Box::new(input.clone()),
        where_clause.clone(),
    );

    assert_eq!(plan.aliased_results(), results.as_slice());
    assert_eq!(plan.input(), &input);
    assert_eq!(plan.where_clause(), &where_clause);
    assert_eq!(
        plan.get_column_result_fields(),
        vec![ColumnField::new("id".into(), ColumnType::BigInt)]
    );
    assert_eq!(
        plan.get_column_references(),
        IndexSet::from_iter([
            ColumnRef::new(table_ref.clone(), "event_id".into(), ColumnType::BigInt),
            ColumnRef::new(table_ref.clone(), "visible".into(), ColumnType::Boolean),
        ])
    );
    assert_eq!(
        plan.get_table_references(),
        IndexSet::from_iter([table_ref])
    );
}
