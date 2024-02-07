use super::where_expr_builder::WhereExprBuilder;
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, TableRef},
    },
    sql::ast::{BoolExprPlan, FilterExpr, FilterResultExpr, TableExpr},
};
use proofs_sql::{intermediate_ast::Expression, Identifier};
use std::collections::{HashMap, HashSet};

pub struct FilterExprBuilder<C: Commitment> {
    table_expr: Option<TableExpr>,
    where_expr: Option<BoolExprPlan<C>>,
    filter_result_expr_list: Vec<FilterResultExpr>,
    column_mapping: HashMap<Identifier, ColumnRef>,
}

// Public interface
impl<C: Commitment> FilterExprBuilder<C> {
    pub fn new(column_mapping: HashMap<Identifier, ColumnRef>) -> Self {
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

    pub fn add_where_expr(mut self, where_expr: Option<Box<Expression>>) -> Self {
        self.where_expr = WhereExprBuilder::new(&self.column_mapping).build(where_expr);
        self
    }

    pub fn add_result_column_set(mut self, columns: HashSet<Identifier>) -> Self {
        // Sorting is required to make the relative order of the columns deterministic
        let mut columns = columns.into_iter().collect::<Vec<_>>();
        columns.sort();

        columns.into_iter().for_each(|column| {
            let column = *self.column_mapping.get(&column).unwrap();
            self.filter_result_expr_list
                .push(FilterResultExpr::new(column));
        });

        self
    }

    pub fn build(self) -> FilterExpr<C> {
        FilterExpr::new(
            self.filter_result_expr_list,
            self.table_expr.expect("Table expr is required"),
            self.where_expr
                .unwrap_or_else(|| BoolExprPlan::new_const_bool(true)),
        )
    }
}
