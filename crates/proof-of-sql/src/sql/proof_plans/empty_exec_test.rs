use super::EmptyExec;
use crate::{
    base::{
        database::{LiteralValue, Table, TableRef},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, SumcheckMleEvaluations,
        VerificationBuilderImpl,
    },
};
use alloc::collections::VecDeque;
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn empty_exec_reports_empty_metadata() {
    let empty = EmptyExec::new();

    assert_eq!(EmptyExec::default(), empty);
    assert!(empty.get_column_result_fields().is_empty());
    assert!(empty.get_column_references().is_empty());
    assert!(empty.get_table_references().is_empty());
}

#[test]
fn empty_exec_verifier_returns_singleton_chi_for_one_empty_row() {
    let empty = EmptyExec::new();
    let singleton_chi_evaluation = TestScalar::from(17u64);
    let mle_evaluations = SumcheckMleEvaluations {
        singleton_chi_evaluation,
        ..Default::default()
    };
    let mut builder = VerificationBuilderImpl::new(
        mle_evaluations,
        &[][..],
        &[][..],
        VecDeque::new(),
        Vec::new(),
        Vec::new(),
        0,
    );
    let accessor: IndexMap<TableRef, IndexMap<Ident, TestScalar>> = IndexMap::default();
    let chi_eval_map: IndexMap<TableRef, (TestScalar, usize)> = IndexMap::default();

    let table_evaluation = empty
        .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
        .unwrap();

    assert!(table_evaluation.column_evals().is_empty());
    assert_eq!(table_evaluation.chi(), (singleton_chi_evaluation, 1));
}

#[test]
fn empty_exec_prover_rounds_return_one_row_without_columns() {
    let empty = EmptyExec::new();
    let alloc = Bump::new();
    let table_map: IndexMap<TableRef, Table<'_, TestScalar>> = IndexMap::default();
    let params: &[LiteralValue] = &[];

    let mut first_round_builder = FirstRoundBuilder::new(0);
    let first_round_table = empty
        .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, params)
        .unwrap();
    assert!(first_round_table.is_empty());
    assert_eq!(first_round_table.num_columns(), 0);
    assert_eq!(first_round_table.num_rows(), 1);

    let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
    let final_round_table = empty
        .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, params)
        .unwrap();
    assert!(final_round_table.is_empty());
    assert_eq!(final_round_table.num_columns(), 0);
    assert_eq!(final_round_table.num_rows(), 1);
}
