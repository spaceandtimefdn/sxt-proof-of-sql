use super::{test_utility::*, DynProofPlan};
use crate::{
    base::database::{ColumnField, ColumnRef, ColumnType, TableRef},
    sql::proof_exprs::test_utility::*,
};

fn test_table_ref() -> TableRef {
    TableRef::new("sxt", "t")
}

fn test_schema() -> Vec<ColumnField> {
    vec![
        column_field("a", ColumnType::BigInt),
        column_field("b", ColumnType::Boolean),
    ]
}

fn test_table_plan() -> DynProofPlan {
    DynProofPlan::new_table(test_table_ref(), test_schema())
}

#[test]
fn constructors_build_expected_plan_variants() {
    assert!(matches!(DynProofPlan::new_empty(), DynProofPlan::Empty(_)));

    let table_ref = test_table_ref();
    let schema = test_schema();
    let table_plan = DynProofPlan::new_table(table_ref.clone(), schema.clone());
    if let DynProofPlan::Table(plan) = &table_plan {
        assert_eq!(plan.table_ref(), &table_ref);
        assert_eq!(plan.schema(), schema);
    } else {
        panic!("expected table plan");
    }

    let result = vec![aliased_plan(const_bigint(1), "one")];
    let projection = DynProofPlan::new_projection(result.clone(), table_plan.clone());
    if let DynProofPlan::Projection(plan) = &projection {
        assert_eq!(plan.aliased_results(), result);
        assert_eq!(plan.input(), &table_plan);
    } else {
        panic!("expected projection plan");
    }

    let filter = DynProofPlan::new_filter(result.clone(), table_plan.clone(), const_bool(true));
    if let DynProofPlan::Filter(plan) = &filter {
        assert_eq!(plan.aliased_results(), result);
        assert_eq!(plan.input(), &table_plan);
        assert_eq!(plan.where_clause(), &const_bool(true));
    } else {
        panic!("expected filter plan");
    }

    let legacy_filter =
        DynProofPlan::new_legacy_filter(result.clone(), tab(&table_ref), const_bool(true));
    if let DynProofPlan::LegacyFilter(plan) = &legacy_filter {
        assert_eq!(plan.aliased_results(), result);
        assert_eq!(plan.table(), &tab(&table_ref));
        assert_eq!(plan.where_clause(), &const_bool(true));
    } else {
        panic!("expected legacy filter plan");
    }

    let group_by = DynProofPlan::try_new_group_by(
        vec![],
        vec![],
        "count".into(),
        tab(&table_ref),
        const_bool(true),
    )
    .unwrap();
    if let DynProofPlan::GroupBy(plan) = &group_by {
        assert!(plan.group_by_exprs().is_empty());
        assert!(plan.sum_expr().is_empty());
        assert_eq!(plan.count_alias(), &"count".into());
        assert_eq!(plan.table(), &tab(&table_ref));
    } else {
        panic!("expected group by plan");
    }

    let aggregate = DynProofPlan::try_new_aggregate(
        vec![],
        vec![],
        "count".into(),
        table_plan.clone(),
        const_bool(true),
    )
    .unwrap();
    if let DynProofPlan::Aggregate(plan) = &aggregate {
        assert!(plan.group_by_exprs().is_empty());
        assert!(plan.sum_expr().is_empty());
        assert_eq!(plan.count_alias(), &"count".into());
        assert_eq!(plan.input(), &table_plan);
    } else {
        panic!("expected aggregate plan");
    }

    let slice = DynProofPlan::new_slice(table_plan.clone(), 2, Some(5));
    if let DynProofPlan::Slice(plan) = &slice {
        assert_eq!(plan.input(), &table_plan);
        assert_eq!(plan.skip(), 2);
        assert_eq!(plan.fetch(), Some(5));
    } else {
        panic!("expected slice plan");
    }

    let union = DynProofPlan::try_new_union(vec![table_plan.clone(), table_plan]).unwrap();
    if let DynProofPlan::Union(plan) = &union {
        assert_eq!(plan.input_plans().len(), 2);
    } else {
        panic!("expected union plan");
    }
}

#[test]
fn result_fields_can_be_viewed_as_column_references() {
    let projection = DynProofPlan::new_projection(
        vec![
            aliased_plan(const_bigint(7), "answer"),
            aliased_plan(const_bool(true), "ok"),
        ],
        test_table_plan(),
    );

    let refs = projection.get_column_result_fields_as_references();

    assert!(refs.contains(&ColumnRef::new(
        TableRef::from_names(None, ""),
        "answer".into(),
        ColumnType::BigInt,
    )));
    assert!(refs.contains(&ColumnRef::new(
        TableRef::from_names(None, ""),
        "ok".into(),
        ColumnType::Boolean,
    )));
}
