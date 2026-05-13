use super::EmptyExec;
use crate::{
    base::{
        database::{ColumnField, ColumnRef, LiteralValue, TableRef},
        map::{IndexMap, IndexSet},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::{FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate},
};
use alloc::{collections::VecDeque, vec::Vec};
use bumpalo::Bump;

#[test]
fn empty_exec_has_no_references_or_columns() {
    let plan = EmptyExec::default();

    assert_eq!(plan, EmptyExec::new());
    assert_eq!(plan.get_column_result_fields(), Vec::<ColumnField>::new());
    assert_eq!(
        plan.get_column_references(),
        IndexSet::<ColumnRef>::default()
    );
    assert_eq!(plan.get_table_references(), IndexSet::<TableRef>::default());
}

#[test]
fn empty_exec_first_round_evaluates_to_one_empty_row() {
    let alloc = Bump::new();
    let plan = EmptyExec::new();
    let mut builder = FirstRoundBuilder::new(1);
    let table_map = IndexMap::default();
    let params = Vec::<LiteralValue>::new();

    let result = plan
        .first_round_evaluate::<Curve25519Scalar>(&mut builder, &alloc, &table_map, &params)
        .unwrap();

    assert_eq!(result.num_rows(), 1);
    assert_eq!(result.num_columns(), 0);
}

#[test]
fn empty_exec_final_round_evaluates_to_one_empty_row() {
    let alloc = Bump::new();
    let plan = EmptyExec::new();
    let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
    let table_map = IndexMap::default();
    let params = Vec::<LiteralValue>::new();

    let result = plan
        .final_round_evaluate::<Curve25519Scalar>(&mut builder, &alloc, &table_map, &params)
        .unwrap();

    assert_eq!(result.num_rows(), 1);
    assert_eq!(result.num_columns(), 0);
}
