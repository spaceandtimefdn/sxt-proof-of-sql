use super::empty_exec::EmptyExec;
use crate::sql::proof::ProofPlan;

#[test]
fn we_can_create_an_empty_exec() {
    let plan = EmptyExec::new();
    assert_eq!(plan, EmptyExec::new());
}

#[test]
fn we_can_create_an_empty_exec_via_default() {
    let plan = EmptyExec::default();
    assert_eq!(plan, EmptyExec::new());
}

#[test]
fn empty_exec_has_no_column_result_fields() {
    let plan = EmptyExec::new();
    assert!(plan.get_column_result_fields().is_empty());
}

#[test]
fn empty_exec_has_no_column_references() {
    let plan = EmptyExec::new();
    assert!(plan.get_column_references().is_empty());
}

#[test]
fn empty_exec_has_no_table_references() {
    let plan = EmptyExec::new();
    assert!(plan.get_table_references().is_empty());
}

#[test]
fn empty_exec_can_be_cloned() {
    let plan = EmptyExec::new();
    let cloned = plan.clone();
    assert_eq!(plan, cloned);
}

#[test]
fn empty_exec_can_be_debug_printed() {
    let plan = EmptyExec::new();
    let debug_str = format!("{plan:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn empty_exec_can_be_serialized_and_deserialized() {
    let plan = EmptyExec::new();
    let serialized = serde_json::to_string(&plan).unwrap();
    let deserialized: EmptyExec = serde_json::from_str(&serialized).unwrap();
    assert_eq!(plan, deserialized);
}
