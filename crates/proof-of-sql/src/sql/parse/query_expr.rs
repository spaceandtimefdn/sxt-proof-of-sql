use super::{EnrichedExpr, FilterExprBuilder, QueryContextBuilder, ResultExprBuilder};
use crate::{
    base::{commitment::Commitment, database::SchemaAccessor},
    sql::{
        ast::{GroupByExpr, ProofPlan},
        parse::ConversionResult,
        transform::ResultExpr,
    },
};
use proof_of_sql_parser::{intermediate_ast::SetExpression, Identifier, SelectStatement};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Serialize, Deserialize)]
/// A `QueryExpr` represents a Proof of SQL query that can be executed against a database.
/// It consists of a `ProofPlan` for provable components and a `ResultExpr` for the rest.
pub struct QueryExpr<C: Commitment> {
    proof_expr: ProofPlan<C>,
    result: ResultExpr,
}

// Implements fmt::Debug to aid in debugging QueryExpr.
// Prints filter and result fields in a readable format.
impl<C: Commitment> fmt::Debug for QueryExpr<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QueryExpr \n[{:#?},\n{:#?}\n]",
            self.proof_expr, self.result
        )
    }
}

impl<C: Commitment> QueryExpr<C> {
    /// Creates a new `QueryExpr` with the given `ProofPlan` and `ResultExpr`.
    pub fn new(proof_expr: ProofPlan<C>, result: ResultExpr) -> Self {
        Self { proof_expr, result }
    }

    /// Parse an intermediate AST `SelectStatement` into a `QueryExpr`.
    pub fn try_new(
        ast: SelectStatement,
        default_schema: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Self> {
        let context = match *ast.expr {
            SetExpression::Query {
                result_exprs,
                from,
                where_expr,
                group_by,
            } => QueryContextBuilder::new(schema_accessor)
                .visit_table_expr(from, default_schema)
                .visit_group_by_exprs(group_by)?
                .visit_result_exprs(result_exprs)?
                .visit_where_expr(where_expr)?
                .visit_order_by_exprs(ast.order_by)
                .visit_slice_expr(ast.slice)
                .build()?,
        };
        let result_aliased_exprs = context.get_aliased_result_exprs()?;
        let group_by = context.get_group_by_exprs();
        if !group_by.is_empty() {
            if let Some(group_by_expr) = Option::<GroupByExpr<C>>::try_from(&context)? {
                return Ok(Self {
                    proof_expr: ProofPlan::GroupBy(group_by_expr),
                    result: ResultExprBuilder::default()
                        .add_select_exprs(result_aliased_exprs)
                        .add_order_by_exprs(context.get_order_by_exprs()?)
                        .add_slice_expr(context.get_slice_expr())
                        .build(),
                });
            }
        }
        let column_mapping = context.get_column_mapping();
        let enriched_exprs = result_aliased_exprs
            .iter()
            .map(|aliased_expr| EnrichedExpr::new(aliased_expr.clone(), column_mapping.clone()))
            .collect::<Vec<_>>();
        let select_exprs = enriched_exprs
            .iter()
            .map(|enriched_expr| enriched_expr.residue_expression.clone())
            .collect::<Vec<_>>();
        let filter = FilterExprBuilder::new(context.get_column_mapping())
            .add_table_expr(*context.get_table_ref())
            .add_where_expr(context.get_where_expr().clone())?
            .add_result_columns(&enriched_exprs)
            .build();
        let result = ResultExprBuilder::default()
            .add_group_by_exprs(context.get_group_by_exprs(), &select_exprs)
            .add_select_exprs(&select_exprs)
            .add_order_by_exprs(context.get_order_by_exprs()?)
            .add_slice_expr(context.get_slice_expr())
            .build();

        Ok(Self {
            proof_expr: ProofPlan::DenseFilter(filter),
            result,
        })
    }

    /// Immutable access to this query's provable filter expression.
    pub fn proof_expr(&self) -> &ProofPlan<C> {
        &self.proof_expr
    }

    /// Immutable access to this query's post-proof result transform expression.
    pub fn result(&self) -> &ResultExpr {
        &self.result
    }
}
