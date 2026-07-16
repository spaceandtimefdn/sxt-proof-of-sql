use arrow::datatypes::DataType;
use core::fmt::{self, Display, Formatter};
use datafusion::{
    common::{DataFusionError, JoinConstraint, JoinType},
    logical_expr::{
        expr::{AggregateFunction, Placeholder},
        Expr, LogicalPlan, Operator,
    },
    physical_plan,
};
use proof_of_sql::{
    base::math::decimal::DecimalError,
    sql::{proof_plans::AggregateExecError, AnalyzeError},
};
use snafu::Snafu;
use sqlparser::parser::ParserError;

/// Errors encountered while converting an aggregate logical plan.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Snafu)]
pub enum AggregatePlanError {
    /// A grouping expression was absent from `DataFusion`'s output alias map.
    #[snafu(display("group expression {expression} has no output alias"))]
    MissingGroupExpressionAlias {
        /// Display name used to look up the expression.
        expression: String,
    },
    /// An aggregate expression was absent from `DataFusion`'s output alias map.
    #[snafu(display("aggregate expression {expression} has no output alias"))]
    MissingAggregateExpressionAlias {
        /// Display name used to look up the expression.
        expression: String,
    },
    /// `DataFusion` placed a non-aggregate expression in the aggregate expression list.
    #[snafu(display("aggregate expression list contains non-aggregate expression {expression}"))]
    UnexpectedAggregateExpression {
        /// Unexpected expression.
        expression: String,
    },
    /// The aggregate proof-plan constructor rejected the requested shape.
    #[snafu(transparent)]
    AggregateExec {
        /// Authoritative aggregate construction error.
        source: AggregateExecError,
    },
}

/// Errors encountered while converting a join logical plan.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Snafu)]
pub enum JoinPlanError {
    /// Only inner joins can be converted to the current proof plan.
    #[snafu(display("join type {join_type} is not supported"))]
    UnsupportedJoinType {
        /// Unsupported join type.
        join_type: JoinType,
    },
    /// Only `ON` join constraints can be converted to the current proof plan.
    #[snafu(display("join constraint {constraint:?} is not supported"))]
    UnsupportedJoinConstraint {
        /// Unsupported join constraint.
        constraint: JoinConstraint,
    },
    /// The join predicate is not a supported pair of matching column references.
    #[snafu(display(
        "join predicate must pair supported columns with the same unqualified name, found {left} and {right}"
    ))]
    UnsupportedPredicate {
        /// Left-hand join expression.
        left: String,
        /// Right-hand join expression.
        right: String,
    },
}

/// Kind of `DataFusion` logical plan node presented to the Proof of SQL converter.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogicalPlanNodeKind {
    /// Projection node.
    Projection,
    /// Filter node.
    Filter,
    /// Window node.
    Window,
    /// Aggregate node.
    Aggregate,
    /// Sort node.
    Sort,
    /// Join node.
    Join,
    /// Cross-join node.
    CrossJoin,
    /// Repartition node.
    Repartition,
    /// Union node.
    Union,
    /// Table-scan node.
    TableScan,
    /// Empty-relation node.
    EmptyRelation,
    /// Subquery node.
    Subquery,
    /// Subquery-alias node.
    SubqueryAlias,
    /// Limit node.
    Limit,
    /// Statement node.
    Statement,
    /// Values node.
    Values,
    /// Explain node.
    Explain,
    /// Analyze node.
    Analyze,
    /// Extension node.
    Extension,
    /// Distinct node.
    Distinct,
    /// Prepare node.
    Prepare,
    /// Data-manipulation node.
    Dml,
    /// Data-definition node.
    Ddl,
    /// Copy node.
    Copy,
    /// Describe-table node.
    DescribeTable,
    /// Unnest node.
    Unnest,
    /// Recursive-query node.
    RecursiveQuery,
}

impl Display for LogicalPlanNodeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Projection => "projection",
            Self::Filter => "filter",
            Self::Window => "window",
            Self::Aggregate => "aggregate",
            Self::Sort => "sort",
            Self::Join => "join",
            Self::CrossJoin => "cross join",
            Self::Repartition => "repartition",
            Self::Union => "union",
            Self::TableScan => "table scan",
            Self::EmptyRelation => "empty relation",
            Self::Subquery => "subquery",
            Self::SubqueryAlias => "subquery alias",
            Self::Limit => "limit",
            Self::Statement => "statement",
            Self::Values => "values",
            Self::Explain => "explain",
            Self::Analyze => "analyze",
            Self::Extension => "extension",
            Self::Distinct => "distinct",
            Self::Prepare => "prepare",
            Self::Dml => "data manipulation",
            Self::Ddl => "data definition",
            Self::Copy => "copy",
            Self::DescribeTable => "describe table",
            Self::Unnest => "unnest",
            Self::RecursiveQuery => "recursive query",
        };
        f.write_str(label)
    }
}

/// Proof of SQL Planner error
#[non_exhaustive]
#[derive(Debug, Snafu)]
pub enum PlannerError {
    /// Returned when the internal analyze process fails
    #[snafu(transparent)]
    AnalyzeError {
        /// Underlying analyze error
        source: AnalyzeError,
    },
    /// Returned when a decimal error occurs
    #[snafu(transparent)]
    DecimalError {
        /// Underlying decimal error
        source: DecimalError,
    },
    /// Returned when sqlparser fails to parse a query
    #[snafu(transparent)]
    SqlParserError {
        /// Underlying sqlparser error
        source: ParserError,
    },
    /// Returned when datafusion fails to plan a query
    #[snafu(transparent)]
    DataFusionError {
        /// Underlying datafusion error
        source: DataFusionError,
    },
    /// Returned if a column is not found
    #[snafu(display("Column not found"))]
    ColumnNotFound,
    /// Returned if a table is not found
    #[snafu(display("Table not found: {}", table_name))]
    TableNotFound {
        /// Table name
        table_name: String,
    },
    /// Returned when a placeholder id is invalid
    #[snafu(display("Placeholder id {id:?} is invalid"))]
    InvalidPlaceholderId {
        /// Unsupported placeholder id
        id: String,
    },
    /// Returned when a placeholder is untyped
    #[snafu(display("Placeholder {placeholder:?} is untyped"))]
    UntypedPlaceholder {
        /// Untyped placeholder
        placeholder: Placeholder,
    },
    /// Returned when a datatype is not supported
    #[snafu(display("Unsupported datatype: {}", data_type))]
    UnsupportedDataType {
        /// Unsupported datatype
        data_type: DataType,
    },
    /// Returned when a binary operator is not supported
    #[snafu(display("Binary operator {} is not supported", op))]
    UnsupportedBinaryOperator {
        /// Unsupported binary operation
        op: Operator,
    },
    /// Returned when the aggregate opetation is not supported
    #[snafu(display("Aggregate operation {op:?} is not supported"))]
    UnsupportedAggregateOperation {
        /// Unsupported aggregate operation
        op: physical_plan::aggregates::AggregateFunction,
    },
    /// Returned when the `AggregateFunction` is not supported
    #[snafu(display("AggregateFunction {function:?} is not supported"))]
    UnsupportedAggregateFunction {
        /// Unsupported `AggregateFunction`
        function: AggregateFunction,
    },
    /// Returned when a logical expression is not resolved
    #[snafu(display("Logical expression {:?} is not supported", expr))]
    UnsupportedLogicalExpression {
        /// Unsupported logical expression
        expr: Box<Expr>,
    },
    /// Returned when an aggregate logical plan cannot be converted.
    #[snafu(display("Aggregate logical plan is not supported: {source}"))]
    UnsupportedAggregatePlan {
        /// Specific aggregate conversion error.
        source: AggregatePlanError,
        /// Complete offending logical plan.
        plan: Box<LogicalPlan>,
    },
    /// Returned when a join logical plan cannot be converted.
    #[snafu(display("Join logical plan is not supported: {source}"))]
    UnsupportedJoinPlan {
        /// Specific join conversion error.
        source: JoinPlanError,
        /// Complete offending logical plan.
        plan: Box<LogicalPlan>,
    },
    /// Returned when a `LogicalPlan` node or shape is not supported.
    #[snafu(display("Logical plan node or shape {node} is not supported"))]
    UnsupportedLogicalPlan {
        /// Kind of node whose particular shape was unsupported.
        node: LogicalPlanNodeKind,
        /// Unsupported `LogicalPlan`
        plan: Box<LogicalPlan>,
    },
    /// Returned when the `LogicalPlan` is not resolved
    #[snafu(display("LogicalPlan is not resolved"))]
    UnresolvedLogicalPlan,
    /// Returned when catalog is provided since it is not supported
    #[snafu(display("Catalog is not supported"))]
    CatalogNotSupported,
}

/// Proof of SQL Planner result
pub type PlannerResult<T> = Result<T, PlannerError>;

#[cfg(test)]
mod tests {
    use super::*;
    use proof_of_sql::base::database::ColumnType;

    #[test]
    fn planner_sub_errors_include_actionable_context() {
        assert_eq!(
            AggregatePlanError::MissingGroupExpressionAlias {
                expression: "table.id".to_string(),
            }
            .to_string(),
            "group expression table.id has no output alias"
        );
        assert_eq!(
            AggregatePlanError::AggregateExec {
                source: AggregateExecError::UnsupportedGroupByExpressionType {
                    data_type: ColumnType::VarChar,
                },
            }
            .to_string(),
            "grouping and deduplication do not support expressions of type VARCHAR"
        );
        assert_eq!(
            JoinPlanError::UnsupportedPredicate {
                left: "left.a".to_string(),
                right: "right.b".to_string(),
            }
            .to_string(),
            "join predicate must pair supported columns with the same unqualified name, found left.a and right.b"
        );
    }

    #[test]
    fn logical_plan_node_kinds_have_human_readable_labels() {
        assert_eq!(LogicalPlanNodeKind::CrossJoin.to_string(), "cross join");
        assert_eq!(LogicalPlanNodeKind::Distinct.to_string(), "distinct");
        assert_eq!(LogicalPlanNodeKind::Dml.to_string(), "data manipulation");
    }
}
