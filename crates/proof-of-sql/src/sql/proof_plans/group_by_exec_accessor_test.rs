use super::{test_utility::*, DynProofPlan, GroupByExec};
use crate::{
    base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{table_utility::*, TableRef, TableTestAccessor, TestAccessor},
    },
    sql::{proof_exprs::test_utility::*, proof_exprs::TableExpr},
};
use bumpalo::Bump;

#[test]
fn we_can_get_fields_and_uniqueness_of_group_by_exec() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
        borrowed_varchar("name", ["alpha", "beta", "gamma"], &alloc),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);

    let table_expr = TableExpr {
        table_ref: t.clone(),
    };
    let group_by_exprs = cols_expr(&t, &["a"], &accessor);
    let sum_exprs = vec![sum_expr(column(&t, "b", &accessor), "sum_b")];
    let where_clause = equal(column(&t, "a", &accessor), const_int128(2_i128));
    let plan = group_by(
        group_by_exprs.clone(),
        sum_exprs.clone(),
        "row_count",
        table_expr.clone(),
        where_clause.clone(),
    );

    if let DynProofPlan::GroupBy(plan) = plan {
        assert_eq!(plan.group_by_exprs(), &group_by_exprs);
        assert_eq!(plan.sum_expr(), &sum_exprs);
        assert_eq!(plan.count_alias(), &"row_count".into());
        assert_eq!(plan.table(), &table_expr);
        assert_eq!(plan.where_clause(), &where_clause);
        assert_eq!(plan.try_get_is_uniqueness_provable(), Some(true));
    } else {
        panic!("Expected GroupByExec plan");
    }

    let no_group_by_plan = group_by(
        vec![],
        vec![sum_expr(column(&t, "b", &accessor), "sum_b")],
        "row_count",
        table_expr.clone(),
        const_bool(true),
    );
    if let DynProofPlan::GroupBy(plan) = no_group_by_plan {
        assert_eq!(plan.try_get_is_uniqueness_provable(), Some(false));
    } else {
        panic!("Expected GroupByExec plan");
    }

    assert!(GroupByExec::try_new(
        cols_expr(&t, &["name"], &accessor),
        vec![sum_expr(column(&t, "b", &accessor), "sum_b")],
        "row_count".into(),
        table_expr,
        const_bool(true),
    )
    .is_none());
}
