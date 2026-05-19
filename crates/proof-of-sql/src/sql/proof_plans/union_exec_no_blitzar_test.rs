use super::test_utility::*;
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, ColumnType, OwnedTableTestAccessor, TableRef, TestAccessor,
        },
    },
    sql::{
        proof::VerifiableQueryResult,
        proof_exprs::test_utility::{cols_expr_plan, column, const_int128, gte},
    },
};

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: NotEnoughInputPlans")]
fn we_cannot_get_empty_union_exec_without_blitzar() {
    union_exec(vec![]);
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: NotEnoughInputPlans")]
fn we_cannot_get_single_input_union_exec_without_blitzar() {
    let data = owned_table([
        bigint("a0", [0_i64, 1, 2, 3, 4]),
        varchar("b0", ["", "1", "2", "3", "4"]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let table_plan = table_exec(
        t.clone(),
        vec![
            column_field("a0", ColumnType::BigInt),
            column_field("b0", ColumnType::VarChar),
        ],
    );
    union_exec(vec![filter(
        cols_expr_plan(&t, &["a0"], &accessor),
        table_plan,
        gte(column(&t, "a0", &accessor), const_int128(2_i128)),
    )]);
}

#[test]
fn we_can_prove_and_verify_union_exec_without_blitzar() {
    let t0 = TableRef::new("sxt", "t0");
    let t1 = TableRef::new("sxt", "t1");
    let data0 = owned_table([
        bigint("a0", [1_i64, 2, 3]),
        varchar("b0", ["one", "two", "three"]),
    ]);
    let data1 = owned_table([bigint("a1", [4_i64, 5]), varchar("b1", ["four", "five"])]);
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t0.clone(), data0, 0);
    accessor.add_table(t1.clone(), data1, 0);

    let plan = union_exec(vec![
        projection(
            cols_expr_plan(&t0, &["a0", "b0"], &accessor),
            table_exec(
                t0.clone(),
                vec![
                    column_field("a0", ColumnType::BigInt),
                    column_field("b0", ColumnType::VarChar),
                ],
            ),
        ),
        projection(
            cols_expr_plan(&t1, &["a1", "b1"], &accessor),
            table_exec(
                t1,
                vec![
                    column_field("a1", ColumnType::BigInt),
                    column_field("b1", ColumnType::VarChar),
                ],
            ),
        ),
    ]);

    let verifiable_result =
        VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let verified = verifiable_result
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected = owned_table([
        bigint("a0", [1_i64, 2, 3, 4, 5]),
        varchar("b0", ["one", "two", "three", "four", "five"]),
    ]);

    assert_eq!(verified, expected);
}

#[test]
fn we_can_verify_empty_union_result_without_blitzar() {
    let t0 = TableRef::new("sxt", "t0");
    let t1 = TableRef::new("sxt", "t1");
    let data0 = owned_table([bigint("a0", [0_i64; 0])]);
    let data1 = owned_table([bigint("a1", [0_i64; 0])]);
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t0.clone(), data0, 0);
    accessor.add_table(t1.clone(), data1, 0);

    let plan = union_exec(vec![
        projection(
            cols_expr_plan(&t0, &["a0"], &accessor),
            table_exec(t0, vec![column_field("a0", ColumnType::BigInt)]),
        ),
        projection(
            cols_expr_plan(&t1, &["a1"], &accessor),
            table_exec(t1, vec![column_field("a1", ColumnType::BigInt)]),
        ),
    ]);

    let verified = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected = owned_table([bigint("a0", [0_i64; 0])]);

    assert_eq!(verified, expected);
}
