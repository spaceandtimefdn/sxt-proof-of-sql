use super::test_utility::*;
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, ColumnType, OwnedTableTestAccessor, TableRef, TestAccessor,
        },
    },
    sql::proof::VerifiableQueryResult,
};

#[test]
fn we_can_prove_a_slice_with_the_naive_commitment_backend() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5]),
        varchar("b", ["one", "two", "three", "four", "five"]),
    ]);
    let table_ref: TableRef = "sxt.slice_source".parse().unwrap();
    let accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
        table_ref.clone(),
        data,
        0,
        (),
    );

    let plan = slice_exec(
        table_exec(
            table_ref.clone(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::VarChar),
            ],
        ),
        1,
        Some(2),
    );

    let result = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([bigint("a", [2_i64, 3]), varchar("b", ["two", "three"])]);
    assert_eq!(result, expected);
}

#[test]
fn we_can_prove_a_union_with_the_naive_commitment_backend() {
    let left = owned_table([
        bigint("a", [1_i64, 2, 3]),
        varchar("b", ["one", "two", "three"]),
    ]);
    let right = owned_table([bigint("a", [4_i64, 5]), varchar("b", ["four", "five"])]);
    let left_ref = TableRef::new("sxt", "left_union");
    let right_ref = TableRef::new("sxt", "right_union");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(left_ref.clone(), left, 0);
    accessor.add_table(right_ref.clone(), right, 0);

    let plan = union_exec(vec![
        table_exec(
            left_ref,
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::VarChar),
            ],
        ),
        table_exec(
            right_ref,
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::VarChar),
            ],
        ),
    ]);

    let result = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([
        bigint("a", [1_i64, 2, 3, 4, 5]),
        varchar("b", ["one", "two", "three", "four", "five"]),
    ]);
    assert_eq!(result, expected);
}
