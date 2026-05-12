use super::TableExec;
use crate::{
    base::{
        database::{table_utility::*, ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
        map::{indexmap, indexset, IndexMap},
        scalar::test_scalar::TestScalar,
    },
    sql::proof::{
        mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder, FirstRoundBuilder,
        ProofPlan, ProverEvaluate,
    },
};
use alloc::collections::VecDeque;
use bumpalo::Bump;
use sqlparser::ast::Ident;

fn table_exec_with_schema() -> TableExec {
    TableExec::new(
        TableRef::new("namespace", "table_name"),
        vec![
            ColumnField::new("id".into(), ColumnType::BigInt),
            ColumnField::new("flag".into(), ColumnType::Boolean),
        ],
    )
}

#[test]
fn we_can_read_table_exec_metadata_and_references_without_blitzar() {
    let plan = table_exec_with_schema();
    let table_ref = TableRef::new("namespace", "table_name");

    assert_eq!(plan.table_ref(), &table_ref);
    assert_eq!(
        plan.schema(),
        &[
            ColumnField::new("id".into(), ColumnType::BigInt),
            ColumnField::new("flag".into(), ColumnType::Boolean),
        ]
    );
    assert_eq!(plan.get_column_result_fields(), plan.schema());
    assert_eq!(plan.get_table_references(), indexset! { table_ref.clone() });
    assert_eq!(
        plan.get_column_references(),
        indexset! {
            ColumnRef::new(table_ref.clone(), Ident::new("id"), ColumnType::BigInt),
            ColumnRef::new(table_ref, Ident::new("flag"), ColumnType::Boolean),
        }
    );
}

#[test]
fn we_can_verifier_evaluate_table_exec_from_accessor_maps_without_blitzar() {
    let plan = table_exec_with_schema();
    let table_ref = plan.table_ref().clone();
    let mut builder = MockVerificationBuilder::<TestScalar>::new(
        vec![],
        0,
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let accessor: IndexMap<_, _> = indexmap! {
        table_ref.clone() => indexmap! {
            Ident::new("id") => TestScalar::from(17),
            Ident::new("flag") => TestScalar::from(3),
        }
    };
    let chi_eval_map = indexmap! {
        table_ref => (TestScalar::from(11), 4),
    };

    let evaluation = plan
        .verifier_evaluate(
            &mut builder,
            &accessor,
            &chi_eval_map,
            &[] as &[LiteralValue],
        )
        .unwrap();

    assert_eq!(
        evaluation.column_evals(),
        &[TestScalar::from(17), TestScalar::from(3)]
    );
    assert_eq!(evaluation.chi(), (TestScalar::from(11), 4));
}

#[test]
fn we_can_pass_through_first_and_final_round_tables_without_blitzar() {
    let alloc = Bump::new();
    let plan = table_exec_with_schema();
    let table_ref = plan.table_ref().clone();
    let input_table = table([
        borrowed_bigint("id", [1_i64, 2, 3], &alloc),
        borrowed_boolean("flag", [true, false, true], &alloc),
    ]);
    let table_map = indexmap! {
        table_ref => input_table.clone(),
    };
    let mut first_round_builder = FirstRoundBuilder::new(input_table.num_rows());
    let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());

    let first_round_table = plan
        .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
        .unwrap();
    let final_round_table = plan
        .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
        .unwrap();

    assert_eq!(first_round_table, input_table);
    assert_eq!(final_round_table, input_table);
    assert!(first_round_builder.pcs_proof_mles().is_empty());
    assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 0);
}
