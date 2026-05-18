use super::{
    test_utility::{aggregate, table_exec},
    AggregateExec, DynProofPlan,
};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, ColumnField, ColumnType, OwnedTableTestAccessor, TableRef,
            TestAccessor,
        },
    },
    sql::{
        proof::{ProofPlan, VerifiableQueryResult},
        proof_exprs::{
            test_utility::{
                add, aliased_plan, col_ref, cols_expr_plan, column, const_bigint, const_bool,
                const_int128, equal, multiply, sum_expr,
            },
            ProofExpr,
        },
    },
};
use alloc::{boxed::Box, vec};

fn add_test_table(
    accessor: &mut OwnedTableTestAccessor<NaiveEvaluationProof>,
    table_ref: &TableRef,
) {
    accessor.add_table(
        table_ref.clone(),
        owned_table([
            bigint("a", [1, 2, 2, 1, 2]),
            bigint("b", [99, 99, 99, 99, 0]),
            bigint("c", [101, 102, 103, 104, 105]),
        ]),
        0,
    );
}

fn test_schema() -> Vec<ColumnField> {
    vec![
        ColumnField::new("a".into(), ColumnType::BigInt),
        ColumnField::new("b".into(), ColumnType::BigInt),
        ColumnField::new("c".into(), ColumnType::BigInt),
    ]
}

#[test]
fn we_can_verify_aggregate_without_group_by_without_blitzar() {
    let table_ref = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    add_test_table(&mut accessor, &table_ref);

    let plan = aggregate(
        vec![],
        vec![sum_expr(column(&table_ref, "c", &accessor), "sum_c")],
        "__count__",
        table_exec(table_ref.clone(), test_schema()),
        equal(column(&table_ref, "b", &accessor), const_int128(99)),
    );

    let verifiable_result =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let result = verifiable_result
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    assert_eq!(
        result,
        owned_table([bigint("sum_c", [410]), bigint("__count__", [4])])
    );
}

#[test]
fn we_can_verify_grouped_aggregate_without_blitzar() {
    let table_ref = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    add_test_table(&mut accessor, &table_ref);

    let plan = aggregate(
        cols_expr_plan(&table_ref, &["a"], &accessor),
        vec![sum_expr(
            add(
                multiply(column(&table_ref, "c", &accessor), const_bigint(2)),
                const_bigint(1),
            ),
            "sum_c",
        )],
        "__count__",
        table_exec(table_ref.clone(), test_schema()),
        equal(column(&table_ref, "b", &accessor), const_int128(99)),
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
            bigint("a", [1, 2]),
            decimal75("sum_c", 40, 0, [412, 412]),
            bigint("__count__", [2, 2]),
        ])
    );
}

#[test]
fn aggregate_metadata_accessors_and_unsupported_grouping_are_covered() {
    let table_ref = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    add_test_table(&mut accessor, &table_ref);

    let plan = aggregate(
        cols_expr_plan(&table_ref, &["a"], &accessor),
        vec![sum_expr(column(&table_ref, "c", &accessor), "sum_c")],
        "row_count",
        table_exec(table_ref.clone(), test_schema()),
        const_bool(true),
    );

    let DynProofPlan::Aggregate(aggregate_exec) = plan else {
        panic!("expected aggregate plan");
    };

    assert!(matches!(aggregate_exec.input(), DynProofPlan::Table(_)));
    assert_eq!(aggregate_exec.group_by_exprs().len(), 1);
    assert_eq!(aggregate_exec.sum_expr().len(), 1);
    assert_eq!(aggregate_exec.count_alias().value, "row_count");
    assert_eq!(
        aggregate_exec.where_clause().data_type(),
        ColumnType::Boolean
    );
    assert_eq!(aggregate_exec.try_get_is_uniqueness_provable(), Some(true));
    assert_eq!(
        aggregate_exec.get_column_result_fields(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("sum_c".into(), ColumnType::BigInt),
            ColumnField::new("row_count".into(), ColumnType::BigInt),
        ]
    );
    assert_eq!(
        aggregate_exec
            .get_column_references()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            col_ref(&table_ref, "a", &accessor),
            col_ref(&table_ref, "b", &accessor),
            col_ref(&table_ref, "c", &accessor),
        ]
    );
    assert!(aggregate_exec.get_table_references().contains(&table_ref));

    let unsupported = AggregateExec::try_new(
        vec![
            aliased_plan(column(&table_ref, "a", &accessor), "a"),
            aliased_plan(column(&table_ref, "b", &accessor), "b"),
        ],
        vec![sum_expr(column(&table_ref, "c", &accessor), "sum_c")],
        "row_count".into(),
        Box::new(table_exec(table_ref, test_schema())),
        const_bool(true),
    );
    assert!(unsupported.is_none());
}
