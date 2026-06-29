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
    use super::DynProofPlan;
    use crate::{
        base::{
            database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
            map::indexset,
        },
        sql::{
            proof_exprs::{AliasedDynProofExpr, DynProofExpr, TableExpr},
            AnalyzeError,
        },
    };
    use sqlparser::ast::Ident;

    fn table_ref() -> TableRef {
        TableRef::new("sxt", "t")
    }

    fn column_field(name: &str, column_type: ColumnType) -> ColumnField {
        ColumnField::new(Ident::from(name), column_type)
    }

    fn column_ref(name: &str, column_type: ColumnType) -> ColumnRef {
        ColumnRef::new(table_ref(), Ident::from(name), column_type)
    }

    fn aliased_column(name: &str, column_type: ColumnType) -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_column(column_ref(name, column_type)),
            alias: Ident::from(name),
        }
    }

    fn table_expr() -> TableExpr {
        TableExpr {
            table_ref: table_ref(),
        }
    }

    fn true_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::Boolean(true))
    }

    #[test]
    fn constructors_wrap_expected_plan_variants() {
        let schema = vec![
            column_field("a", ColumnType::BigInt),
            column_field("b", ColumnType::Boolean),
        ];
        let table = DynProofPlan::new_table(table_ref(), schema.clone());
        match &table {
            DynProofPlan::Table(plan) => {
                assert_eq!(plan.table_ref(), &table_ref());
                assert_eq!(plan.schema(), schema);
            }
            _ => panic!("expected table plan"),
        }

        assert!(matches!(DynProofPlan::new_empty(), DynProofPlan::Empty(_)));

        let aliased = aliased_column("a", ColumnType::BigInt);
        let projection = DynProofPlan::new_projection(vec![aliased.clone()], table.clone());
        match &projection {
            DynProofPlan::Projection(plan) => {
                assert_eq!(plan.aliased_results(), &[aliased.clone()]);
                assert!(matches!(plan.input(), DynProofPlan::Table(_)));
            }
            _ => panic!("expected projection plan"),
        }

        let where_clause = true_expr();
        let legacy_filter = DynProofPlan::new_legacy_filter(
            vec![aliased.clone()],
            table_expr(),
            where_clause.clone(),
        );
        match &legacy_filter {
            DynProofPlan::LegacyFilter(plan) => {
                assert_eq!(plan.aliased_results(), &[aliased.clone()]);
                assert_eq!(plan.table(), &table_expr());
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected legacy filter plan"),
        }

        let filter =
            DynProofPlan::new_filter(vec![aliased.clone()], table.clone(), where_clause.clone());
        match &filter {
            DynProofPlan::Filter(plan) => {
                assert_eq!(plan.aliased_results(), &[aliased.clone()]);
                assert!(matches!(plan.input(), DynProofPlan::Table(_)));
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected filter plan"),
        }

        let slice = DynProofPlan::new_slice(table.clone(), 2, Some(3));
        match &slice {
            DynProofPlan::Slice(plan) => {
                assert!(matches!(plan.input(), DynProofPlan::Table(_)));
                assert_eq!(plan.skip(), 2);
                assert_eq!(plan.fetch(), Some(3));
            }
            _ => panic!("expected slice plan"),
        }
    }

    #[test]
    fn try_constructors_return_expected_success_and_error_variants() {
        let table =
            DynProofPlan::new_table(table_ref(), vec![column_field("a", ColumnType::BigInt)]);
        let where_clause = true_expr();

        let group_by = DynProofPlan::try_new_group_by(
            Vec::new(),
            Vec::new(),
            Ident::from("count"),
            table_expr(),
            where_clause.clone(),
        )
        .unwrap();
        match &group_by {
            DynProofPlan::GroupBy(plan) => {
                assert_eq!(plan.group_by_exprs(), &[]);
                assert_eq!(plan.sum_expr(), &[]);
                assert_eq!(plan.count_alias(), &Ident::from("count"));
                assert_eq!(plan.table(), &table_expr());
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected group-by plan"),
        }

        let aggregate = DynProofPlan::try_new_aggregate(
            Vec::new(),
            Vec::new(),
            Ident::from("count"),
            table.clone(),
            where_clause.clone(),
        )
        .unwrap();
        match &aggregate {
            DynProofPlan::Aggregate(plan) => {
                assert_eq!(plan.group_by_exprs(), &[]);
                assert_eq!(plan.sum_expr(), &[]);
                assert_eq!(plan.count_alias(), &Ident::from("count"));
                assert!(matches!(plan.input(), DynProofPlan::Table(_)));
                assert_eq!(plan.where_clause(), &where_clause);
            }
            _ => panic!("expected aggregate plan"),
        }

        let union =
            DynProofPlan::try_new_union(vec![table.clone(), DynProofPlan::new_empty()]).unwrap();
        match &union {
            DynProofPlan::Union(plan) => assert_eq!(plan.input_plans().len(), 2),
            _ => panic!("expected union plan"),
        }

        assert!(matches!(
            DynProofPlan::try_new_union(vec![table]),
            Err(AnalyzeError::NotEnoughInputPlans)
        ));
    }

    #[test]
    fn result_fields_can_be_returned_as_column_references() {
        let plan = DynProofPlan::new_table(
            table_ref(),
            vec![
                column_field("a", ColumnType::BigInt),
                column_field("b", ColumnType::Boolean),
            ],
        );

        assert_eq!(
            plan.get_column_result_fields_as_references(),
            indexset! {
                ColumnRef::new(TableRef::from_names(None, ""), Ident::from("a"), ColumnType::BigInt),
                ColumnRef::new(TableRef::from_names(None, ""), Ident::from("b"), ColumnType::Boolean),
            }
        );
    }
}
