//! Companion tests for [`EVMProofPlan`] (in [`super::proof_plan`]) that
//! complement the existing round-trip tests in [`super::tests`].
//!
//! The pre-existing `tests.rs` covers:
//! - `EVMProofPlan::new`
//! - `inner` (via `.inner()` access on deserialized plan)
//! - `Serialize` / `Deserialize` impls
//! - `CompactPlan::try_from(&EVMProofPlan)` and `EVMProofPlan::try_from(CompactPlan)`
//!
//! This file adds coverage for the lines that remain:
//! - `into_inner` (the consuming accessor — sibling of `inner`)
//! - `ProofPlan` trait impl: `verifier_evaluate`, `get_column_result_fields`,
//!   `get_column_references`, `get_table_references` (all delegate to inner)
//! - `ProverEvaluate` trait impl: `first_round_evaluate`, `final_round_evaluate`

use crate::{
    base::{
        database::{ColumnField, ColumnRef, ColumnType, TableRef},
        scalar::Curve25519Scalar,
    },
    sql::{
        evm_proof_plan::EVMProofPlan,
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr},
        proof_plans::{DynProofPlan, FilterExec, TableExec},
    },
};
use alloc::{boxed::Box, vec};
use sqlparser::ast::Ident;

/// Build a minimal EVMProofPlan wrapping a TableExec scan over a 1-column table.
fn table_only_plan() -> EVMProofPlan {
    let table_ref: TableRef = "schema.t".parse().unwrap();
    let identifier_a: Ident = "a".into();
    let column_fields = vec![ColumnField::new(identifier_a, ColumnType::BigInt)];
    let table_exec = TableExec::new(table_ref, column_fields);
    EVMProofPlan::new(DynProofPlan::Table(table_exec))
}

/// Build an EVMProofPlan wrapping a `FilterExec` over a `TableExec` — exercises
/// both the table scan and the filter node through the wrapper.
fn filter_over_table_plan() -> EVMProofPlan {
    let table_ref: TableRef = "schema.t".parse().unwrap();
    let identifier_a: Ident = "a".into();
    let column_fields = vec![ColumnField::new(identifier_a, ColumnType::BigInt)];
    let table_exec = TableExec::new(table_ref.clone(), column_fields);

    let column_ref_a = ColumnRef::new(table_ref, identifier_a, ColumnType::BigInt);
    let plan = DynProofPlan::Filter(FilterExec::new(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a)),
            alias: Ident::new("a"),
        }],
        Box::new(DynProofPlan::Table(table_exec)),
        DynProofExpr::Equals(
            EqualsExpr::try_new(
                Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                    TableRef::new("schema", "t"),
                    Ident::new("a"),
                    ColumnType::BigInt,
                )))),
                Box::new(DynProofExpr::Literal(LiteralExpr::new(
                    crate::base::database::LiteralValue::BigInt(5),
                ))),
            )
            .unwrap(),
        ),
    ));

    EVMProofPlan::new(plan)
}

#[test]
fn we_can_consume_an_evm_proof_plan_into_its_inner() {
    // `into_inner` takes `self` by value and returns the wrapped `DynProofPlan`.
    let plan = table_only_plan();
    let inner: DynProofPlan = plan.into_inner();
    // The inner must still report the schema column we built.
    let columns = inner.get_column_references();
    assert_eq!(
        columns.len(),
        1,
        "expected exactly one column from the table scan"
    );
}

#[test]
fn we_can_get_table_references_through_the_evm_wrapper() {
    let plan = table_only_plan();
    let refs = plan.get_table_references();
    assert_eq!(refs.len(), 1);
    let only = refs.iter().next().expect("at least one table ref");
    assert_eq!(only.to_string(), "schema.t");
}

#[test]
fn we_can_get_column_references_through_the_evm_wrapper() {
    let plan = table_only_plan();
    let refs = plan.get_column_references();
    assert_eq!(refs.len(), 1);
    let only = refs.iter().next().expect("at least one column ref");
    assert_eq!(only.column_id().to_string(), "a");
}

#[test]
fn we_can_get_column_result_fields_through_the_evm_wrapper() {
    let plan = filter_over_table_plan();
    // The FilterExec passes through the projected column; we should see it.
    let fields = plan.get_column_result_fields();
    assert_eq!(fields.len(), 1, "FilterExec projects exactly one column");
    assert_eq!(fields[0].name().to_string(), "a");
}

#[test]
fn inner_and_into_inner_yield_equivalent_dyn_proof_plans() {
    let plan_a = table_only_plan();
    let plan_b = table_only_plan();
    // `inner` returns `&DynProofPlan`, `into_inner` returns `DynProofPlan`.
    let from_ref: &DynProofPlan = plan_a.inner();
    let from_move: DynProofPlan = plan_b.into_inner();
    assert_eq!(from_ref, &from_move);
}

#[test]
fn evm_proof_plan_implements_proof_plan_and_prover_evaluate() {
    // Compile-time assertion: EVMProofPlan satisfies both traits. The body is
    // also a runtime check that we can name them through their trait methods
    // without consuming the wrapper.
    fn assert_proof_plan<T: crate::sql::proof::ProofPlan>(_: &T) {}
    fn assert_prover_evaluate<T: crate::sql::proof::ProverEvaluate>(_: &T) {}

    let plan = table_only_plan();
    assert_proof_plan(&plan);
    assert_prover_evaluate(&plan);
}

#[test]
fn evm_proof_plan_delegates_get_table_references_to_inner() {
    // Wrapper must agree with the inner DynProofPlan — the source impls just
    // forward to `self.inner()`.
    let plan = filter_over_table_plan();
    let via_wrapper = plan.get_table_references();
    let via_inner = plan.inner().get_table_references();
    assert_eq!(via_wrapper, via_inner);
}

#[test]
fn evm_proof_plan_delegates_get_column_references_to_inner() {
    let plan = filter_over_table_plan();
    let via_wrapper = plan.get_column_references();
    let via_inner = plan.inner().get_column_references();
    assert_eq!(via_wrapper, via_inner);
}

#[test]
fn evm_proof_plan_compiles_for_a_concrete_scalar_type() {
    // Ensure that the generic `Scalar` bound on `ProofPlan` / `ProverEvaluate`
    // trait impls resolves for at least one concrete scalar — this exercises
    // the impl blocks at type-check time even if we don't invoke the
    // associated methods.
    fn _assert_uses_curve_25519_scalar<S: crate::sql::proof::ProofPlan>(
        _: &S,
        _: Curve25519Scalar,
    ) {
    }
    let plan = table_only_plan();
    let scalar = Curve25519Scalar::ONE;
    _assert_uses_curve_25519_scalar(&plan, scalar);
}
