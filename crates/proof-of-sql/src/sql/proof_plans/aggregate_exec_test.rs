use super::test_utility::*;
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, ColumnField, ColumnType, TableRef,
            TableTestAccessor, TestAccessor,
        },
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{test_utility::*, TableExpr},
        proof_plans::{DynProofPlan, FilterExec},
    },
};
use alloc::boxed::Box;
use bumpalo::Bump;

/// `select sum(c) as sum_c, count(*) as __count__ from sxt.t where b = 99`
#[test]
fn we_can_prove_aggregation_without_group_by() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 2, 1, 2], &alloc),
        borrowed_bigint("b", [99, 99, 99, 99, 0], &alloc),
        borrowed_bigint("c", [101, 102, 103, 104, 105], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = aggregate(
        vec![],
        vec![sum_expr(column(&t, "c", &accessor), "sum_c")],
        "__count__",
        Box::new(table_exec(
            t.clone(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::BigInt),
                ColumnField::new("c".into(), ColumnType::BigInt),
            ],
        )),
        equal(column(&t, "b", &accessor), const_int128(99)),
        true,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("sum_c", [101 + 104 + 102 + 103]),
        bigint("__count__", [4]),
    ]);
    assert_eq!(res, expected);
}

/// `select a, sum(c) as sum_c, count(*) as __count__ from sxt.t where b = 99 group by a`
#[test]
fn we_can_prove_a_simple_group_by_with_bigint_columns() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 2, 1, 2], &alloc),
        borrowed_bigint("b", [99, 99, 99, 99, 0], &alloc),
        borrowed_bigint("c", [101, 102, 103, 104, 105], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = aggregate(
        vec![col_expr_plan(&t, "a", &accessor)],
        vec![sum_expr(column(&t, "c", &accessor), "sum_c")],
        "__count__",
        Box::new(table_exec(
            t.clone(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::BigInt),
                ColumnField::new("c".into(), ColumnType::BigInt),
            ],
        )),
        equal(column(&t, "b", &accessor), const_int128(99)),
        true,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("a", [1, 2]),
        bigint("sum_c", [101 + 104, 102 + 103]),
        bigint("__count__", [2, 2]),
    ]);
    assert_eq!(res, expected);
}

/// `select a, sum(c * 2 + 1) as sum_c, count(*) as __count__ from sxt.t where b = 99 group by a`
#[test]
fn we_can_prove_a_group_by_with_bigint_columns() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 2, 1, 2], &alloc),
        borrowed_bigint("b", [99, 99, 99, 99, 0], &alloc),
        borrowed_bigint("c", [101, 102, 103, 104, 105], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = aggregate(
        vec![col_expr_plan(&t, "a", &accessor)],
        vec![sum_expr(
            add(
                multiply(column(&t, "c", &accessor), const_bigint(2)),
                const_bigint(1),
            ),
            "sum_c",
        )],
        "__count__",
        Box::new(table_exec(
            t.clone(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::BigInt),
                ColumnField::new("c".into(), ColumnType::BigInt),
            ],
        )),
        equal(column(&t, "b", &accessor), const_int128(99)),
        true,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("a", [1, 2]),
        decimal75("sum_c", 40, 0, [(101 + 104) * 2 + 2, (102 + 103) * 2 + 2]),
        bigint("__count__", [2, 2]),
    ]);
    assert_eq!(res, expected);
}

#[expect(clippy::too_many_lines)]
#[test]
fn we_can_prove_a_complex_group_by_query_with_many_columns() {
    let alloc = Bump::new();
    let scalar_filter_data: Vec<Curve25519Scalar> = [
        333, 222, 222, 333, 222, 333, 333, 333, 222, 222, 222, 333, 222, 222, 222, 222, 222, 222,
        333, 333,
    ]
    .iter()
    .map(core::convert::Into::into)
    .collect();
    let scalar_group_data: Vec<Curve25519Scalar> =
        [5, 4, 5, 4, 4, 4, 5, 4, 4, 4, 5, 4, 4, 4, 5, 4, 4, 4, 4, 5]
            .iter()
            .map(core::convert::Into::into)
            .collect();
    let scalar_sum_data: Vec<Curve25519Scalar> = [
        119, 522, 100, 325, 501, 447, 759, 375, 212, 532, 459, 616, 579, 179, 695, 963, 532, 868,
        331, 830,
    ]
    .iter()
    .map(core::convert::Into::into)
    .collect();
    let data = table([
        borrowed_bigint(
            "bigint_filter",
            [
                30, 20, 30, 30, 30, 20, 30, 20, 30, 20, 30, 20, 20, 20, 30, 30, 20, 20, 20, 30,
            ],
            &alloc,
        ),
        borrowed_bigint(
            "bigint_group",
            [7, 6, 6, 6, 7, 7, 6, 6, 6, 6, 7, 7, 6, 7, 6, 7, 7, 7, 6, 7],
            &alloc,
        ),
        borrowed_bigint(
            "bigint_sum",
            [
                834, 985, 832, 300, 146, 624, 553, 637, 770, 574, 913, 600, 336, 984, 198, 257,
                781, 196, 537, 358,
            ],
            &alloc,
        ),
        borrowed_int128(
            "int128_filter",
            [
                1030, 1030, 1030, 1020, 1020, 1030, 1020, 1020, 1020, 1030, 1030, 1030, 1020, 1020,
                1030, 1020, 1020, 1030, 1020, 1030,
            ],
            &alloc,
        ),
        borrowed_int128(
            "int128_group",
            [8, 8, 8, 8, 8, 8, 9, 9, 8, 9, 8, 9, 8, 9, 8, 9, 8, 8, 8, 8],
            &alloc,
        ),
        borrowed_int128(
            "int128_sum",
            [
                275, 225, 315, 199, 562, 578, 563, 513, 634, 829, 613, 295, 509, 923, 133, 973,
                700, 464, 622, 943,
            ],
            &alloc,
        ),
        borrowed_varchar(
            "varchar_filter",
            [
                "f2", "f2", "f3", "f2", "f2", "f3", "f3", "f2", "f2", "f3", "f2", "f2", "f2", "f3",
                "f2", "f3", "f2", "f2", "f3", "f3",
            ],
            &alloc,
        ),
        borrowed_varchar(
            "varchar_group",
            [
                "g1", "g2", "g1", "g1", "g1", "g1", "g2", "g1", "g1", "g1", "g2", "g2", "g1", "g1",
                "g1", "g2", "g1", "g2", "g1", "g1",
            ],
            &alloc,
        ),
        borrowed_scalar("scalar_filter", scalar_filter_data, &alloc),
        borrowed_scalar("scalar_group", scalar_group_data, &alloc),
        borrowed_scalar("scalar_sum", scalar_sum_data, &alloc),
    ]);

    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // SELECT scalar_group, int128_group, bigint_group, sum(bigint_sum + 1) as sum_int, sum(bigint_sum - int128_sum) as sum_bigint, sum(scalar_filter) as sum_scal, count(*) as __count__
    //  FROM sxt.t WHERE int128_filter = 1020 AND varchar_filter = 'f2'
    //  GROUP BY scalar_group, int128_group, bigint_group
    let expr = aggregate(
        cols_expr_plan(
            &t,
            &["scalar_group", "int128_group", "bigint_group"],
            &accessor,
        ),
        vec![
            sum_expr(
                add(column(&t, "bigint_sum", &accessor), const_bigint(1)),
                "sum_int",
            ),
            sum_expr(
                subtract(
                    column(&t, "bigint_sum", &accessor),
                    column(&t, "int128_sum", &accessor),
                ),
                "sum_128",
            ),
            sum_expr(column(&t, "scalar_sum", &accessor), "sum_scal"),
        ],
        "__count__",
        Box::new(table_exec(
            t.clone(),
            vec![
                ColumnField::new("bigint_filter".into(), ColumnType::BigInt),
                ColumnField::new("bigint_group".into(), ColumnType::BigInt),
                ColumnField::new("bigint_sum".into(), ColumnType::BigInt),
                ColumnField::new("int128_filter".into(), ColumnType::Int128),
                ColumnField::new("int128_group".into(), ColumnType::Int128),
                ColumnField::new("int128_sum".into(), ColumnType::Int128),
                ColumnField::new("varchar_filter".into(), ColumnType::VarChar),
                ColumnField::new("varchar_group".into(), ColumnType::VarChar),
                ColumnField::new("scalar_filter".into(), ColumnType::Scalar),
                ColumnField::new("scalar_group".into(), ColumnType::Scalar),
                ColumnField::new("scalar_sum".into(), ColumnType::Scalar),
            ],
        )),
        and(
            equal(column(&t, "int128_filter", &accessor), const_int128(1020)),
            equal(column(&t, "varchar_filter", &accessor), const_varchar("f2")),
        ),
        true,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        scalar("scalar_group", [4, 4, 4]),
        int128("int128_group", [8, 8, 9]),
        bigint("bigint_group", [6, 7, 6]),
        decimal75("sum_int", 20, 0, [1409, 929, 638]),
        decimal75("sum_128", 40, 0, [64, -335, 124]),
        scalar("sum_scal", [1116, 1033, 375]),
        bigint("__count__", [3, 2, 1]),
    ]);
    assert_eq!(res, expected);

    // SELECT sum(bigint_sum) as sum_int, sum(int128_sum * 4) as sum_128, sum(scalar_sum) as sum_scal, count(*) as __count__
    //  FROM sxt.t WHERE int128_filter = 1020 AND varchar_filter = 'f2'
    let expr = aggregate(
        vec![],
        vec![
            sum_expr(column(&t, "bigint_sum", &accessor), "sum_int"),
            sum_expr(
                multiply(column(&t, "int128_sum", &accessor), const_bigint(4)),
                "sum_128",
            ),
            sum_expr(column(&t, "scalar_sum", &accessor), "sum_scal"),
        ],
        "__count__",
        Box::new(table_exec(
            t.clone(),
            vec![
                ColumnField::new("bigint_filter".into(), ColumnType::BigInt),
                ColumnField::new("bigint_group".into(), ColumnType::BigInt),
                ColumnField::new("bigint_sum".into(), ColumnType::BigInt),
                ColumnField::new("int128_filter".into(), ColumnType::Int128),
                ColumnField::new("int128_group".into(), ColumnType::Int128),
                ColumnField::new("int128_sum".into(), ColumnType::Int128),
                ColumnField::new("varchar_filter".into(), ColumnType::VarChar),
                ColumnField::new("varchar_group".into(), ColumnType::VarChar),
                ColumnField::new("scalar_filter".into(), ColumnType::Scalar),
                ColumnField::new("scalar_group".into(), ColumnType::Scalar),
                ColumnField::new("scalar_sum".into(), ColumnType::Scalar),
            ],
        )),
        and(
            equal(column(&t, "int128_filter", &accessor), const_int128(1020)),
            equal(column(&t, "varchar_filter", &accessor), const_varchar("f2")),
        ),
        true,
    );
    let res = VerifiableQueryResult::new(&expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &expr, &accessor, &t);
    let res = res.verify(&expr, &accessor, &(), &[]).unwrap().table;
    let expected = owned_table([
        bigint("sum_int", [1406 + 927 + 637]),
        decimal75("sum_128", 59, 0, [(1342 + 1262 + 513) * 4]),
        scalar("sum_scal", [1116 + 1033 + 375]),
        bigint("__count__", [3 + 2 + 1]),
    ]);
    assert_eq!(res, expected);
}

/// Test for non-trivial composition of `AggregateExec` and `FilterExec`
#[test]
fn we_can_compose_aggregate_with_filter() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 3, 3, 5, 5], &alloc),
        borrowed_bigint("b", [10, 20, 20, 40, 40], &alloc),
        borrowed_bigint("c", [100, 200, 300, 400, 500], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let intermediate_data = table([
        borrowed_decimal75("a", 39, 0, [6, 6, 10, 10], &alloc),
        borrowed_decimal75("b", 39, 0, [40, 40, 80, 80], &alloc),
        borrowed_decimal75("c", 39, 0, [400, 600, 800, 1000], &alloc),
    ]);
    let mut intermediate_accessor =
        TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    intermediate_accessor.add_table(t.clone(), intermediate_data, 0);

    // First create a `FilterExec` that filters rows where a >= 2 and doubles the value of column a, b, c
    let filter_plan = Box::new(DynProofPlan::Filter(FilterExec::new(
        vec![
            aliased_plan(multiply(column(&t, "a", &accessor), const_bigint(2)), "a"),
            aliased_plan(multiply(column(&t, "b", &accessor), const_bigint(2)), "b"),
            aliased_plan(multiply(column(&t, "c", &accessor), const_bigint(2)), "c"),
        ],
        TableExpr {
            table_ref: t.clone(),
        },
        gte(column(&t, "a", &accessor), const_smallint(2)),
    )));

    // Then create an `AggregateExec` that groups by a + b and 2 * a and sums column c
    let aggregate_expr = aggregate(
        vec![
            aliased_plan(
                add(
                    column(&t, "a", &intermediate_accessor),
                    column(&t, "b", &intermediate_accessor),
                ),
                "a_plus_b",
            ),
            aliased_plan(
                multiply(column(&t, "a", &intermediate_accessor), const_bigint(2)),
                "two_a",
            ),
        ],
        vec![sum_expr(column(&t, "c", &intermediate_accessor), "sum_c")],
        "__count__",
        filter_plan,
        const_bool(true),
        true,
    );

    let res = VerifiableQueryResult::new(&aggregate_expr, &accessor, &(), &[]).unwrap();
    exercise_verification(&res, &aggregate_expr, &accessor, &t);
    let res = res
        .verify(&aggregate_expr, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([
        decimal75("a_plus_b", 40, 0, [46, 90]),
        decimal75("two_a", 59, 0, [12, 20]),
        decimal75("sum_c", 39, 0, [1000, 1800]),
        bigint("__count__", [2, 2]),
    ]);

    assert_eq!(res, expected);
}
