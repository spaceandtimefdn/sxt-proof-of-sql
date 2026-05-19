use crate::{logical_plan_to_proof_plan, PlannerResult, PoSqlContextProvider};
use alloc::{sync::Arc, vec::Vec};
use datafusion::{
    config::ConfigOptions,
    logical_expr::LogicalPlan,
    optimizer::{Analyzer, Optimizer, OptimizerContext, OptimizerRule},
    sql::planner::{ParserOptions, SqlToRel},
};
use indexmap::IndexSet;
use proof_of_sql::{
    base::database::{ParseError, SchemaAccessor, TableRef},
    sql::proof_plans::DynProofPlan,
};
use sqlparser::ast::{visit_relations, Statement};
use std::ops::ControlFlow;

/// Get [`Optimizer`]
///
/// In order to support queries such as `select $1::varchar;` we have to temporarily disable
/// [`CommonSubexprEliminate`] rule in the optimizer in `DataFusion` 38. Once we upgrade to
/// `DataFusion` 46 we can remove this function and use `Optimizer::new()` directly.
pub fn optimizer() -> Optimizer {
    // Step 1: Grab the recommended set
    let recommended_rules: Vec<Arc<dyn OptimizerRule + Send + Sync>> = Optimizer::new().rules;

    // Step 2: Filter out [`CommonSubexprEliminate`]
    let filtered_rules = recommended_rules
        .into_iter()
        .filter(|rule| rule.name() != "common_sub_expression_eliminate")
        .collect::<Vec<_>>();

    // Step 3: Build an optimizer with the new list
    Optimizer::with_rules(filtered_rules)
}

/// Convert a SQL query to a Proof of SQL plan using schema from provided tables
///
/// This function does the following
/// 1. Parse the SQL query into AST using sqlparser
/// 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
/// 3. Analyze the `LogicalPlan` using `Analyzer`
/// 4. Optimize the `LogicalPlan` using `Optimizer`
/// 5. Convert the optimized `LogicalPlan` into a Proof of SQL plan
fn sql_to_posql_plans<T, F, A>(
    statements: &[Statement],
    schemas: &A,
    config: &ConfigOptions,
    planner_converter: F,
) -> PlannerResult<Vec<T>>
where
    F: Fn(&LogicalPlan, &A) -> PlannerResult<T>,
    A: SchemaAccessor + Clone,
{
    let context_provider = PoSqlContextProvider::new(schemas.clone());
    // 1. Parse the SQL query into AST using sqlparser
    statements
        .iter()
        .map(|ast| -> PlannerResult<T> {
            // 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
            let raw_logical_plan = SqlToRel::new_with_options(
                &context_provider,
                ParserOptions {
                    parse_float_as_decimal: config.sql_parser.parse_float_as_decimal,
                    enable_ident_normalization: config.sql_parser.enable_ident_normalization,
                },
            )
            .sql_statement_to_plan(ast.clone())?;
            // 3. Analyze the `LogicalPlan` using `Analyzer`
            let analyzer = Analyzer::new();
            let analyzed_logical_plan =
                analyzer.execute_and_check(raw_logical_plan, config, |_, _| {})?;
            // 4. Optimize the `LogicalPlan` using `Optimizer`
            let optimizer = optimizer();
            let optimizer_context = OptimizerContext::default();
            let optimized_logical_plan =
                optimizer.optimize(analyzed_logical_plan, &optimizer_context, |_, _| {})?;
            // 5. Convert the optimized `LogicalPlan` into a Proof of SQL plan
            planner_converter(&optimized_logical_plan, schemas)
        })
        .collect::<PlannerResult<Vec<_>>>()
}

/// Convert a SQL query to a `DynProofPlan` using schema from provided tables
///
/// See `sql_to_posql_plans` for more details
pub fn sql_to_proof_plans<A: SchemaAccessor + Clone>(
    statements: &[Statement],
    schemas: &A,
    config: &ConfigOptions,
) -> PlannerResult<Vec<DynProofPlan>> {
    sql_to_posql_plans(statements, schemas, config, logical_plan_to_proof_plan)
}

/// Given a `Statement` retrieves all unique tables in the query
pub fn get_table_refs_from_statement(
    statement: &Statement,
) -> Result<IndexSet<TableRef>, ParseError> {
    let mut table_refs: IndexSet<TableRef> = IndexSet::<TableRef>::new();
    visit_relations(statement, |object_name| {
        match object_name.to_string().as_str().try_into() {
            Ok(table_ref) => {
                table_refs.insert(table_ref);
                ControlFlow::Continue(())
            }
            e => ControlFlow::Break(e),
        }
    })
    .break_value()
    .transpose()?;
    Ok(table_refs)
}

#[cfg(test)]
mod tests {
    use super::get_table_refs_from_statement;
    use crate::{conversion::sql_to_posql_plans, PlannerResult};
    use datafusion::{config::ConfigOptions, logical_expr::LogicalPlan};
    use indexmap::IndexSet;
    use proof_of_sql::{
        base::database::{
            ColumnField, ColumnRef, ColumnType, SchemaAccessor, TableRef, TableTestAccessor,
        },
        proof_primitive::dory::DynamicDoryEvaluationProof,
        sql::{
            proof_exprs::{AliasedDynProofExpr, DynProofExpr},
            proof_plans::DynProofPlan,
        },
    };
    use sqlparser::ast::Ident;
    use sqlparser::{dialect::GenericDialect, parser::Parser};

    #[derive(Clone)]
    struct NullableOrdersSchema;

    impl SchemaAccessor for NullableOrdersSchema {
        fn lookup_column(&self, _table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType> {
            match column_id.value.as_str() {
                "id" | "amount" => Some(ColumnType::BigInt),
                _ => None,
            }
        }

        fn lookup_schema(&self, _table_ref: &TableRef) -> Vec<(Ident, ColumnType)> {
            vec![
                ("id".into(), ColumnType::BigInt),
                ("amount".into(), ColumnType::BigInt),
            ]
        }

        fn lookup_column_fields(&self, _table_ref: &TableRef) -> Vec<ColumnField> {
            vec![
                ColumnField::new("id".into(), ColumnType::BigInt),
                ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
            ]
        }
    }

    #[test]
    fn we_can_get_table_references() {
        let statement = Parser::parse_sql(
            &GenericDialect {},
            "SELECT e.employee_id, e.employee_name, d.department_name, p.project_name, s.salary
FROM employees e
JOIN departments d ON e.department_id = d.department_id
JOIN management.projects p ON e.employee_id = p.employee_id
JOIN internal.salaries s ON e.employee_id = s.employee_id
WHERE e.department_id IN (
    SELECT department_id
    FROM departments
    WHERE department_name = 'Sales'
)
AND p.project_id IN (
    SELECT project_id
    FROM project_assignments
    WHERE employee_id = e.employee_id
)
AND s.salary > (
    SELECT AVG(salary)
    FROM internal.salaries
    WHERE department_id = e.department_id
);
",
        )
        .unwrap()[0]
            .clone();
        let table_refs = get_table_refs_from_statement(&statement).unwrap();
        let expected_table_refs: IndexSet<TableRef> = [
            ("", "departments"),
            ("", "employees"),
            ("management", "projects"),
            ("", "project_assignments"),
            ("internal", "salaries"),
        ]
        .map(|(s, t)| TableRef::new(s, t))
        .into_iter()
        .collect();
        assert_eq!(table_refs, expected_table_refs);
    }

    #[test]
    fn we_can_use_abs() {
        let statements = Parser::parse_sql(&GenericDialect {}, "SELECT ABS(-1-1);").unwrap();
        sql_to_posql_plans(
            &statements,
            &TableTestAccessor::<DynamicDoryEvaluationProof>::default(),
            &ConfigOptions::default(),
            |a, _| -> PlannerResult<LogicalPlan> { Ok(a.clone()) },
        )
        .unwrap();
    }

    #[test]
    fn sql_is_null_uses_generated_presence_column_in_plan() {
        let statements = Parser::parse_sql(
            &GenericDialect {},
            "SELECT id FROM orders WHERE amount IS NULL;",
        )
        .unwrap();
        let plans = super::sql_to_proof_plans(
            &statements,
            &NullableOrdersSchema,
            &ConfigOptions::default(),
        )
        .unwrap();
        let table_ref = TableRef::from_names(None, "orders");
        let amount_ref =
            ColumnRef::new_nullable(table_ref.clone(), "amount".into(), ColumnType::BigInt);

        assert_eq!(
            plans,
            vec![DynProofPlan::new_filter(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(ColumnRef::new(
                        table_ref.clone(),
                        "id".into(),
                        ColumnType::BigInt,
                    )),
                    alias: "id".into(),
                }],
                DynProofPlan::new_table(
                    table_ref,
                    vec![
                        ColumnField::new("id".into(), ColumnType::BigInt),
                        ColumnField::new(
                            ColumnRef::presence_column_id(&"amount".into()),
                            ColumnType::Boolean,
                        ),
                    ],
                ),
                DynProofExpr::new_is_null(amount_ref),
            )]
        );
    }
}
