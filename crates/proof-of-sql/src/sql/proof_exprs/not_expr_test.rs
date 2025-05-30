use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, Column, OwnedTableTestAccessor, TableRef,
            TableTestAccessor, TestAccessor,
        },
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{not_expr::NotExpr, test_utility::*, DynProofExpr, ProofExpr},
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

#[test]
fn we_can_prove_a_not_equals_query_with_a_single_selected_row() {
    let data = owned_table([
        bigint("a", [123_i64, 456]),
        bigint("b", [0_i64, 1]),
        varchar("d", ["alfa", "gama"]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        cols_expr_plan(&t, &["a", "d"], &accessor),
        tab(&t),
        not(equal(column(&t, "b", &accessor), const_bigint(1))),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([bigint("a", [123]), varchar("d", ["alfa"])]);
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
        ]);

        // Generate random values to filter by
        let filter_val_a = dist.sample(&mut rng);
        let filter_val_b = format!("s{}", dist.sample(&mut rng));

        // Create and verify proof
        let t = TableRef::new("sxt", "t");
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            t.clone(),
            data.clone(),
            offset,
            (),
        );
        let ast = filter(
            cols_expr_plan(&t, &["a", "b"], &accessor),
            tab(&t),
            not(and(
                equal(column(&t, "a", &accessor), const_bigint(filter_val_a)),
                equal(
                    column(&t, "b", &accessor),
                    const_scalar::<TestScalar, _>(filter_val_b.as_str()),
                ),
            )),
        );
        let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
        exercise_verification(&verifiable_res, &ast, &accessor, &t);
        let res = verifiable_res
            .verify(&ast, &accessor, &(), &[])
            .unwrap()
            .table;

        // Calculate/compare expected result
        let (expected_a, expected_b): (Vec<_>, Vec<_>) =
            multizip((data["a"].i64_iter(), data["b"].string_iter()))
                .filter_map(|(a, b)| {
                    if a != &filter_val_a || b != &filter_val_b {
                        Some((*a, b.clone()))
                    } else {
                        None
                    }
                })
                .multiunzip();
        let expected_result = owned_table([bigint("a", expected_a), varchar("b", expected_b)]);

        assert_eq!(expected_result, res);
    }
}

#[test]
fn we_can_query_random_tables_with_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    test_random_tables_with_given_offset(75);
}

#[test]
fn we_can_compute_the_correct_output_of_a_not_expr_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [123, 456], &alloc),
        borrowed_bigint("b", [0, 1], &alloc),
        borrowed_varchar("d", ["alfa", "gama"], &alloc),
    ]);
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = TableRef::new("sxt", "t");
    accessor.add_table(t.clone(), data.clone(), 0);
    let not_expr: DynProofExpr = not(equal(column(&t, "b", &accessor), const_int128(1)));
    let res = not_expr.first_round_evaluate(&alloc, &data, &[]).unwrap();
    let expected_res = Column::Boolean(&[true, false]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_cannot_not_nonbool_type() {
    let alloc = Bump::new();
    let data = table([borrowed_smallint("a", [1_i16, 2, 3, 4], &alloc)]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data.clone(), 0, ());
    let expr = Box::new(column(&t, "a", &accessor));
    let not_err = NotExpr::try_new(expr.clone()).unwrap_err();
    assert!(matches!(
        not_err,
        AnalyzeError::InvalidDataType { expr_type: _ }
    ));
}
