use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, ColumnField, ColumnRef, ColumnType, OwnedTableTestAccessor,
            TableRef,
        },
        map::{indexmap, IndexSet},
        proof::ProofError,
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{
            exercise_verification, mock_verification_builder::MockVerificationBuilder,
            VerifiableQueryResult,
        },
        proof_exprs::{test_utility::*, ColumnExpr, ProofExpr},
        proof_plans::test_utility::*,
    },
};
use sqlparser::ast::Ident;

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = owned_table([boolean("a", [true, false])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        cols_expr_plan(&t, &["a"], &accessor),
        table_exec(
            t.clone(),
            vec![ColumnField::new("a".into(), ColumnType::Boolean)],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([boolean("a", [true, false])]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_inspect_column_expr_metadata_and_missing_verifier_column_error() {
    let t = TableRef::new("sxt", "t");
    let column_ref = ColumnRef::new(t, Ident::from("a"), ColumnType::Boolean);
    let expr = ColumnExpr::new(column_ref.clone());

    assert_eq!(expr.get_column_reference(), column_ref);
    assert_eq!(expr.column_ref(), &column_ref);
    assert_eq!(expr.column_id(), Ident::from("a"));
    assert_eq!(
        expr.get_column_field(),
        ColumnField::new("a".into(), ColumnType::Boolean)
    );

    let mut columns = IndexSet::default();
    expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 1);
    assert!(columns.contains(&column_ref));

    let mut verifier = MockVerificationBuilder::<TestScalar>::new(
        Vec::new(),
        1,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let accessor = indexmap! {};
    let err = expr
        .verifier_evaluate(&mut verifier, &accessor, TestScalar::ONE, &[])
        .unwrap_err();
    assert!(matches!(
        err,
        ProofError::VerificationError {
            error: "Column Not Found"
        }
    ));
}
