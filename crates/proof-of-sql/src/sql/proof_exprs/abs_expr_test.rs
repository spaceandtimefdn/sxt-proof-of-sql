use crate::{
    base::database::{
        owned_table_utility::{bigint, decimal75, int, owned_table, smallint, varchar},
        ColumnType, LiteralValue, OwnedTableTestAccessor, TableRef,
    },
    sql::{
        proof::{exercise_verification, VerifiableQueryResult},
        proof_exprs::{test_utility::*, AbsExpr, DynProofExpr},
        proof_plans::test_utility::{column_field, projection, table_exec},
        AnalyzeError,
    },
};
use blitzar::proof::InnerProductProof;

#[test]
fn we_can_prove_a_query_with_absolute_value() {
    let data = owned_table([
        smallint("a", [1_i16, 2, 3, 4]),
        int("b", [0_i32, 1, 0, 1]),
        varchar("d", ["ab", "t", "efg", "g"]),
        bigint("c", [0_i64, 2, 2, 0]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        vec![
            aliased_plan(
                abs(subtract(column(&t, "a", &accessor), const_bigint(2))),
                "a",
            ),
            col_expr_plan(&t, "c", &accessor),
            aliased_plan(add(column(&t, "b", &accessor), const_bigint(4)), "res"),
            col_expr_plan(&t, "d", &accessor),
        ],
        table_exec(
            t.clone(),
            vec![
                column_field("a", ColumnType::SmallInt),
                column_field("b", ColumnType::Int),
                column_field("d", ColumnType::VarChar),
                column_field("c", ColumnType::BigInt),
            ],
        ),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        decimal75("a", 20, 0, [1_i16, 0, 1, 2]),
        bigint("c", [0i16, 2, 2, 0]),
        decimal75("res", 20, 0, [4_i64, 5, 4, 5]),
        varchar("d", ["ab", "t", "efg", "g"]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_cannot_construct_abs_expr_with_bad_type() {
    let input_expr = Box::new(DynProofExpr::new_literal(LiteralValue::VarChar(
        "test".to_string(),
    )));
    let err = AbsExpr::try_new(input_expr).unwrap_err();
    assert!(
        matches!(err, AnalyzeError::InvalidDataType { expr_type } if expr_type == ColumnType::VarChar)
    );
}
