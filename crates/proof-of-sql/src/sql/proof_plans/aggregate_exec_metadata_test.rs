use super::{test_utility::table_exec, AggregateExec};
use crate::{
    base::database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
    sql::{
        proof::ProofPlan,
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr},
    },
};

fn table_schema() -> Vec<ColumnField> {
    vec![
        ColumnField::new("a".into(), ColumnType::BigInt),
        ColumnField::new("b".into(), ColumnType::Boolean),
        ColumnField::new("c".into(), ColumnType::BigInt),
        ColumnField::new("name".into(), ColumnType::VarChar),
    ]
}

fn column_ref(table_ref: &TableRef, name: &str, column_type: ColumnType) -> ColumnRef {
    ColumnRef::new(table_ref.clone(), name.into(), column_type)
}

fn column(table_ref: &TableRef, name: &str, column_type: ColumnType) -> DynProofExpr {
    DynProofExpr::Column(ColumnExpr::new(column_ref(table_ref, name, column_type)))
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

fn count_all_filter() -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Boolean(true))
}

#[test]
fn aggregate_exec_reports_result_fields_and_references() {
    let table_ref = TableRef::new("sxt", "t");
    let aggregate = AggregateExec::try_new(
        vec![aliased_column(
            &table_ref,
            "a",
            ColumnType::BigInt,
            "group_a",
        )],
        vec![aliased_column(&table_ref, "c", ColumnType::BigInt, "sum_c")],
        "__count__".into(),
        Box::new(table_exec(table_ref.clone(), table_schema())),
        count_all_filter(),
    )
    .unwrap();

    assert_eq!(
        aggregate.get_column_result_fields(),
        vec![
            ColumnField::new("group_a".into(), ColumnType::BigInt),
            ColumnField::new("sum_c".into(), ColumnType::BigInt),
            ColumnField::new("__count__".into(), ColumnType::BigInt),
        ]
    );

    let column_refs = aggregate.get_column_references();
    assert_eq!(column_refs.len(), 4);
    assert!(column_refs.contains(&column_ref(&table_ref, "a", ColumnType::BigInt)));
    assert!(column_refs.contains(&column_ref(&table_ref, "b", ColumnType::Boolean)));
    assert!(column_refs.contains(&column_ref(&table_ref, "c", ColumnType::BigInt)));
    assert!(column_refs.contains(&column_ref(&table_ref, "name", ColumnType::VarChar)));

    let table_refs = aggregate.get_table_references();
    assert_eq!(table_refs.len(), 1);
    assert!(table_refs.contains(&table_ref));
}

#[test]
fn aggregate_exec_distinguishes_uniqueness_modes() {
    let table_ref = TableRef::new("sxt", "t");
    let input = table_exec(table_ref.clone(), table_schema());
    let sum_expr = vec![aliased_column(&table_ref, "c", ColumnType::BigInt, "sum_c")];

    let ungrouped = AggregateExec::try_new(
        vec![],
        sum_expr.clone(),
        "__count__".into(),
        Box::new(input.clone()),
        count_all_filter(),
    )
    .unwrap();
    assert_eq!(ungrouped.try_get_is_uniqueness_provable(), Some(false));

    let numeric_group = AggregateExec::try_new(
        vec![aliased_column(
            &table_ref,
            "a",
            ColumnType::BigInt,
            "group_a",
        )],
        sum_expr.clone(),
        "__count__".into(),
        Box::new(input.clone()),
        count_all_filter(),
    )
    .unwrap();
    assert_eq!(numeric_group.try_get_is_uniqueness_provable(), Some(true));

    assert!(AggregateExec::try_new(
        vec![aliased_column(
            &table_ref,
            "name",
            ColumnType::VarChar,
            "group_name",
        )],
        sum_expr,
        "__count__".into(),
        Box::new(input),
        count_all_filter(),
    )
    .is_none());
}
