use super::test_utility::*;
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, ColumnType, OwnedTableTestAccessor, TableRef, TestAccessor,
        },
    },
    sql::{proof::VerifiableQueryResult, proof_exprs::test_utility::*},
};

fn sample_accessor(table_ref: &TableRef) -> OwnedTableTestAccessor<'_, NaiveEvaluationProof> {
    let data = owned_table([
        bigint("a", [1_i64, 2, 2, 1, 2]),
        bigint("b", [99_i64, 99, 99, 99, 0]),
        bigint("c", [101_i64, 102, 103, 104, 105]),
    ]);
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(table_ref.clone(), data, 0);
    accessor
}

#[test]
fn we_can_prove_group_by_with_the_naive_commitment_backend() {
    let table_ref = TableRef::new("sxt", "group_source");
    let accessor = sample_accessor(&table_ref);
    let plan = group_by(
        cols_expr(&table_ref, &["a"], &accessor),
        vec![sum_expr(
            add(
                multiply(column(&table_ref, "c", &accessor), const_bigint(2)),
                const_bigint(1),
            ),
            "sum_c",
        )],
        "__count__",
        tab(&table_ref),
        equal(column(&table_ref, "b", &accessor), const_int128(99)),
    );

    let result = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([
        bigint("a", [1_i64, 2]),
        decimal75("sum_c", 40, 0, [(101 + 104) * 2 + 2, (102 + 103) * 2 + 2]),
        bigint("__count__", [2_i64, 2]),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn we_can_prove_aggregate_with_the_naive_commitment_backend() {
    let table_ref = TableRef::new("sxt", "aggregate_source");
    let accessor = sample_accessor(&table_ref);
    let plan = aggregate(
        cols_expr_plan(&table_ref, &["a"], &accessor),
        vec![sum_expr(column(&table_ref, "c", &accessor), "sum_c")],
        "__count__",
        table_exec(
            table_ref.clone(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::BigInt),
                column_field("c", ColumnType::BigInt),
            ],
        ),
        equal(column(&table_ref, "b", &accessor), const_int128(99)),
    );

    let result = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([
        bigint("a", [1_i64, 2]),
        bigint("sum_c", [101_i64 + 104, 102 + 103]),
        bigint("__count__", [2_i64, 2]),
    ]);
    assert_eq!(result, expected);
}
