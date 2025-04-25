use super::test_utility::*;
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, ColumnField, ColumnType, TableRef,
            TableTestAccessor, TestAccessor,
        },
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::test_utility::*,
    },
};
use bumpalo::Bump;

#[test]
fn we_can_correctly_filter_data_with_generalized_filter() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // Create a TableExec as input for GeneralizedFilterExec
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
        ],
    );

    // Create GeneralizedFilterExec with TableExec as input
    let where_clause = equal(column(&t, "a", &accessor), const_int128(5_i128));
    let expr = generalized_filter(
        cols_expr_plan(&t, &["b"], &accessor),
        table_exec,
        where_clause,
    );

    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected_res = owned_table([bigint("b", [3_i64, 5])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_correctly_filter_with_complex_condition() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [10, 20, 30, 40, 50], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // Create a TableExec as input for GeneralizedFilterExec
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
            ColumnField::new("c".into(), ColumnType::Int128),
        ],
    );

    // Create GeneralizedFilterExec with TableExec as input and complex condition
    let where_clause = and(
        gte(column(&t, "a", &accessor), const_int128(4_i128)),
        lte(column(&t, "c", &accessor), const_int128(30_i128)),
    );
    let expr = generalized_filter(
        cols_expr_plan(&t, &["a", "b", "c"], &accessor),
        table_exec,
        where_clause,
    );

    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [4_i64, 5]),
        bigint("b", [2, 3]),
        int128("c", [20, 30]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compose_multiple_filters() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [10, 20, 30, 40, 50], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // Create a TableExec as input
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
            ColumnField::new("c".into(), ColumnType::Int128),
        ],
    );

    // First filter to keep rows where a > 3
    let first_filter = generalized_filter(
        cols_expr_plan(&t, &["a", "b", "c"], &accessor),
        table_exec,
        gt(column(&t, "a", &accessor), const_int128(3_i128)),
    );

    // Second filter to keep rows where b < 4
    let expr = generalized_filter(
        cols_expr_plan(&t, &["a", "b", "c"], &accessor),
        first_filter,
        lt(column(&t, "b", &accessor), const_int128(4_i128)),
    );

    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected_res = owned_table([
        bigint("a", [4_i64, 5]),
        bigint("b", [2, 3]),
        int128("c", [20, 30]),
    ]);
    assert_eq!(res, expected_res);
}

// Test for non-trivial composition of GeneralizedFilterExec, GeneralizedFilterExec and TableExec
#[test]
fn we_can_compose_complex_filters() {
    let alloc = Bump::new();
    let data = table([
        borrowed_int("a", [1, 3, 5, 7, 9], &alloc),
        borrowed_int("b", [2, 4, 6, 8, 10], &alloc),
        borrowed_int128("c", [10, 20, 30, 40, 50], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    let intermediate_data = table([
        borrowed_decimal75("a_plus_b", 19, 0, [7, 11, 15, 19], &alloc),
        borrowed_int("a", [3, 5, 7, 9], &alloc),
        borrowed_int128("c", [20, 30, 40, 50], &alloc),
    ]);
    let mut intermediate_accessor =
        TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    intermediate_accessor.add_table(t.clone(), intermediate_data, 0);

    // Create a TableExec as input
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::Int),
            ColumnField::new("b".into(), ColumnType::Int),
            ColumnField::new("c".into(), ColumnType::Int128),
        ],
    );

    // First filter: transform the data by adding a to b and filter out c < 15
    let first_filter = generalized_filter(
        vec![
            aliased_plan(
                add(column(&t, "a", &accessor), column(&t, "b", &accessor)),
                "a_plus_b",
            ),
            aliased_plan(column(&t, "a", &accessor), "a"),
            aliased_plan(column(&t, "c", &accessor), "c"),
        ],
        table_exec,
        lt(column(&t, "c", &accessor), const_smallint(15_i16)),
    );

    // Second filter: filter where a_plus_b > 11
    let expr = generalized_filter(
        vec![
            aliased_plan(column(&t, "a_plus_b", &intermediate_accessor), "sum"),
            aliased_plan(column(&t, "a", &intermediate_accessor), "a"),
            aliased_plan(column(&t, "c", &intermediate_accessor), "c"),
        ],
        first_filter,
        gt(
            column(&t, "a_plus_b", &intermediate_accessor),
            const_int128(11_i128),
        ),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected_res = owned_table([
        decimal75("sum", 19, 0, [15, 19]), // a+b values for rows 2,3,4
        int("a", [5, 7]),               // a values for rows 2,3,4
        int128("c", [30, 40]),             // c values for rows 2,3,4
    ]);
    assert_eq!(res, expected_res);
}
