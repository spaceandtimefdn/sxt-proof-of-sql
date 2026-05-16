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
        base::{database::SchemaAccessorImpl, map::indexmap},
        sql::proof_exprs::{test_utility as expr, ProofExpr},
    };

    fn table_and_schema() -> (TableRef, Vec<ColumnField>, SchemaAccessorImpl) {
        let table_ref = TableRef::from_names(Some("sxt"), "orders");
        let schema = vec![
            column_field("id", ColumnType::Int),
            column_field("flag", ColumnType::Boolean),
            column_field("amount", ColumnType::BigInt),
        ];
        let accessor = SchemaAccessorImpl::new(indexmap! {
            table_ref.clone() => schema
                .iter()
                .map(|field| (field.name(), field.data_type()))
                .collect(),
        });

        (table_ref, schema, accessor)
    }

    #[test]
    fn we_can_construct_basic_plan_helpers() {
        let (table_ref, schema, _) = table_and_schema();
        let field = column_field("amount", ColumnType::BigInt);

        assert_eq!(field.name().value, "amount");
        assert_eq!(field.data_type(), ColumnType::BigInt);
        assert!(matches!(empty_exec(), DynProofPlan::Empty(_)));

        match table_exec(table_ref.clone(), schema.clone()) {
            DynProofPlan::Table(plan) => {
                assert_eq!(plan.table_ref(), &table_ref);
                assert_eq!(plan.schema(), schema);
            }
            _ => panic!("expected table plan"),
        }
    }

    #[test]
    fn we_can_construct_projection_filter_and_slice_helpers() {
        let (table_ref, schema, accessor) = table_and_schema();
        let source = table_exec(table_ref.clone(), schema);
        let results = vec![expr::col_expr_plan(&table_ref, "amount", &accessor)];

        match projection(results.clone(), source.clone()) {
            DynProofPlan::Projection(plan) => {
                assert_eq!(plan.aliased_results(), results);
                assert_eq!(plan.input(), &source);
            }
            _ => panic!("expected projection plan"),
        }

        match filter(results.clone(), source.clone(), expr::const_bool(true)) {
            DynProofPlan::Filter(plan) => {
                assert_eq!(plan.aliased_results(), results);
                assert_eq!(plan.input(), &source);
                assert_eq!(plan.where_clause().data_type(), ColumnType::Boolean);
            }
            _ => panic!("expected filter plan"),
        }

        match slice_exec(source.clone(), 2, Some(5)) {
            DynProofPlan::Slice(plan) => {
                assert_eq!(plan.input(), &source);
                assert_eq!(plan.skip(), 2);
                assert_eq!(plan.fetch(), Some(5));
            }
            _ => panic!("expected slice plan"),
        }
    }

    #[test]
    fn we_can_construct_legacy_group_by_and_aggregate_helpers() {
        let (table_ref, schema, accessor) = table_and_schema();
        let table_expr = expr::tab(&table_ref);
        let amount_sum = vec![expr::sum_expr(
            expr::column(&table_ref, "amount", &accessor),
            "total",
        )];
        let id_expr = expr::col_expr(&table_ref, "id", &accessor);
        let where_clause = expr::const_bool(true);

        match legacy_filter(amount_sum.clone(), table_expr.clone(), where_clause.clone()) {
            DynProofPlan::LegacyFilter(plan) => {
                assert_eq!(plan.aliased_results(), amount_sum);
                assert_eq!(plan.table(), &table_expr);
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected legacy filter plan"),
        }

        match group_by(
            vec![id_expr.clone()],
            amount_sum.clone(),
            "row_count",
            table_expr,
            where_clause.clone(),
        ) {
            DynProofPlan::GroupBy(plan) => {
                assert_eq!(plan.group_by_exprs(), [id_expr.clone()]);
                assert_eq!(plan.sum_expr(), amount_sum);
                assert_eq!(plan.count_alias().value, "row_count");
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected group by plan"),
        }

        let source = table_exec(table_ref.clone(), schema);
        let group_alias = vec![expr::aliased_plan(
            expr::column(&table_ref, "id", &accessor),
            "id",
        )];
        match aggregate(
            group_alias.clone(),
            amount_sum.clone(),
            "row_count",
            source.clone(),
            where_clause.clone(),
        ) {
            DynProofPlan::Aggregate(plan) => {
                assert_eq!(plan.group_by_exprs(), group_alias);
                assert_eq!(plan.sum_expr(), amount_sum);
                assert_eq!(plan.count_alias().value, "row_count");
                assert_eq!(plan.input(), &source);
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected aggregate plan"),
        }
    }

    #[test]
    fn we_can_construct_union_and_join_helpers() {
        let left_ref = TableRef::from_names(Some("sxt"), "left_orders");
        let right_ref = TableRef::from_names(Some("sxt"), "right_orders");
        let left = table_exec(left_ref, vec![column_field("id", ColumnType::Int)]);
        let right = table_exec(right_ref, vec![column_field("id", ColumnType::Int)]);

        match union_exec(vec![left.clone(), right.clone()]) {
            DynProofPlan::Union(plan) => {
                assert_eq!(plan.input_plans(), [left.clone(), right.clone()]);
            }
            _ => panic!("expected union plan"),
        }

        match sort_merge_join(
            left.clone(),
            right.clone(),
            vec![0],
            vec![0],
            vec!["id".into()],
        ) {
            DynProofPlan::SortMergeJoin(plan) => {
                assert_eq!(plan.left_plan(), &left);
                assert_eq!(plan.right_plan(), &right);
                assert_eq!(plan.left_join_column_indexes(), &vec![0]);
                assert_eq!(plan.right_join_column_indexes(), &vec![0]);
                assert_eq!(plan.result_idents()[0].value, "id");
            }
            _ => panic!("expected sort merge join plan"),
        }
    }
}
