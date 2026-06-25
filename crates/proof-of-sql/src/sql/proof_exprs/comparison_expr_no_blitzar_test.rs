use super::InequalityExpr;
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{Column, ColumnRef, ColumnType, Table, TableRef, TableTestAccessor},
        map::{indexmap, IndexSet},
        polynomial::MultilinearExtension,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{
            mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder,
            FirstRoundBuilder,
        },
        proof_exprs::{test_utility::*, ColumnExpr, DynProofExpr, EqualsExpr, ProofExpr},
        AnalyzeError,
    },
};
use bumpalo::Bump;
use sqlparser::ast::Ident;
use std::collections::VecDeque;

#[test]
fn we_can_inspect_equals_expr_without_blitzar() {
    let t: TableRef = "sxt.t".parse().unwrap();
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::BigInt);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::BigInt);
    let equals_expr = EqualsExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
    )
    .unwrap();

    assert_eq!(equals_expr.data_type(), ColumnType::Boolean);
    assert_eq!(equals_expr.lhs().data_type(), ColumnType::BigInt);
    assert_eq!(equals_expr.rhs().data_type(), ColumnType::BigInt);

    let mut columns = IndexSet::default();
    equals_expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 2);
    assert!(columns.contains(&a));
    assert!(columns.contains(&b));
}

#[test]
fn we_can_reject_equals_expr_type_mismatch_without_blitzar() {
    let equals_err =
        EqualsExpr::try_new(Box::new(const_bigint(12)), Box::new(const_varchar("12"))).unwrap_err();

    assert!(matches!(
        equals_err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}

#[test]
fn we_can_evaluate_equals_expr_rounds_without_blitzar() {
    let alloc = Bump::new();
    let lhs = [1_i64, 2, 3, 4];
    let rhs = [1_i64, 0, 3, 5];
    let table = Table::try_new(indexmap! {
        "a".into() => Column::<TestScalar>::BigInt(&lhs),
        "b".into() => Column::<TestScalar>::BigInt(&rhs),
    })
    .unwrap();
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), table.clone(), 0, ());
    let equals_expr = equal(column(&t, "a", &accessor), column(&t, "b", &accessor));

    let first_round_res = equals_expr
        .first_round_evaluate(&alloc, &table, &[])
        .unwrap();
    assert_eq!(
        first_round_res,
        Column::<TestScalar>::Boolean(&[true, false, true, false])
    );

    let mut final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
    let final_round_res = equals_expr
        .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
        .unwrap();
    assert_eq!(
        final_round_res,
        Column::<TestScalar>::Boolean(&[true, false, true, false])
    );
    assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
    assert_eq!(final_round_builder.sumcheck_subpolynomials().len(), 2);
}

#[test]
fn we_can_verify_equals_expr_constraints_without_blitzar() {
    let alloc = Bump::new();
    let t: TableRef = "sxt.t".parse().unwrap();
    let lhs = [1_i64, 2, 3, 4];
    let rhs = [1_i64, 0, 3, 5];
    let expected = [true, false, true, false];
    let table = Table::try_new(indexmap! {
        "a".into() => Column::<TestScalar>::BigInt(&lhs),
        "b".into() => Column::<TestScalar>::BigInt(&rhs),
    })
    .unwrap();
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::BigInt);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::BigInt);
    let equals_expr = EqualsExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
    )
    .unwrap();

    let first_round_builder = FirstRoundBuilder::new(4);
    let mut final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
    equals_expr
        .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
        .unwrap();

    let verification_builder = run_verify_for_each_row(
        4,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_eval, evaluation_point| {
            let accessor = indexmap! {
                a.column_id() => (&lhs[..]).inner_product(evaluation_point),
                b.column_id() => (&rhs[..]).inner_product(evaluation_point),
            };
            let eval = equals_expr
                .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                .unwrap();
            assert_eq!(eval, (&expected[..]).inner_product(evaluation_point));
        },
    );
    assert_eq!(
        verification_builder.get_identity_results(),
        vec![vec![true, true]; 4]
    );
}

#[test]
fn we_can_inspect_inequality_expr_without_blitzar() {
    let t: TableRef = "sxt.t".parse().unwrap();
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::BigInt);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::BigInt);
    let inequality_expr = InequalityExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
        true,
    )
    .unwrap();

    assert_eq!(inequality_expr.data_type(), ColumnType::Boolean);
    assert_eq!(inequality_expr.lhs().data_type(), ColumnType::BigInt);
    assert_eq!(inequality_expr.rhs().data_type(), ColumnType::BigInt);
    assert!(inequality_expr.is_lt());

    let mut columns = IndexSet::default();
    inequality_expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 2);
    assert!(columns.contains(&a));
    assert!(columns.contains(&b));
}

#[test]
fn we_can_reject_inequality_expr_type_mismatch_without_blitzar() {
    let inequality_err = InequalityExpr::try_new(
        Box::new(const_varchar("12")),
        Box::new(const_bigint(12)),
        true,
    )
    .unwrap_err();

    assert!(matches!(
        inequality_err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}

#[test]
fn we_can_evaluate_inequality_expr_rounds_without_blitzar() {
    let alloc = Bump::new();
    let lhs = [-1_i64, 9, 1, 4];
    let rhs = [1_i64, 2, 3, 4];
    let table = Table::try_new(indexmap! {
        "a".into() => Column::<TestScalar>::BigInt(&lhs),
        "b".into() => Column::<TestScalar>::BigInt(&rhs),
    })
    .unwrap();
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), table.clone(), 0, ());
    let inequality_expr = lt(column(&t, "a", &accessor), column(&t, "b", &accessor));

    let first_round_res = inequality_expr
        .first_round_evaluate(&alloc, &table, &[])
        .unwrap();
    assert_eq!(
        first_round_res,
        Column::<TestScalar>::Boolean(&[true, false, true, false])
    );

    let mut final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
    let final_round_res = inequality_expr
        .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
        .unwrap();
    assert_eq!(
        final_round_res,
        Column::<TestScalar>::Boolean(&[true, false, true, false])
    );
    assert_eq!(final_round_builder.bit_distributions().len(), 1);
}

#[test]
fn we_can_verify_inequality_expr_constraints_without_blitzar() {
    let alloc = Bump::new();
    let t: TableRef = "sxt.t".parse().unwrap();
    let lhs = [-1_i64, 9, 1, 4];
    let rhs = [1_i64, 2, 3, 4];
    let expected = [true, false, true, false];
    let table = Table::try_new(indexmap! {
        "a".into() => Column::<TestScalar>::BigInt(&lhs),
        "b".into() => Column::<TestScalar>::BigInt(&rhs),
    })
    .unwrap();
    let a = ColumnRef::new(t.clone(), Ident::from("a"), ColumnType::BigInt);
    let b = ColumnRef::new(t, Ident::from("b"), ColumnType::BigInt);
    let inequality_expr = InequalityExpr::try_new(
        Box::new(DynProofExpr::Column(ColumnExpr::new(a.clone()))),
        Box::new(DynProofExpr::Column(ColumnExpr::new(b.clone()))),
        true,
    )
    .unwrap();

    let first_round_builder = FirstRoundBuilder::new(4);
    let mut final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
    inequality_expr
        .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
        .unwrap();

    let verification_builder = run_verify_for_each_row(
        4,
        &first_round_builder,
        &final_round_builder,
        Vec::new(),
        3,
        |verification_builder, chi_eval, evaluation_point| {
            let accessor = indexmap! {
                a.column_id() => (&lhs[..]).inner_product(evaluation_point),
                b.column_id() => (&rhs[..]).inner_product(evaluation_point),
            };
            let eval = inequality_expr
                .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                .unwrap();
            assert_eq!(eval, (&expected[..]).inner_product(evaluation_point));
        },
    );
    assert!(verification_builder
        .get_identity_results()
        .iter()
        .flatten()
        .all(|constraint_passed| *constraint_passed));
}
