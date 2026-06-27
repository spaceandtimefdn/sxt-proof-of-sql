use super::test_utility::*;
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, table_utility::*, ColumnType, TableRef, TableTestAccessor,
            TestAccessor,
        },
    },
    sql::proof::VerifiableQueryResult,
};
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn we_can_prove_a_sort_merge_join_with_the_naive_commitment_backend() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let left_ref: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 1, 2, 7], &alloc),
        borrowed_varchar(
            "human",
            ["Cassia", "Cassia", "Gretta", "Gretta", "Ian", "Ian", "Erik"],
            &alloc,
        ),
    ]);
    let right_ref: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(left_ref.clone(), left, 0);
    accessor.add_table(right_ref.clone(), right, 0);

    let plan = sort_merge_join(
        table_exec(
            left_ref.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            right_ref,
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );

    let result = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([
        bigint("id", [1_i64, 1, 2, 2, 4]),
        varchar("name", ["Chloe", "Chloe", "Margaret", "Margaret", "Lucy"]),
        varchar("human", ["Cassia", "Ian", "Cassia", "Ian", "Gretta"]),
    ]);
    assert_eq!(result, expected);
}
