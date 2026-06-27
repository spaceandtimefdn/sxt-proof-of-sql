use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, Column, ColumnRef, ColumnType,
            OwnedTableTestAccessor, Table, TableRef, TableTestAccessor, TestAccessor,
        },
        map::indexmap,
        polynomial::MultilinearExtension,
    },
    sql::{
        proof::{
            exercise_verification, mock_verification_builder::run_verify_for_each_row,
            FinalRoundBuilder, FirstRoundBuilder, VerifiableQueryResult,
        },
        proof_exprs::{or_expr::OrExpr, test_utility::*, ColumnExpr, DynProofExpr, ProofExpr},
        proof_plans::test_utility::*,
        AnalyzeError,
    },
};
use bumpalo::Bump;
use itertools::{multizip, MultiUnzip};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
use sqlparser::ast::Ident;
use std::collections::VecDeque;

#[test]
fn we_can_prove_a_simple_or_query() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4]),
        varchar("d", ["ab", "t", "g", "efg"]),
        bigint("b", [0_i64, 1, 0, 2]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        cols_expr_plan(&t, &["a", "d"], &accessor),
        table_exec(
            t.clone(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("d", ColumnType::VarChar),
                column_field("b", ColumnType::BigInt),
            ],
        ),
        or(
            equal(column(&t, "b", &accessor), const_bigint(1)),
            equal(column(&t, "d", &accessor), const_varchar("g")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([bigint("a", [2_i64, 3]), varchar("d", ["t", "g"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_or_query_with_variable_integer_types() {
    let data = owned_table([
        int128("a", [1_i128, 2, 3, 4]),
        varchar("d", ["ab", "t", "g", "efg"]),
        smallint("b", [0_i16, 1, 0, 2]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        cols_expr_plan(&t, &["a", "d"], &accessor),
        table_exec(
            t.clone(),
            vec![
                column_field("a", ColumnType::Int128),
                column_field("d", ColumnType::VarChar),
                column_field("b", ColumnType::SmallInt),
            ],
        ),
        or(
            equal(column(&t, "b", &accessor), const_bigint(1)),
            equal(column(&t, "d", &accessor), const_varchar("g")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([int128("a", [2_i64, 3]), varchar("d", ["t", "g"])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_or_query_where_both_lhs_and_rhs_are_true() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4]),
        int128("b", [0_i128, 1, 1, 1]),
        int("c", [0_i32, 2, 2, 0]),
        varchar("d", ["ab", "t", "g", "efg"]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        cols_expr_plan(&t, &["a", "d"], &accessor),
        table_exec(
            t.clone(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::Int128),
                column_field("c", ColumnType::Int),
                column_field("d", ColumnType::VarChar),
            ],
        ),
        or(
            equal(column(&t, "b", &accessor), const_bigint(1)),
            equal(column(&t, "d", &accessor), const_varchar("g")),
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([bigint("a", [2_i64, 3, 4]), varchar("d", ["t", "g", "efg"])]);
    assert_eq!(res, expected_res);
}

fn test_random_tables_with_given_offset(offset: usize) {
    let dist = Uniform::new(-3, 4);
    let mut rng = StdRng::from_seed([0u8; 32]);
    for _ in 0..20 {
        // Generate random table
        let n = Uniform::new(1, 21).sample(&mut rng);
        let data = owned_table([
            bigint("a", dist.sample_iter(&mut rng).take(n)),
            varchar(
                "b",
                dist.sample_iter(&mut rng).take(n).map(|v| format!("s{v}")),
            ),
            bigint("c", dist.sample_iter(&mut rng).take(n)),
            varchar(
                "d",
                dist.sample_iter(&mut rng).take(n).map(|v| format!("s{v}")),
            ),
        ]);

        // Generate random values to filter by
        let filter_val1 = format!("s{}", dist.sample(&mut rng));
        let filter_val2 = dist.sample(&mut rng);

        // Create and verify proof
        let t = TableRef::new("sxt", "t");
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t.clone(),
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            cols_expr_plan(&t, &["a", "d"], &accessor),
            table_exec(
                t.clone(),
                vec![
                    column_field("a", ColumnType::BigInt),
                    column_field("b", ColumnType::VarChar),
                    column_field("c", ColumnType::BigInt),
                    column_field("d", ColumnType::VarChar),
                ],
            ),
            or(
                equal(
                    column(&t, "b", &accessor),
                    const_varchar(filter_val1.as_str()),
                ),
                equal(column(&t, "c", &accessor), const_bigint(filter_val2)),
            ),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
        exercise_verification(&verifiable_res, &ast, &accessor, &t);
        let res = verifiable_res
            .verify(&ast, &accessor, &(), &[])
            .unwrap()
            .table;

        // Calculate/compare expected result
        let (expected_a, expected_d): (Vec<_>, Vec<_>) = multizip((
            data["a"].i64_iter(),
            data["b"].string_iter(),
            data["c"].i64_iter(),
            data["d"].string_iter(),
        ))
        .filter_map(|(a, b, c, d)| {
            if b == &filter_val1 || c == &filter_val2 {
                Some((*a, d.clone()))
            } else {
                None
            }
        })
        .multiunzip();
        let expected_result = owned_table([bigint("a", expected_a), varchar("d", expected_d)]);

        assert_eq!(expected_result, res);
    }
}

#[test]
fn we_can_query_random_tables_with_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    test_random_tables_with_given_offset(1001);
}

#[test]
fn we_can_compute_the_correct_output_of_an_or_expr_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 3, 4], &alloc),
        borrowed_bigint("b", [0, 1, 0, 1], &alloc),
        borrowed_bigint("c", [0, 2, 2, 0], &alloc),
        borrowed_varchar("d", ["ab", "t", "g", "efg"], &alloc),
    ]);
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = TableRef::new("sxt", "t");
    accessor.add_table(t.clone(), data.clone(), 0);
    let and_expr: DynProofExpr = or(
        equal(column(&t, "b", &accessor), const_int128(1)),
        equal(column(&t, "d", &accessor), const_varchar("g")),
    );
    let res = and_expr.first_round_evaluate(&alloc, &data, &[]).unwrap();
    let expected_res = Column::Boolean(&[false, true, true, true]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_verify_a_simple_or_proof() {
    let alloc = Bump::new();
    let t: TableRef = "sxt.t".parse().unwrap();
    let lhs = &[true, true, false, false];
    let rhs = &[true, false, true, false];
    let table = Table::try_new(indexmap! {
        "a".into() => Column::Boolean::<TestScalar>(lhs),
        "b".into() => Column::Boolean::<TestScalar>(rhs),
    })
    .unwrap();
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::Boolean);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::Boolean);
    let or_expr = OrExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
    )
    .unwrap();

    let first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(4);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(4, VecDeque::new());

    or_expr
        .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
        .unwrap();

    let verification_builder = run_verify_for_each_row(
        4,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, _chi_eval, evaluation_point| {
            let lhs_eval = lhs.inner_product(evaluation_point);
            let rhs_eval = rhs.inner_product(evaluation_point);
            let accessor = indexmap! {
                a.clone().column_id() => lhs_eval,
                b.clone().column_id() => rhs_eval,
            };
            let res = or_expr
                .verifier_evaluate(verification_builder, &accessor, TestScalar::ONE, &[])
                .unwrap();
            assert_eq!(res, lhs_eval + rhs_eval - lhs_eval * rhs_eval);
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true]; 4]
    );
}

#[test]
fn we_cannot_or_mismatching_types() {
    let alloc = Bump::new();
    let data = table([
        borrowed_smallint("a", [1_i16, 2, 3, 4], &alloc),
        borrowed_varchar("b", ["a", "b", "s", "z"], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data.clone(), 0, ());
    let lhs = Box::new(column(&t, "a", &accessor));
    let rhs = Box::new(column(&t, "b", &accessor));
    let and_err = OrExpr::try_new(lhs.clone(), rhs.clone()).unwrap_err();
    assert!(matches!(
        and_err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}
