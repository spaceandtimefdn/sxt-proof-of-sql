use super::{test_utility::*, ProjectionExec};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::*, ColumnField, ColumnRef, ColumnType, OwnedTableTestAccessor,
            TableRef, TestAccessor,
        },
        map::IndexSet,
    },
    sql::{
        proof::{ProofPlan, VerifiableQueryResult},
        proof_exprs::{test_utility::*, ColumnExpr, DynProofExpr},
    },
};
use alloc::boxed::Box;
use sqlparser::ast::Ident;

#[test]
fn we_can_prove_a_projection_with_the_naive_commitment_backend() {
    let data = owned_table([
        bigint("a", [1_i64, 2, 3, 4]),
        int128("b", [10_i128, 20, 30, 40]),
    ]);
    let table_ref = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(table_ref.clone(), data, 0);

    let plan = projection(
        vec![
            col_expr_plan(&table_ref, "b", &accessor),
            aliased_plan(const_bigint(9), "nine"),
        ],
        table_exec(
            table_ref.clone(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::Int128),
            ],
        ),
    );

    let result = VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[])
        .unwrap()
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;

    let expected = owned_table([
        int128("b", [10_i128, 20, 30, 40]),
        bigint("nine", [9_i64; 4]),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn projection_metadata_uses_aliases_and_forwards_input_references() {
    let table_ref = TableRef::new("sxt", "t");
    let projection = ProjectionExec::new(
        vec![
            aliased_plan(
                DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    table_ref.clone(),
                    Ident::new("a"),
                    ColumnType::BigInt,
                ))),
                "renamed_a",
            ),
            aliased_plan(const_bigint(7), "seven"),
        ],
        Box::new(table_exec(
            table_ref.clone(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::Int128),
            ],
        )),
    );

    assert_eq!(
        projection.get_column_result_fields(),
        vec![
            ColumnField::new("renamed_a".into(), ColumnType::BigInt),
            ColumnField::new("seven".into(), ColumnType::BigInt),
        ]
    );
    assert_eq!(
        projection.get_column_references(),
        IndexSet::from_iter([
            ColumnRef::new(table_ref.clone(), Ident::new("a"), ColumnType::BigInt),
            ColumnRef::new(table_ref.clone(), Ident::new("b"), ColumnType::Int128),
        ])
    );
    assert_eq!(
        projection.get_table_references(),
        IndexSet::from_iter([table_ref])
    );
}
