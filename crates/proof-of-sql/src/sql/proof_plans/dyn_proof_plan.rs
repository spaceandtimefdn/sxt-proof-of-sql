use super::{
    AggregateExec, EmptyExec, FilterExec, GroupByExec, LegacyFilterExec, ProjectionExec, SliceExec,
    SortMergeJoinExec, TableExec, UnionExec,
};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, LiteralValue, Table, TableEvaluation, TableRef},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
        AnalyzeResult,
    },
};
use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// The query plan for proving a query
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[enum_dispatch::enum_dispatch]
pub enum DynProofPlan {
    /// Source [`ProofPlan`] for (sub)queries without table source such as `SELECT "No table here" as msg;`
    Empty(EmptyExec),
    /// Source [`ProofPlan`] for (sub)queries with table source such as `SELECT col from tab;`
    Table(TableExec),
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
    /// ```
    Projection(ProjectionExec),
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
    ///         SUM(<sum_expr1>.0) as <sum_expr1>.1, ..., SUM(<sum_exprN>.0) as <sum_exprN>.1,
    ///         COUNT(*) as count_alias
    ///     FROM <table>
    ///     WHERE <where_clause>
    ///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
    /// ```
    GroupBy(GroupByExec),
    /// Provable expressions for queries of the form
    /// ```ignore
    ///     SELECT <group_by_expr1>.expr as <group_by_expr1>.alias, ..., <group_by_exprM>.expr as <group_by_exprM>.alias,
    ///         SUM(<sum_expr1>.expr) as <sum_expr1>.alias, ..., SUM(<sum_exprN>.expr) as <sum_exprN>.alias,
    ///         COUNT(*) as <count_alias>
    ///     FROM <input>
    ///     WHERE <where_clause>
    ///     GROUP BY <group_by_expr1>.expr, ..., <group_by_exprM>.expr
    /// ```
    /// Similar to `GroupBy` but accepts a [`DynProofPlan`] as input
    Aggregate(AggregateExec),
    /// Provable expressions for queries of the form, where the result is sent in a dense form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
    /// ```
    LegacyFilter(LegacyFilterExec),
    /// Provable expressions for queries of the form, where the result is sent in a dense form
    /// ```ignore
    ///     SELECT <result_expr1>, ..., <result_exprN> FROM <input> WHERE <where_clause>
    /// ```
    /// Accepts a [`DynProofPlan`] as input
    Filter(FilterExec),
    /// `ProofPlan` for queries of the form
    /// ```ignore
    ///     <ProofPlan> LIMIT <fetch> [OFFSET <skip>]
    /// ```
    Slice(SliceExec),
    /// `ProofPlan` for queries of the form
    /// ```ignore
    ///     <ProofPlan>
    ///     UNION ALL
    ///     <ProofPlan>
    ///     ...
    ///     UNION ALL
    ///     <ProofPlan>
    /// ```
    Union(UnionExec),
    /// `ProofPlan` for queries of the form
    /// ```ignore
    ///     <ProofPlan> INNER JOIN <ProofPlan>
    ///     ON col1 = col2
    /// ```
    SortMergeJoin(SortMergeJoinExec),
}

impl DynProofPlan {
    /// Creates a new empty plan.
    #[must_use]
    pub fn new_empty() -> Self {
        Self::Empty(EmptyExec::new())
    }

    /// Creates a new table plan.
    #[must_use]
    pub fn new_table(table_ref: TableRef, schema: Vec<ColumnField>) -> Self {
        Self::Table(TableExec::new(table_ref, schema))
    }

    /// Creates a new projection plan.
    #[must_use]
    pub fn new_projection(aliased_results: Vec<AliasedDynProofExpr>, input: DynProofPlan) -> Self {
        Self::Projection(ProjectionExec::new(aliased_results, Box::new(input)))
    }

    /// Creates a new legacy filter plan.
    #[must_use]
    pub fn new_legacy_filter(
        aliased_results: Vec<AliasedDynProofExpr>,
        input: TableExpr,
        filter_expr: DynProofExpr,
    ) -> Self {
        Self::LegacyFilter(LegacyFilterExec::new(aliased_results, input, filter_expr))
    }

    /// Creates a new group by plan.
    #[must_use]
    pub fn try_new_group_by(
        group_by_exprs: Vec<ColumnExpr>,
        sum_expr: Vec<AliasedDynProofExpr>,
        count_alias: Ident,
        table: TableExpr,
        where_clause: DynProofExpr,
    ) -> Option<Self> {
        GroupByExec::try_new(group_by_exprs, sum_expr, count_alias, table, where_clause)
            .map(Self::GroupBy)
    }

    /// Creates a new aggregate plan.
    #[must_use]
    pub fn try_new_aggregate(
        group_by_exprs: Vec<AliasedDynProofExpr>,
        sum_expr: Vec<AliasedDynProofExpr>,
        count_alias: Ident,
        input: DynProofPlan,
        where_clause: DynProofExpr,
    ) -> Option<Self> {
        AggregateExec::try_new(
            group_by_exprs,
            sum_expr,
            count_alias,
            Box::new(input),
            where_clause,
        )
        .map(Self::Aggregate)
    }

    /// Creates a new slice plan.
    #[must_use]
    pub fn new_slice(input: DynProofPlan, skip: usize, fetch: Option<usize>) -> Self {
        Self::Slice(SliceExec::new(Box::new(input), skip, fetch))
    }

    /// Creates a new union plan.
    pub fn try_new_union(inputs: Vec<DynProofPlan>) -> AnalyzeResult<Self> {
        UnionExec::try_new(inputs).map(Self::Union)
    }

    /// Creates a new filter plan.
    #[must_use]
    pub fn new_filter(
        aliased_results: Vec<AliasedDynProofExpr>,
        input: DynProofPlan,
        filter_expr: DynProofExpr,
    ) -> Self {
        Self::Filter(FilterExec::new(
            aliased_results,
            Box::new(input),
            filter_expr,
        ))
    }

    /// Returns the resulting column fields of the plan as column references
    pub(crate) fn get_column_result_fields_as_references(&self) -> IndexSet<ColumnRef> {
        self.get_column_result_fields()
            .into_iter()
            .map(|f| ColumnRef::new(TableRef::from_names(None, ""), f.name(), f.data_type()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::database::{ColumnType, LiteralValue};

    fn bigint_field(name: &str) -> ColumnField {
        ColumnField::new(name.into(), ColumnType::BigInt)
    }

    fn single_col_table(table: &str, col: &str) -> DynProofPlan {
        DynProofPlan::new_table(table.parse().unwrap(), vec![bigint_field(col)])
    }

    #[test]
    fn we_can_get_column_result_fields_as_references() {
        let plan = single_col_table("sxt.t", "a");
        let refs = plan.get_column_result_fields_as_references();
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn we_can_create_union_plan_from_two_compatible_tables() {
        let plan1 = single_col_table("sxt.t1", "a");
        let plan2 = single_col_table("sxt.t2", "a");
        assert!(DynProofPlan::try_new_union(vec![plan1, plan2]).is_ok());
    }

    #[test]
    fn try_new_group_by_body_is_executed_regardless_of_result() {
        // GroupByExec::try_new is called and map is applied; result may be Some or None.
        let table: TableRef = "sxt.t".parse().unwrap();
        let result = DynProofPlan::try_new_group_by(
            vec![],
            vec![],
            "count".into(),
            TableExpr { table_ref: table },
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        );
        let _ = result;
    }
}
