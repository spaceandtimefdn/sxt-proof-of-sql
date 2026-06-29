use super::test_utility::*;
use crate::{
    base::{
        database::{
            owned_table_utility::*, table_utility::*, ColumnType, TableRef, TableTestAccessor,
            TestAccessor,
        },
        map::IndexMap,
        proof::ProofError,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{
            exercise_verification, mock_verification_builder::MockVerificationBuilder, ProofPlan,
            VerifiableQueryResult,
        },
        proof_exprs::test_utility::*,
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_sort_merge_join() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 1, 2, 7], &alloc),
        borrowed_varchar(
            "human",
            ["Cassia", "Cassia", "Gretta", "Gretta", "Ian", "Ian", "Erik"],
            &alloc,
        ),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [1_i64, 1, 2, 2, 4]),
        varchar("name", ["Chloe", "Chloe", "Margaret", "Margaret", "Lucy"]),
        varchar("human", ["Cassia", "Ian", "Cassia", "Ian", "Gretta"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_result_from_a_complex_query_involving_sort_merge_join() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let cats = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5, 6, 29, 20, 21], &alloc),
        borrowed_varchar(
            "name",
            [
                "Chloe", "Margaret", "Prudence", "Lucy", "Pepper", "Rocky", "Whiskers", "Mittens",
                "Felix",
            ],
            &alloc,
        ),
    ]);
    let table_cats: TableRef = "sxt.cats".parse().unwrap();
    let cat_details = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 1, 2, 7, 5, 6], &alloc),
        borrowed_varchar(
            "human",
            [
                "Cassia", "Cassia", "Gretta", "Gretta", "Ian", "Ian", "Erik", "Gretta", "Gretta",
            ],
            &alloc,
        ),
    ]);
    let table_cat_details: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_cats.clone(), cats, 0);
    accessor.add_table(table_cat_details.clone(), cat_details, 0);
    let ast = slice_exec(
        sort_merge_join(
            filter(
                cols_expr_plan(&table_cats, &["id", "name"], &accessor),
                table_exec(
                    table_cats.clone(),
                    vec![
                        column_field("id", ColumnType::BigInt),
                        column_field("name", ColumnType::VarChar),
                    ],
                ),
                lte(column(&table_cats, "id", &accessor), const_int128(20)),
            ),
            filter(
                cols_expr_plan(&table_cat_details, &["id", "human"], &accessor),
                table_exec(
                    table_cat_details.clone(),
                    vec![
                        column_field("id", ColumnType::BigInt),
                        column_field("human", ColumnType::VarChar),
                    ],
                ),
                not(equal(
                    column(&table_cat_details, "human", &accessor),
                    const_varchar("Gretta"),
                )),
            ),
            vec![0],
            vec![0],
            vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
        ),
        2,
        Some(3),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &table_cats);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [2_i64, 2]),
        varchar("name", ["Margaret", "Margaret"]),
        varchar("human", ["Cassia", "Ian"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
#[expect(clippy::too_many_lines)]
fn we_can_prove_and_get_the_correct_result_from_a_complex_query_involving_two_sort_merge_joins() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let cats = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5, 6, 10, 29, 20, 21], &alloc),
        borrowed_varchar(
            "name",
            [
                "Chloe", "Margaret", "Prudence", "Lucy", "Pepper", "Rocky", "Nova", "Whiskers",
                "Mittens", "Felix",
            ],
            &alloc,
        ),
    ]);
    let table_cats: TableRef = "sxt.cats".parse().unwrap();
    let cat_human = table([
        borrowed_bigint("id", [1_i64, 2, 98, 4, 10, 1, 2, 7, 5, 6], &alloc),
        borrowed_varchar(
            "human",
            [
                "Cassia", "Cassia", "Gretta", "Gretta", "Trevor", "Ian", "Ian", "Erik", "Gretta",
                "Gretta",
            ],
            &alloc,
        ),
        borrowed_varchar(
            "state",
            ["TX", "TX", "NC", "NC", "CO", "NC", "NC", "ND", "NC", "NC"],
            &alloc,
        ),
    ]);
    let table_cat_human: TableRef = "sxt.cat_human".parse().unwrap();
    let cat_vet = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5, 6, 9, 8, 10], &alloc),
        borrowed_varchar(
            "hospital",
            [
                "Mint Hill",
                "Mint Hill",
                "Brown Creek",
                "Brown Creek",
                "Brown Creek",
                "Brown Creek",
                "Clear Creek",
                "Clear Creek",
                "Rock Creek",
            ],
            &alloc,
        ),
    ]);
    let table_cat_vet: TableRef = "sxt.cat_vet".parse().unwrap();
    accessor.add_table(table_cats.clone(), cats, 0);
    accessor.add_table(table_cat_human.clone(), cat_human, 0);
    accessor.add_table(table_cat_vet.clone(), cat_vet, 0);
    let ast = sort_merge_join(
        sort_merge_join(
            filter(
                cols_expr_plan(&table_cats, &["id", "name"], &accessor),
                table_exec(
                    table_cats.clone(),
                    vec![
                        column_field("id", ColumnType::BigInt),
                        column_field("name", ColumnType::VarChar),
                    ],
                ),
                lte(column(&table_cats, "id", &accessor), const_int128(20)),
            ),
            filter(
                cols_expr_plan(&table_cat_human, &["id", "human", "state"], &accessor),
                table_exec(
                    table_cat_human.clone(),
                    vec![
                        column_field("id", ColumnType::BigInt),
                        column_field("human", ColumnType::VarChar),
                        column_field("state", ColumnType::VarChar),
                    ],
                ),
                not(equal(
                    column(&table_cat_human, "human", &accessor),
                    const_varchar("Gretta"),
                )),
            ),
            vec![0],
            vec![0],
            vec![
                Ident::new("id"),
                Ident::new("name"),
                Ident::new("human"),
                Ident::new("state"),
            ],
        ),
        filter(
            cols_expr_plan(&table_cat_vet, &["id", "hospital"], &accessor),
            table_exec(
                table_cat_vet.clone(),
                vec![
                    column_field("id", ColumnType::BigInt),
                    column_field("hospital", ColumnType::VarChar),
                ],
            ),
            not(equal(
                column(&table_cat_vet, "hospital", &accessor),
                const_varchar("Clear Creek"),
            )),
        ),
        vec![0],
        vec![0],
        vec![
            Ident::new("id"),
            Ident::new("name"),
            Ident::new("human"),
            Ident::new("state"),
            Ident::new("hospital"),
        ],
    );

    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &table_cats);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [1_i64, 1, 2, 2, 10]),
        varchar("name", ["Chloe", "Chloe", "Margaret", "Margaret", "Nova"]),
        varchar("human", ["Cassia", "Ian", "Cassia", "Ian", "Trevor"]),
        varchar("state", ["TX", "NC", "TX", "NC", "CO"]),
        varchar(
            "hospital",
            [
                "Mint Hill",
                "Mint Hill",
                "Mint Hill",
                "Mint Hill",
                "Rock Creek",
            ],
        ),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_sort_merge_join() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [10_i64, 11, 12], &alloc),
        borrowed_varchar("human", ["Rachel", "Rachel", "Megan"], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[expect(clippy::too_many_lines)]
#[test]
fn we_can_prove_and_get_the_correct_empty_result_from_a_sort_merge_join_if_one_or_both_tables_have_no_rows(
) {
    // Left table has no rows but right table has rows
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("name", [""; 0], &alloc),
    ]);
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [10_i64, 11, 12], &alloc),
        borrowed_varchar("human", ["Rachel", "Rachel", "Megan"], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &table_right);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);

    // Right table has no rows but left table has rows
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [1_i64, 2, 3, 4, 5], &alloc),
        borrowed_varchar(
            "name",
            ["Chloe", "Margaret", "Prudence", "Lucy", "Pepper"],
            &alloc,
        ),
    ]);
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("human", [""; 0], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &table_left);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);

    // Both tables have no rows
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let left = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("name", [""; 0], &alloc),
    ]);
    let table_left: TableRef = "sxt.cats".parse().unwrap();
    let right = table([
        borrowed_bigint("id", [0_i64; 0], &alloc),
        borrowed_varchar("human", [""; 0], &alloc),
    ]);
    let table_right: TableRef = "sxt.cat_details".parse().unwrap();
    accessor.add_table(table_left.clone(), left, 0);
    accessor.add_table(table_right.clone(), right, 0);
    let ast = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("name", ColumnType::VarChar),
            ],
        ),
        table_exec(
            table_right.clone(),
            vec![
                column_field("id", ColumnType::BigInt),
                column_field("human", ColumnType::VarChar),
            ],
        ),
        vec![0],
        vec![0],
        vec![Ident::new("id"), Ident::new("name"), Ident::new("human")],
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        bigint("id", [0_i64; 0]),
        varchar("name", [""; 0]),
        varchar("human", [""; 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
#[should_panic(expected = "Join column index out of bounds")]
fn we_cannot_create_sort_merge_join_exec_with_out_of_bounds_join_column_index() {
    sort_merge_join(
        table_exec(
            "sxt.left".parse().unwrap(),
            vec![column_field("a", ColumnType::BigInt)],
        ),
        table_exec(
            "sxt.right".parse().unwrap(),
            vec![column_field("b", ColumnType::BigInt)],
        ),
        vec![1], // index 1 is out of bounds for a 1-column table
        vec![0],
        vec![Ident::new("b")],
    );
}

#[test]
#[should_panic(expected = "Join columns should have the same number of columns")]
fn we_cannot_create_sort_merge_join_exec_with_mismatched_join_column_counts() {
    sort_merge_join(
        table_exec(
            "sxt.left".parse().unwrap(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::BigInt),
            ],
        ),
        table_exec(
            "sxt.right".parse().unwrap(),
            vec![
                column_field("c", ColumnType::BigInt),
                column_field("d", ColumnType::BigInt),
            ],
        ),
        vec![0],     // 1 join column on left
        vec![0, 1],  // 2 join columns on right — mismatch
        vec![Ident::new("a"), Ident::new("c"), Ident::new("d")],
    );
}

#[test]
#[should_panic(
    expected = "The amount of result idents should be the same as the expected number of columns"
)]
fn we_cannot_create_sort_merge_join_exec_with_wrong_result_ident_count() {
    sort_merge_join(
        table_exec(
            "sxt.left".parse().unwrap(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::BigInt),
            ],
        ),
        table_exec(
            "sxt.right".parse().unwrap(),
            vec![
                column_field("c", ColumnType::BigInt),
                column_field("d", ColumnType::BigInt),
            ],
        ),
        vec![0],
        vec![0],
        vec![], // expected 2+2-1=3 idents, providing 0
    );
}

#[test]
fn we_get_error_when_verifying_sort_merge_join_with_multiple_join_columns() {
    let table_left: TableRef = "sxt.left".parse().unwrap();
    let table_right: TableRef = "sxt.right".parse().unwrap();
    let plan = sort_merge_join(
        table_exec(
            table_left.clone(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::BigInt),
            ],
        ),
        table_exec(
            table_right.clone(),
            vec![
                column_field("c", ColumnType::BigInt),
                column_field("d", ColumnType::BigInt),
            ],
        ),
        vec![0, 1], // 2 join columns triggers "not supported yet" guard
        vec![0, 1],
        vec![Ident::new("a"), Ident::new("b")],
    );

    let mut left_cols: IndexMap<Ident, TestScalar> = IndexMap::default();
    left_cols.insert(Ident::new("a"), TestScalar::ONE);
    left_cols.insert(Ident::new("b"), TestScalar::ONE);
    let mut right_cols: IndexMap<Ident, TestScalar> = IndexMap::default();
    right_cols.insert(Ident::new("c"), TestScalar::ONE);
    right_cols.insert(Ident::new("d"), TestScalar::ONE);
    let mut accessor: IndexMap<TableRef, IndexMap<Ident, TestScalar>> = IndexMap::default();
    accessor.insert(table_left.clone(), left_cols);
    accessor.insert(table_right.clone(), right_cols);

    let mut chi_eval_map: IndexMap<TableRef, (TestScalar, usize)> = IndexMap::default();
    chi_eval_map.insert(table_left, (TestScalar::ONE, 1));
    chi_eval_map.insert(table_right, (TestScalar::ONE, 1));

    let mut builder = MockVerificationBuilder::<TestScalar>::new(
        vec![],
        2,
        vec![],
        vec![],
        vec![TestScalar::ONE, TestScalar::ONE], // alpha, beta post-result challenges
        vec![1],                                 // result chi evaluation length
        vec![1],                                 // left rho evaluation length
    );

    let err = plan
        .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
        .unwrap_err();
    assert!(matches!(
        err,
        ProofError::VerificationError {
            error: "Join on multiple columns not supported yet"
        }
    ));
}
