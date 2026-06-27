use super::test_utility::*;
use crate::{
    base::database::{
        owned_table_utility::*, table_utility::*, ColumnField, ColumnRef, ColumnType,
        TableEvaluation, TableRef, TableTestAccessor,
    },
    base::{
        map::{indexmap, indexset},
        scalar::test_scalar::TestScalar,
    },
    sql::proof::{
        exercise_verification, mock_verification_builder::MockVerificationBuilder, ProofPlan,
        VerifiableQueryResult,
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn we_can_create_and_prove_an_empty_table_exec() {
    let alloc = Bump::new();
    let table_ref = TableRef::new("namespace", "table_name");
    let plan = table_exec(
        table_ref.clone(),
        vec![ColumnField::new("a".into(), ColumnType::BigInt)],
    );
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref.clone(),
        table([borrowed_bigint("a", [0_i64; 0], &alloc)]),
        0_usize,
        (),
    );
    let verifiable_res =
        VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
    let res = verifiable_res
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected = owned_table([bigint("a", [0_i64; 0])]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_create_and_prove_a_table_exec() {
    let alloc = Bump::new();
    let table_ref = TableRef::new("namespace", "table_name");
    let plan = table_exec(
        table_ref.clone(),
        vec![
            ColumnField::new("language_rank".into(), ColumnType::BigInt),
            ColumnField::new("language_name".into(), ColumnType::VarChar),
            ColumnField::new("space_and_time".into(), ColumnType::VarChar),
        ],
    );
    let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref.clone(),
        table([
            borrowed_bigint("language_rank", [0_i64, 1, 2, 3], &alloc),
            borrowed_varchar(
                "language_name",
                ["English", "Español", "Português", "Français"],
                &alloc,
            ),
            borrowed_varchar(
                "space_and_time",
                [
                    "space and time",
                    "espacio y tiempo",
                    "espaço e tempo",
                    "espace et temps",
                ],
                &alloc,
            ),
        ]),
        0_usize,
        (),
    );
    let verifiable_res = VerifiableQueryResult::new(&plan, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &plan, &accessor, &table_ref);
    let res = verifiable_res
        .verify(&plan, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected = owned_table([
        bigint("language_rank", [0, 1, 2, 3]),
        varchar(
            "language_name",
            ["English", "Español", "Português", "Français"],
        ),
        varchar(
            "space_and_time",
            [
                "space and time",
                "espacio y tiempo",
                "espaço e tempo",
                "espace et temps",
            ],
        ),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn we_can_verify_table_exec_metadata_and_evaluations_directly() {
    let table_ref = TableRef::new("namespace", "table_name");
    let schema = vec![
        ColumnField::new("language_rank".into(), ColumnType::BigInt),
        ColumnField::new("language_name".into(), ColumnType::VarChar),
    ];
    let plan = table_exec(table_ref.clone(), schema.clone());

    assert_eq!(plan.get_column_result_fields(), schema);
    assert_eq!(plan.get_table_references(), indexset! { table_ref.clone() });
    assert_eq!(
        plan.get_column_references(),
        indexset! {
            ColumnRef::new(table_ref.clone(), Ident::from("language_rank"), ColumnType::BigInt),
            ColumnRef::new(table_ref.clone(), Ident::from("language_name"), ColumnType::VarChar),
        }
    );

    let mut builder = MockVerificationBuilder::<TestScalar>::new(
        Vec::new(),
        0,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let accessor = indexmap! {
        table_ref.clone() => indexmap! {
            Ident::from("language_rank") => TestScalar::from(1),
            Ident::from("language_name") => TestScalar::from(2),
        },
    };
    let chi_eval_map = indexmap! {
        table_ref => (TestScalar::from(3), 4),
    };

    let res = plan
        .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
        .unwrap();
    assert_eq!(
        res,
        TableEvaluation::new(
            vec![TestScalar::from(1), TestScalar::from(2)],
            (TestScalar::from(3), 4)
        )
    );
}
