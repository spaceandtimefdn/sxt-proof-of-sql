use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, ColumnField, ColumnType, OwnedTableTestAccessor, TableRef,
        },
    },
    sql::{
        proof::VerifiableQueryResult, proof_exprs::test_utility::*, proof_plans::test_utility::*,
    },
};

#[test]
fn we_can_prove_a_query_with_a_single_selected_row() {
    let data = owned_table([boolean("a", [true, false])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        cols_expr_plan(&t, &["a"], &accessor),
        table_exec(
            t.clone(),
            vec![ColumnField::new("a".into(), ColumnType::Boolean)],
        ),
    );
    let verifiable_res: VerifiableQueryResult<NaiveEvaluationProof> =
        VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([boolean("a", [true, false])]);
    assert_eq!(res, expected_res);
}
