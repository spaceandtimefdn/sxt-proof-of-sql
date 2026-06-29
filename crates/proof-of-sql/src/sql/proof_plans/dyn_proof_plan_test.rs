use super::DynProofPlan;
use crate::base::{
    database::{ColumnField, ColumnRef, ColumnType, TableRef},
    map::IndexSet,
};

#[test]
fn result_fields_are_exposed_as_unqualified_column_refs() {
    let table_ref = TableRef::new("sxt", "orders");
    let plan = DynProofPlan::new_table(
        table_ref,
        vec![
            ColumnField::new("order_id".into(), ColumnType::BigInt),
            ColumnField::new("amount".into(), ColumnType::Int128),
        ],
    );

    let result_refs = plan.get_column_result_fields_as_references();
    let expected_refs = IndexSet::from_iter([
        ColumnRef::new(
            TableRef::from_names(None, ""),
            "order_id".into(),
            ColumnType::BigInt,
        ),
        ColumnRef::new(
            TableRef::from_names(None, ""),
            "amount".into(),
            ColumnType::Int128,
        ),
    ]);

    assert_eq!(result_refs, expected_refs);
}
