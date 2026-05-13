use super::{test_utility::*, AggregateExec, DynProofPlan};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            table_utility::*, ColumnField, ColumnType, TableRef, TableTestAccessor, TestAccessor,
        },
    },
    sql::proof_exprs::test_utility::*,
};
use bumpalo::Bump;

#[test]
fn we_can_get_fields_and_uniqueness_of_aggregate_exec() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
        borrowed_varchar("name", ["alpha", "beta", "gamma"], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    let table_exec = table_exec(
        t.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
            ColumnField::new("name".into(), ColumnType::VarChar),
        ],
    );
    let group_by_exprs = cols_expr_plan(&t, &["a"], &accessor);
    let sum_exprs = vec![sum_expr(column(&t, "b", &accessor), "sum_b")];
    let where_clause = equal(column(&t, "a", &accessor), const_int128(2_i128));
    let plan = aggregate(
        group_by_exprs.clone(),
        sum_exprs.clone(),
        "row_count",
        table_exec.clone(),
        where_clause.clone(),
    );

    if let DynProofPlan::Aggregate(plan) = plan {
        assert_eq!(plan.group_by_exprs(), &group_by_exprs);
        assert_eq!(plan.sum_expr(), &sum_exprs);
        assert_eq!(plan.count_alias(), &"row_count".into());
        assert_eq!(plan.input(), &table_exec);
        assert_eq!(plan.where_clause(), &where_clause);
        assert_eq!(plan.try_get_is_uniqueness_provable(), Some(true));
    } else {
        panic!("Expected AggregateExec plan");
    }

    let no_group_by_plan = aggregate(
        vec![],
        vec![sum_expr(column(&t, "b", &accessor), "sum_b")],
        "row_count",
        table_exec.clone(),
        const_bool(true),
    );
    if let DynProofPlan::Aggregate(plan) = no_group_by_plan {
        assert_eq!(plan.try_get_is_uniqueness_provable(), Some(false));
    } else {
        panic!("Expected AggregateExec plan");
    }

    assert!(AggregateExec::try_new(
        cols_expr_plan(&t, &["name"], &accessor),
        vec![sum_expr(column(&t, "b", &accessor), "sum_b")],
        "row_count".into(),
        Box::new(table_exec),
        const_bool(true),
    )
    .is_none());
}
