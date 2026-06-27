use super::{test_utility::*, DynProofExpr, ProofExpr};
use crate::base::{
    commitment::InnerProductProof,
    database::{
        table_utility::*, Column, ColumnRef, ColumnType, Table, TableRef, TableTestAccessor,
        TestAccessor,
    },
    map::indexmap,
    polynomial::MultilinearExtension,
    scalar::test_scalar::TestScalar,
};
use crate::sql::proof::{
    mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder, FirstRoundBuilder,
};
use crate::sql::proof_exprs::ColumnExpr;
use bumpalo::Bump;
use sqlparser::ast::Ident;
use std::collections::VecDeque;

#[test]
fn we_can_compute_the_correct_result_of_a_complex_bool_expr_using_first_round_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint(
            "a",
            [1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 7, 8, 9, 999],
            &alloc,
        ),
        borrowed_varchar(
            "b",
            [
                "g", "g", "t", "ghi", "g", "g", "jj", "f", "g", "g", "gar", "qwe", "g", "g", "poi",
                "zxc", "999",
            ],
            &alloc,
        ),
        borrowed_int128(
            "c",
            [
                3, 123, 3, 234, 3, 345, 3, 456, 3, 567, 3, 678, 3, 789, 3, 890, 999,
            ],
            &alloc,
        ),
    ]);
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = TableRef::new("sxt", "t");
    accessor.add_table(t.clone(), data.clone(), 0);
    // (a <= 5 || b == "g") && c != 3
    let bool_expr: DynProofExpr = and(
        or(
            lte(column(&t, "a", &accessor), const_bigint(5)),
            equal(column(&t, "b", &accessor), const_varchar("g")),
        ),
        not(equal(column(&t, "c", &accessor), const_int128(3))),
    );
    let res = bool_expr.first_round_evaluate(&alloc, &data, &[]).unwrap();
    let expected_res = Column::Boolean(&[
        false, true, false, true, false, true, false, true, false, true, false, true, false, true,
        false, false, false,
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_verify_the_correct_result_of_a_complex_bool_expr() {
    let alloc = Bump::new();
    let t: TableRef = "sxt.t".parse().unwrap();
    let lhs = &[true, true, false, false];
    let rhs = &[true, false, true, false];
    let excluded = &[false, true, false, true];
    let expected = &[true, false, true, false];
    let table = Table::try_new(indexmap! {
        "lhs".into() => Column::Boolean::<TestScalar>(lhs),
        "rhs".into() => Column::Boolean::<TestScalar>(rhs),
        "excluded".into() => Column::Boolean::<TestScalar>(excluded),
    })
    .unwrap();
    let lhs_ref = ColumnRef::new(t.clone(), Ident::from("lhs"), ColumnType::Boolean);
    let rhs_ref = ColumnRef::new(t.clone(), Ident::from("rhs"), ColumnType::Boolean);
    let excluded_ref = ColumnRef::new(t, Ident::from("excluded"), ColumnType::Boolean);
    let bool_expr: DynProofExpr = and(
        or(
            DynProofExpr::Column(ColumnExpr::new(lhs_ref.clone())),
            DynProofExpr::Column(ColumnExpr::new(rhs_ref.clone())),
        ),
        not(DynProofExpr::Column(ColumnExpr::new(excluded_ref.clone()))),
    );

    let first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(4);
    let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
        FinalRoundBuilder::new(4, VecDeque::new());

    let res = bool_expr
        .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
        .unwrap();
    assert_eq!(res, Column::Boolean(expected));

    let verification_builder = run_verify_for_each_row(
        4,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_eval, evaluation_point| {
            let accessor = indexmap! {
                lhs_ref.clone().column_id() => lhs.inner_product(evaluation_point),
                rhs_ref.clone().column_id() => rhs.inner_product(evaluation_point),
                excluded_ref.clone().column_id() => excluded.inner_product(evaluation_point),
            };
            let res = bool_expr
                .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                .unwrap();
            assert_eq!(res, expected.inner_product(evaluation_point));
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true, true, true]; 4]
    );
}
