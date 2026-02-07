use super::{DynProofPlan, EmptyExec, TableExec};
use crate::base::database::{ColumnField, ColumnType, TableRef};
use alloc::vec;

#[test]
fn dyn_proof_plan_new_empty_returns_empty_variant() {
    let plan = DynProofPlan::new_empty();
    match plan {
        DynProofPlan::Empty(_) => {}
        _ => panic!("Expected Empty variant"),
    }
}

#[test]
fn dyn_proof_plan_new_table_returns_table_variant() {
    let table_ref = "namespace.table".parse::<TableRef>().unwrap();
    let fields = vec![ColumnField::new("col1".into(), ColumnType::BigInt)];
    let plan = DynProofPlan::new_table(table_ref, fields);
    match plan {
        DynProofPlan::Table(_) => {}
        _ => panic!("Expected Table variant"),
    }
}

#[test]
fn dyn_proof_plan_get_column_result_fields_as_references() {
    let table_ref = "namespace.table".parse::<TableRef>().unwrap();
    let fields = vec![ColumnField::new("col1".into(), ColumnType::BigInt)];
    let plan = DynProofPlan::new_table(table_ref, fields);

    let refs = plan.get_column_result_fields_as_references();
    assert_eq!(refs.len(), 1);
    let col_ref = refs.first().unwrap();
    assert_eq!(col_ref.column_id().value.as_str(), "col1");
    // The table_ref in get_column_result_fields_as_references is hardcoded to empty/none
    assert_eq!(col_ref.table_ref().to_string(), "");
}

#[test]
fn dyn_proof_plan_dispatch_works() {
    use crate::sql::proof::ProofPlan;

    let plan = DynProofPlan::new_empty();
    // This should call EmptyExec::get_column_result_fields via enum_dispatch
    assert!(plan.get_column_result_fields().is_empty());
}
