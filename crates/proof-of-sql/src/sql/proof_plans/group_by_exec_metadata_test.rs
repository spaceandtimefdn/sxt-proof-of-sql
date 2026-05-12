use super::GroupByExec;
use crate::{
    base::database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
    sql::{
        proof::ProofPlan,
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
    },
};

fn table_ref() -> TableRef {
    TableRef::new("sxt", "t")
}

fn column_ref(table_ref: &TableRef, name: &str, column_type: ColumnType) -> ColumnRef {
    ColumnRef::new(table_ref.clone(), name.into(), column_type)
}

fn column_expr(table_ref: &TableRef, name: &str, column_type: ColumnType) -> ColumnExpr {
    ColumnExpr::new(column_ref(table_ref, name, column_type))
}

fn column(table_ref: &TableRef, name: &str, column_type: ColumnType) -> DynProofExpr {
    DynProofExpr::Column(column_expr(table_ref, name, column_type))
}

fn aliased_column(
    table_ref: &TableRef,
    name: &str,
    column_type: ColumnType,
    alias: &str,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: column(table_ref, name, column_type),
        alias: alias.into(),
    }
}

fn table_expr(table_ref: &TableRef) -> TableExpr {
    TableExpr {
        table_ref: table_ref.clone(),
    }
}

fn count_all_filter() -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Boolean(true))
}

#[test]
fn group_by_exec_reports_result_fields_and_references() {
    let table_ref = table_ref();
    let group_by = GroupByExec::try_new(
        vec![column_expr(&table_ref, "a", ColumnType::BigInt)],
        vec![aliased_column(&table_ref, "c", ColumnType::BigInt, "sum_c")],
        "__count__".into(),
        table_expr(&table_ref),
        count_all_filter(),
    )
    .unwrap();

    assert_eq!(
        group_by.get_column_result_fields(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("sum_c".into(), ColumnType::BigInt),
            ColumnField::new("__count__".into(), ColumnType::BigInt),
        ]
    );

    let column_refs = group_by.get_column_references();
    assert_eq!(column_refs.len(), 2);
    assert!(column_refs.contains(&column_ref(&table_ref, "a", ColumnType::BigInt)));
    assert!(column_refs.contains(&column_ref(&table_ref, "c", ColumnType::BigInt)));

    let table_refs = group_by.get_table_references();
    assert_eq!(table_refs.len(), 1);
    assert!(table_refs.contains(&table_ref));
}

#[test]
fn group_by_exec_distinguishes_uniqueness_modes() {
    let table_ref = table_ref();
    let sum_expr = vec![aliased_column(&table_ref, "c", ColumnType::BigInt, "sum_c")];

    let ungrouped = GroupByExec::try_new(
        vec![],
        sum_expr.clone(),
        "__count__".into(),
        table_expr(&table_ref),
        count_all_filter(),
    )
    .unwrap();
    assert_eq!(ungrouped.try_get_is_uniqueness_provable(), Some(false));

    let numeric_group = GroupByExec::try_new(
        vec![column_expr(&table_ref, "a", ColumnType::BigInt)],
        sum_expr.clone(),
        "__count__".into(),
        table_expr(&table_ref),
        count_all_filter(),
    )
    .unwrap();
    assert_eq!(numeric_group.try_get_is_uniqueness_provable(), Some(true));

    assert!(GroupByExec::try_new(
        vec![column_expr(&table_ref, "name", ColumnType::VarChar)],
        sum_expr,
        "__count__".into(),
        table_expr(&table_ref),
        count_all_filter(),
    )
    .is_none());
}
