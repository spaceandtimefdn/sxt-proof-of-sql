use super::{ConversionError, ProvableExprPlanBuilder};
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType},
    },
    sql::ast::{ProvableExpr, ProvableExprPlan},
};
use indexmap::IndexMap;
use proof_of_sql_parser::{intermediate_ast::Expression, Identifier};

/// Builder that enables building a `proof_of_sql::sql::ast::ProvableExprPlan` from a `proof_of_sql_parser::intermediate_ast::Expression` that is
/// intended to be used as the where clause in a filter expression or group by expression.
pub struct WhereExprBuilder<'a> {
    builder: ProvableExprPlanBuilder<'a>,
}
impl<'a> WhereExprBuilder<'a> {
    /// Creates a new `WhereExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a IndexMap<Identifier, ColumnRef>) -> Self {
        Self {
            builder: ProvableExprPlanBuilder::new(column_mapping),
        }
    }
    /// Builds a `proof_of_sql::sql::ast::ProvableExprPlan` from a `proof_of_sql_parser::intermediate_ast::Expression` that is
    /// intended to be used as the where clause in a filter expression or group by expression.
    pub fn build<C: Commitment>(
        self,
        where_expr: Option<Box<Expression>>,
    ) -> Result<Option<ProvableExprPlan<C>>, ConversionError> {
        where_expr
            .map(|where_expr| {
                let expr_plan = self.builder.build(&where_expr)?;
                // Ensure that the expression is a boolean expression
                match expr_plan.data_type() {
                    ColumnType::Boolean => Ok(expr_plan),
                    _ => Err(ConversionError::NonbooleanWhereClause(
                        expr_plan.data_type(),
                    )),
                }
            })
            .transpose()
    }
}
