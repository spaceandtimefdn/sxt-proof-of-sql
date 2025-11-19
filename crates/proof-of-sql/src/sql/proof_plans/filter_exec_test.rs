use super::{test_utility::*, DynProofPlan};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, ColumnField, ColumnRef, ColumnType, TableRef,
            TableTestAccessor, TestAccessor,
        },
        try_standard_binary_deserialization, try_standard_binary_serialization,
    },
    sql::{
        evm_proof_plan::EVMProofPlan,
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{test_utility::*, AddExpr, AliasedDynProofExpr, DynProofExpr},
    },
};
use bumpalo::Bump;

#[test]
fn we_can_get_fields_of_filter() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // Create a TableExec as input for FilterExec
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
        ],
    );

    let where_clause = equal(column(&t, "a", &accessor), const_int128(2_i128));
    let plan = filter(
        cols_expr_plan(&t, &["a"], &accessor),
        table_exec.clone(),
        where_clause.clone(),
    );
    let expected_aliased_results = vec![col_expr_plan(&t, "a", &accessor)];
    if let DynProofPlan::Filter(plan) = plan {
        assert_eq!(plan.aliased_results(), &expected_aliased_results,);
        assert_eq!(plan.input(), &table_exec,);
        assert_eq!(plan.where_clause(), &where_clause);
    } else {
        panic!("Expected FilterExec plan");
    }
}

#[test]
fn we_can_correctly_filter_data_with_filter() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    // Create a TableExec as input for FilterExec
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
        ],
    );

    // Create FilterExec with TableExec as input
    let where_clause = equal(column(&t, "a", &accessor), const_int128(5_i128));
    let expr = filter(
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

    // Create a TableExec as input for FilterExec
    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
            ColumnField::new("c".into(), ColumnType::Int128),
        ],
    );

    // Create FilterExec with TableExec as input and complex condition
    let where_clause = and(
        gte(column(&t, "a", &accessor), const_int128(4_i128)),
        lte(column(&t, "c", &accessor), const_int128(30_i128)),
    );
    let expr = filter(
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
    let first_filter = filter(
        cols_expr_plan(&t, &["a", "b", "c"], &accessor),
        table_exec,
        gt(column(&t, "a", &accessor), const_int128(3_i128)),
    );

    // Second filter to keep rows where b < 4
    let expr = filter(
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

// Test for non-trivial composition of FilterExec, FilterExec and TableExec
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
        borrowed_decimal75("a_plus_b", 11, 0, [7, 11, 15, 19], &alloc),
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

    // First filter: transform the data by adding a to b and filter out c > 15
    let first_filter = filter(
        vec![
            aliased_plan(
                add(column(&t, "a", &accessor), column(&t, "b", &accessor)),
                "a_plus_b",
            ),
            aliased_plan(column(&t, "a", &accessor), "a"),
            aliased_plan(column(&t, "c", &accessor), "c"),
        ],
        table_exec,
        gt(column(&t, "c", &accessor), const_smallint(15_i16)),
    );

    // Second filter: filter where a_plus_b > 11
    let expr = filter(
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
        decimal75("sum", 11, 0, [15, 19]), // a+b values for rows 3,4
        int("a", [7, 9]),                  // a values for rows 3,4
        int128("c", [40, 50]),             // c values for rows 3,4
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_have_projection_as_input_plan_for_filter() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 4, 5, 2, 5], &alloc),
        borrowed_bigint("b", [1, 2, 3, 4, 5], &alloc),
        borrowed_int128("c", [10, 20, 30, 40, 50], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
            ColumnField::new("c".into(), ColumnType::Int128),
        ],
    );

    // Create a TableExec as input
    let projection = projection(
        vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    t.clone(),
                    "a".into(),
                    ColumnType::BigInt,
                )),
                alias: "x".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    t.clone(),
                    "b".into(),
                    ColumnType::BigInt,
                )),
                alias: "y".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_column(ColumnRef::new(
                    t.clone(),
                    "c".into(),
                    ColumnType::Int128,
                )),
                alias: "z".into(),
            },
        ],
        table_exec,
    );

    let dummy_table = TableRef::new("", "");
    let filter_results = vec![
        AliasedDynProofExpr {
            expr: DynProofExpr::Add(
                AddExpr::try_new(
                    Box::new(DynProofExpr::new_column(ColumnRef::new(
                        dummy_table.clone(),
                        "x".into(),
                        ColumnType::BigInt,
                    ))),
                    Box::new(DynProofExpr::new_column(ColumnRef::new(
                        dummy_table.clone(),
                        "y".into(),
                        ColumnType::BigInt,
                    ))),
                )
                .unwrap(),
            ),
            alias: "xplusy".into(),
        },
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(ColumnRef::new(
                dummy_table.clone(),
                "z".into(),
                ColumnType::Int128,
            )),
            alias: "z".into(),
        },
    ];
    // First filter to keep rows where a > 3
    let filter = filter(
        filter_results,
        projection,
        gt(
            DynProofExpr::new_column(ColumnRef::new(
                dummy_table.clone(),
                "z".into(),
                ColumnType::Int128,
            )),
            const_int128(13_i128),
        ),
    );
    let res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&filter, &accessor, &(), &[]).unwrap();
    res.verify(&filter, &accessor, &(), &[]).unwrap();
    let expected_evm_proof_plan = EVMProofPlan::new(filter);
    let deserialized_evem_proof_plan: EVMProofPlan = try_standard_binary_deserialization(
        &try_standard_binary_serialization(expected_evm_proof_plan).unwrap(),
    )
    .unwrap()
    .0;
    let res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&deserialized_evem_proof_plan, &accessor, &(), &[]).unwrap();
    res.verify(&deserialized_evem_proof_plan, &accessor, &(), &[])
        .unwrap();
}
