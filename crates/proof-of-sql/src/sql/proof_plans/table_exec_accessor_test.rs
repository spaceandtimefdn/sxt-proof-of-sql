use super::TableExec;
use crate::{
    base::{
        database::{ColumnField, ColumnRef, ColumnType, TableRef},
        map::indexset,
    },
    sql::proof::ProofPlan,
};

#[test]
fn we_can_get_table_exec_accessors_and_references() {
    let table_ref = TableRef::new("namespace", "orders");
    let schema = vec![
        ColumnField::new("order_id".into(), ColumnType::BigInt),
        ColumnField::new("customer".into(), ColumnType::VarChar),
        ColumnField::new("is_priority".into(), ColumnType::Boolean),
    ];
    let plan = TableExec::new(table_ref.clone(), schema.clone());

    assert_eq!(plan.table_ref(), &table_ref);
    assert_eq!(plan.schema(), schema.as_slice());
    assert_eq!(plan.get_column_result_fields(), schema);
    assert_eq!(plan.get_table_references(), indexset! {table_ref.clone()});
    assert_eq!(
        plan.get_column_references(),
        indexset! {
            ColumnRef::new(table_ref.clone(), "order_id".into(), ColumnType::BigInt),
            ColumnRef::new(table_ref.clone(), "customer".into(), ColumnType::VarChar),
            ColumnRef::new(table_ref, "is_priority".into(), ColumnType::Boolean),
        }
    );
}

#[test]
fn table_exec_allows_an_empty_schema_for_count_like_plans() {
    let table_ref = TableRef::new("namespace", "empty_projection");
    let plan = TableExec::new(table_ref.clone(), vec![]);

    assert_eq!(plan.table_ref(), &table_ref);
    assert!(plan.schema().is_empty());
    assert!(plan.get_column_result_fields().is_empty());
    assert!(plan.get_column_references().is_empty());
    assert_eq!(plan.get_table_references(), indexset! {table_ref});
}
