use arrow::datatypes::DataType;
use datafusion::{
    common::DataFusionError,
    logical_expr::{
        expr::{AggregateFunction, Placeholder},
        Expr, LogicalPlan, Operator,
    },
    physical_plan,
};
use proof_of_sql::{base::math::decimal::DecimalError, sql::AnalyzeError};
use snafu::Snafu;
use sqlparser::parser::ParserError;

/// Proof of SQL Planner error
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
    /// Returned when a `LogicalPlan` is not supported
    #[snafu(display("LogicalPlan is not supported"))]
    UnsupportedLogicalPlan {
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
    use super::PlannerError;
    use arrow::datatypes::DataType;
    use datafusion::logical_expr::Operator;

    #[test]
    fn column_not_found_displays_correct_message() {
        let err = PlannerError::ColumnNotFound;
        assert_eq!(err.to_string(), "Column not found");
    }

    #[test]
    fn table_not_found_displays_table_name() {
        let err = PlannerError::TableNotFound {
            table_name: "myschema.mytable".to_string(),
        };
        assert_eq!(err.to_string(), "Table not found: myschema.mytable");
    }

    #[test]
    fn table_not_found_with_empty_table_name() {
        let err = PlannerError::TableNotFound {
            table_name: String::new(),
        };
        assert_eq!(err.to_string(), "Table not found: ");
    }

    #[test]
    fn invalid_placeholder_id_displays_id() {
        let err = PlannerError::InvalidPlaceholderId {
            id: "$0".to_string(),
        };
        assert!(err.to_string().contains("$0"));
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn unsupported_data_type_displays_type() {
        let err = PlannerError::UnsupportedDataType {
            data_type: DataType::LargeUtf8,
        };
        assert!(err.to_string().contains("Unsupported datatype"));
        assert!(err.to_string().contains("LargeUtf8"));
    }

    #[test]
    fn unsupported_binary_operator_displays_operator() {
        let err = PlannerError::UnsupportedBinaryOperator {
            op: Operator::RegexMatch,
        };
        assert!(err.to_string().contains("not supported"));
    }

    #[test]
    fn unresolved_logical_plan_displays_correct_message() {
        let err = PlannerError::UnresolvedLogicalPlan;
        assert_eq!(err.to_string(), "LogicalPlan is not resolved");
    }

    #[test]
    fn catalog_not_supported_displays_correct_message() {
        let err = PlannerError::CatalogNotSupported;
        assert_eq!(err.to_string(), "Catalog is not supported");
    }

    #[test]
    fn planner_errors_implement_debug() {
        let err = PlannerError::ColumnNotFound;
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("ColumnNotFound"));

        let err2 = PlannerError::UnresolvedLogicalPlan;
        let debug_str2 = format!("{err2:?}");
        assert!(debug_str2.contains("UnresolvedLogicalPlan"));
    }
}
