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
    use crate::{
        base::database::{ColumnField, ColumnType, TableRef},
        sql::proof::ProofPlan,
    };
    use sqlparser::ast::Ident;

    fn bigint_field(name: &str) -> ColumnField {
        ColumnField::new(Ident::new(name), ColumnType::BigInt)
    }

    fn table_ref() -> TableRef {
        "namespace.table".parse().unwrap()
    }

    // ── Constructors ─────────────────────────────────────────────────────────

    #[test]
    fn new_empty_returns_empty_variant() {
        let plan = DynProofPlan::new_empty();
        assert!(matches!(plan, DynProofPlan::Empty(_)));
    }

    #[test]
    fn new_table_returns_table_variant() {
        let schema = vec![bigint_field("a"), bigint_field("b")];
        let plan = DynProofPlan::new_table(table_ref(), schema);
        assert!(matches!(plan, DynProofPlan::Table(_)));
    }

    #[test]
    fn new_slice_returns_slice_variant() {
        let input = DynProofPlan::new_empty();
        let plan = DynProofPlan::new_slice(input, 10, Some(5));
        assert!(matches!(plan, DynProofPlan::Slice(_)));
    }

    #[test]
    fn new_slice_no_fetch_returns_slice_variant() {
        let input = DynProofPlan::new_empty();
        let plan = DynProofPlan::new_slice(input, 0, None);
        assert!(matches!(plan, DynProofPlan::Slice(_)));
    }

    #[test]
    fn new_projection_returns_projection_variant() {
        let input = DynProofPlan::new_empty();
        let plan = DynProofPlan::new_projection(vec![], input);
        assert!(matches!(plan, DynProofPlan::Projection(_)));
    }

    #[test]
    fn table_plan_schema_roundtrip() {
        let schema = vec![bigint_field("x"), bigint_field("y")];
        let plan = DynProofPlan::new_table(table_ref(), schema);
        // column_fields() from ProofPlan trait should return the schema we set
        let fields = plan.get_column_result_fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), &Ident::new("x"));
        assert_eq!(fields[1].name(), &Ident::new("y"));
    }

    // ── get_table_references / get_column_references ──────────────────────────

    #[test]
    fn empty_plan_has_no_table_references() {
        let plan = DynProofPlan::new_empty();
        assert!(plan.get_table_references().is_empty());
    }

    #[test]
    fn empty_plan_has_no_column_references() {
        let plan = DynProofPlan::new_empty();
        assert!(plan.get_column_references().is_empty());
    }

    #[test]
    fn table_plan_returns_its_table_reference() {
        let schema = vec![bigint_field("x")];
        let tref = table_ref();
        let plan = DynProofPlan::new_table(tref.clone(), schema);
        let refs = plan.get_table_references();
        assert!(refs.contains(&tref));
    }

    // ── PartialEq / Clone ────────────────────────────────────────────────────

    #[test]
    fn two_empty_plans_are_equal() {
        assert_eq!(DynProofPlan::new_empty(), DynProofPlan::new_empty());
    }

    #[test]
    fn empty_and_table_plans_are_not_equal() {
        let table = DynProofPlan::new_table(table_ref(), vec![]);
        let empty = DynProofPlan::new_empty();
        assert_ne!(table, empty);
    }

    #[test]
    fn clone_of_empty_plan_equals_original() {
        let plan = DynProofPlan::new_empty();
        assert_eq!(plan.clone(), plan);
    }
}
