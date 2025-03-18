use super::{where_expr_builder::WhereExprBuilder, ConversionError, EnrichedExpr};
use crate::{
    base::{
        database::{ColumnRef, ColumnType, LiteralValue, TableRef},
        map::IndexMap,
    },
    sql::{
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, IsTrueExpr, ProofExpr, TableExpr},
        proof_plans::FilterExec,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use itertools::Itertools;
use proof_of_sql_parser::intermediate_ast::Expression;
use sqlparser::ast::Ident;
pub struct FilterExecBuilder {
    table_expr: Option<TableExpr>,
    where_expr: Option<DynProofExpr>,
    filter_result_expr_list: Vec<AliasedDynProofExpr>,
    column_mapping: IndexMap<Ident, ColumnRef>,
}

// Public interface
impl FilterExecBuilder {
    pub fn new(column_mapping: IndexMap<Ident, ColumnRef>) -> Self {
        Self {
            table_expr: None,
            where_expr: None,
            filter_result_expr_list: vec![],
            column_mapping,
        }
    }

    pub fn add_table_expr(mut self, table_ref: TableRef) -> Self {
        self.table_expr = Some(TableExpr { table_ref });
        self
    }

    pub fn add_where_expr(
        mut self,
        where_expr: Option<Box<Expression>>,
    ) -> Result<Self, ConversionError> {
        self.where_expr = WhereExprBuilder::new(&self.column_mapping).build(where_expr)?;
        Ok(self)
    }

    /// # Panics
    ///
    /// Will panic if:
    /// - `self.column_mapping.get(alias)` returns `None`, which can occur if the alias is not found in the column mapping.
    pub fn add_result_columns(mut self, columns: &[EnrichedExpr]) -> Self {
        // If a column is provable, add it to the filter result expression list
        // If at least one column is non-provable, add all columns from the column mapping to the filter result expression list
        let mut has_nonprovable_column = false;
        for enriched_expr in columns {
            if let Some(plan) = &enriched_expr.dyn_proof_expr {
                self.filter_result_expr_list.push(AliasedDynProofExpr {
                    expr: plan.clone(),
                    alias: enriched_expr.residue_expression.alias.into(),
                });
            } else {
                has_nonprovable_column = true;
            }
        }

        if has_nonprovable_column {
            // Has to keep them sorted to have deterministic order for tests
            for alias in self.column_mapping.keys().sorted() {
                let column_ref = self.column_mapping.get(alias).unwrap();
                self.filter_result_expr_list.push(AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(column_ref.clone()),
                    alias: alias.clone(),
                });
            }
        }
        self
    }

    #[expect(clippy::missing_panics_doc)]
    pub fn build(self) -> FilterExec {
        // Wrap the WHERE clause in an IsTrueExpr to correctly handle NULL values
        // In SQL's three-valued logic, a row is only included if the WHERE condition 
        // evaluates to TRUE (not NULL and not FALSE)
        let where_clause = self.where_expr.unwrap_or_else(|| 
            DynProofExpr::new_literal(LiteralValue::Boolean(true))
        );

        // Ensure the WHERE clause is wrapped in IsTrueExpr for proper NULL handling
        let where_clause = if where_clause.data_type() == ColumnType::Boolean {
            // Only wrap if it's a boolean expression and not already an IS TRUE expression
            match &where_clause {
                DynProofExpr::IsTrue(_) => where_clause, // Already wrapped
                _ => DynProofExpr::IsTrue(IsTrueExpr::new(Box::new(where_clause)))
            }
        } else {
            // Non-boolean expressions should have been caught earlier
            where_clause
        };

        FilterExec::new(
            self.filter_result_expr_list,
            self.table_expr.expect("Table expr is required"),
            where_clause,
        )
    }
}
