use super::{
    DynProofPlan, EmptyExec, GroupByExec, LegacyFilterExec, ProjectionExec, SliceExec,
    SortMergeJoinExec, TableExec, UnionExec,
};
use crate::{
    base::database::{ColumnField, ColumnType, TableRef},
    sql::proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
};
use alloc::boxed::Box;
use sqlparser::ast::Ident;

pub fn column_field(name: &str, column_type: ColumnType) -> ColumnField {
    ColumnField::new(name.into(), column_type)
}

pub fn empty_exec() -> DynProofPlan {
    DynProofPlan::Empty(EmptyExec::new())
}

pub fn table_exec(table_ref: TableRef, schema: Vec<ColumnField>) -> DynProofPlan {
    DynProofPlan::Table(TableExec::new(table_ref, schema))
}

pub fn projection(results: Vec<AliasedDynProofExpr>, input: DynProofPlan) -> DynProofPlan {
    DynProofPlan::Projection(ProjectionExec::new(results, Box::new(input)))
}

pub fn legacy_filter(
    results: Vec<AliasedDynProofExpr>,
    table: TableExpr,
    where_clause: DynProofExpr,
) -> DynProofPlan {
    DynProofPlan::LegacyFilter(LegacyFilterExec::new(results, table, where_clause))
}

pub fn filter(
    results: Vec<AliasedDynProofExpr>,
    input: DynProofPlan,
    where_clause: DynProofExpr,
) -> DynProofPlan {
    DynProofPlan::new_filter(results, input, where_clause)
}

/// # Panics
///
/// Will panic if `count_alias` cannot be parsed as a valid identifier.
pub fn group_by(
    group_by_exprs: Vec<ColumnExpr>,
    sum_expr: Vec<AliasedDynProofExpr>,
    count_alias: &str,
    table: TableExpr,
    where_clause: DynProofExpr,
) -> DynProofPlan {
    DynProofPlan::GroupBy(
        GroupByExec::try_new(
            group_by_exprs,
            sum_expr,
            count_alias.into(),
            table,
            where_clause,
        )
        .unwrap(),
    )
}

/// # Panics
///
/// Will panic if `count_alias` cannot be parsed as a valid identifier.
pub fn aggregate(
    group_by_exprs: Vec<AliasedDynProofExpr>,
    sum_expr: Vec<AliasedDynProofExpr>,
    count_alias: &str,
    input: DynProofPlan,
    where_clause: DynProofExpr,
) -> DynProofPlan {
    DynProofPlan::try_new_aggregate(
        group_by_exprs,
        sum_expr,
        count_alias.into(),
        input,
        where_clause,
    )
    .unwrap()
}

pub fn slice_exec(input: DynProofPlan, skip: usize, fetch: Option<usize>) -> DynProofPlan {
    DynProofPlan::Slice(SliceExec::new(Box::new(input), skip, fetch))
}

pub fn union_exec(inputs: Vec<DynProofPlan>) -> DynProofPlan {
    DynProofPlan::Union(UnionExec::try_new(inputs).unwrap())
}

pub fn sort_merge_join(
    left: DynProofPlan,
    right: DynProofPlan,
    left_join_column_indexes: Vec<usize>,
    right_join_column_indexes: Vec<usize>,
    result_idents: Vec<Ident>,
) -> DynProofPlan {
    DynProofPlan::SortMergeJoin(SortMergeJoinExec::new(
        Box::new(left),
        Box::new(right),
        left_join_column_indexes,
        right_join_column_indexes,
        result_idents,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            commitment::naive_evaluation_proof::NaiveEvaluationProof,
            database::{table_utility::*, TableTestAccessor},
        },
        sql::proof_exprs::{test_utility::*, ProofExpr},
    };
    use bumpalo::Bump;

    #[test]
    fn test_utility_builds_proof_plan_variants() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "proof_plan_test_utility");
        let data = table([
            borrowed_bigint("id", [1, 2], &alloc),
            borrowed_int("value", [3, 4], &alloc),
        ]);
        let accessor = TableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            data,
            0,
            (),
        );
        let schema = vec![
            column_field("id", ColumnType::BigInt),
            column_field("value", ColumnType::Int),
        ];

        assert_eq!(schema[0].name().to_string(), "id");
        assert_eq!(schema[0].data_type(), ColumnType::BigInt);

        assert!(matches!(empty_exec(), DynProofPlan::Empty(_)));

        let table_plan = table_exec(table_ref.clone(), schema.clone());
        match &table_plan {
            DynProofPlan::Table(plan) => {
                assert_eq!(plan.table_ref(), &table_ref);
                assert_eq!(plan.schema(), schema.as_slice());
            }
            _ => panic!("expected table plan"),
        }

        let aliased_id = col_expr_plan(&table_ref, "id", &accessor);
        let projected = projection(vec![aliased_id.clone()], table_plan.clone());
        match &projected {
            DynProofPlan::Projection(plan) => {
                assert_eq!(plan.aliased_results(), &[aliased_id.clone()]);
                assert_eq!(plan.input(), &table_plan);
            }
            _ => panic!("expected projection plan"),
        }

        let table_expr = tab(&table_ref);
        let where_clause = equal(column(&table_ref, "id", &accessor), const_bigint(1));
        let legacy_filtered = legacy_filter(
            vec![aliased_id.clone()],
            table_expr.clone(),
            where_clause.clone(),
        );
        match &legacy_filtered {
            DynProofPlan::LegacyFilter(plan) => {
                assert_eq!(plan.aliased_results(), &[aliased_id.clone()]);
                assert_eq!(plan.table(), &table_expr);
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected legacy filter plan"),
        }

        let filtered = filter(
            vec![aliased_id.clone()],
            table_plan.clone(),
            where_clause.clone(),
        );
        match &filtered {
            DynProofPlan::Filter(plan) => {
                assert_eq!(plan.aliased_results(), &[aliased_id.clone()]);
                assert_eq!(plan.input(), &table_plan);
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected filter plan"),
        }

        let id_expr = col_expr(&table_ref, "id", &accessor);
        let value_sum = sum_expr(column(&table_ref, "value", &accessor), "value_sum");
        let grouped = group_by(
            vec![id_expr.clone()],
            vec![value_sum.clone()],
            "row_count",
            table_expr.clone(),
            where_clause.clone(),
        );
        match &grouped {
            DynProofPlan::GroupBy(plan) => {
                assert_eq!(plan.group_by_exprs(), &[id_expr.clone()]);
                assert_eq!(plan.sum_expr(), &[value_sum.clone()]);
                assert_eq!(plan.count_alias().to_string(), "row_count");
                assert_eq!(plan.table(), &table_expr);
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected group-by plan"),
        }

        let aggregate_group = aliased_plan(column(&table_ref, "id", &accessor), "id");
        let aggregated = aggregate(
            vec![aggregate_group.clone()],
            vec![value_sum.clone()],
            "row_count",
            table_plan.clone(),
            where_clause.clone(),
        );
        match &aggregated {
            DynProofPlan::Aggregate(plan) => {
                assert_eq!(plan.group_by_exprs(), &[aggregate_group.clone()]);
                assert_eq!(plan.sum_expr(), &[value_sum.clone()]);
                assert_eq!(plan.count_alias().to_string(), "row_count");
                assert_eq!(plan.input(), &table_plan);
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected aggregate plan"),
        }

        let sliced = slice_exec(table_plan.clone(), 1, Some(2));
        match &sliced {
            DynProofPlan::Slice(plan) => {
                assert_eq!(plan.input(), &table_plan);
                assert_eq!(plan.skip(), 1);
                assert_eq!(plan.fetch(), Some(2));
            }
            _ => panic!("expected slice plan"),
        }

        let unioned = union_exec(vec![table_plan.clone(), table_plan.clone()]);
        match &unioned {
            DynProofPlan::Union(plan) => assert_eq!(plan.input_plans().len(), 2),
            _ => panic!("expected union plan"),
        }

        let joined = sort_merge_join(
            table_plan.clone(),
            table_plan.clone(),
            vec![0],
            vec![0],
            vec!["id".into(), "left_value".into(), "right_value".into()],
        );
        match &joined {
            DynProofPlan::SortMergeJoin(plan) => {
                assert_eq!(plan.left_plan(), &table_plan);
                assert_eq!(plan.right_plan(), &table_plan);
                assert_eq!(plan.left_join_column_indexes(), &vec![0]);
                assert_eq!(plan.right_join_column_indexes(), &vec![0]);
                assert_eq!(plan.result_idents().len(), 3);
                assert_eq!(plan.result_idents()[0].to_string(), "id");
            }
            _ => panic!("expected sort merge join plan"),
        }

        assert_eq!(aliased_id.expr.data_type(), ColumnType::BigInt);
    }
}
