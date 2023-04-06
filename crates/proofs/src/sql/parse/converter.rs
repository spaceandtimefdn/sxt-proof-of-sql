use super::ResultColumnAliasGraph;
use crate::base::database::{ColumnRef, ColumnType, SchemaAccessor, TableRef};
use crate::base::scalar::ToScalar;
use crate::sql::ast::{
    AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};
use crate::sql::parse::{ConversionError, ConversionResult, QueryExpr, ResultExprBuilder};
use crate::sql::transform::ResultExpr;

use curve25519_dalek::scalar::Scalar;
use proofs_sql::intermediate_ast::OrderBy;
use proofs_sql::intermediate_ast::{
    Expression, Literal, ResultColumn, SetExpression, TableExpression,
};
use proofs_sql::{Identifier, ResourceId, SelectStatement};
use std::ops::Deref;

#[derive(Default)]
pub struct Converter {
    /// The current table in context
    current_table: Option<TableRef>,
    result_column_alias_graph: Option<ResultColumnAliasGraph>,
}

impl Converter {
    /// Convert an Intermediate AST into a Provable AST
    ///
    /// # Parameters:
    ///
    /// ast: the proper intermediate ast to be converted into a provable ast.
    ///
    /// schema_accessor: this accessor is particularly useful
    ///     to allow us to check if a given column exists in a
    ///     given `schema_table.table_name` as well as check
    ///     its type. We also use it to fetch all columns
    ///     existing in a given `schema_table.table_name`,
    ///     necessary to convert a `select * from T` intermediate ast
    ///     into the provable ast.
    ///
    /// default_schema: in case no schema is specified in the given
    ///     intermediate ast, we use this `default_schema` to
    ///     create the `TableRef`. Otherwise, we use the already
    ///     SelectStatements' schema to create the `TableRef`.
    ///
    /// # Return:
    ///
    /// The provable ast, wrapped inside a parse result.
    pub fn visit_intermediate_ast(
        &mut self,
        ast: &SelectStatement,
        schema_accessor: &dyn SchemaAccessor,
        default_schema: Identifier,
    ) -> ConversionResult<QueryExpr> {
        let filter_expr =
            self.visit_set_expression(ast.expr.deref(), schema_accessor, default_schema)?;

        self.result_column_alias_graph =
            Some(ResultColumnAliasGraph::new(filter_expr.get_results()));

        let result_expr = self.build_result_expr(ast)?;

        Ok(QueryExpr::new(Box::new(filter_expr), Box::new(result_expr)))
    }
}

/// Visit intermediate ast
impl Converter {
    /// Convert a `SetExpression` into a `FilterExpr`
    fn visit_set_expression(
        &mut self,
        expr: &SetExpression,
        schema_accessor: &dyn SchemaAccessor,
        default_schema: Identifier,
    ) -> ConversionResult<FilterExpr> {
        match expr {
            SetExpression::Query {
                columns,
                from,
                where_expr,
            } => {
                // we should always visit table_expr first, as we need to know the current table name
                let table = self.visit_table_expressions(&from[..], default_schema);

                let filter_result_expr_list =
                    self.visit_result_columns(&columns[..], schema_accessor)?;

                let where_clause = match where_expr {
                    Some(where_expr) => {
                        self.visit_bool_expression(where_expr.deref(), schema_accessor)?
                    }
                    None => Box::new(ConstBoolExpr::new(true)),
                };

                Ok(FilterExpr::new(
                    filter_result_expr_list,
                    table,
                    where_clause,
                ))
            }
        }
    }
}

/// Table expression
impl Converter {
    /// Convert a `TableExpression` into a TableExpr
    fn visit_table_expression(
        &mut self,
        table_expr: &TableExpression,
        default_schema: Identifier,
    ) -> TableExpr {
        match table_expr {
            TableExpression::Named { table, schema } => {
                let schema = schema.unwrap_or(default_schema);

                let table_ref = TableRef::new(ResourceId::new(schema, *table));

                self.current_table = Some(table_ref);

                TableExpr { table_ref }
            }
        }
    }

    /// Convert a `TableExpression slice` into a `TableExpr`
    fn visit_table_expressions(
        &mut self,
        table_exprs: &[Box<TableExpression>],
        default_schema: Identifier,
    ) -> TableExpr {
        assert!(table_exprs.len() == 1);

        self.visit_table_expression(table_exprs[0].deref(), default_schema)
    }
}

/// Utilities methods
impl Converter {
    fn check_ambiguous_alias_name(&self, alias_name: &Identifier) -> ConversionResult<()> {
        let result_column_alias_graph = self.result_column_alias_graph.as_ref().unwrap();

        if result_column_alias_graph
            .get_alias_mapping(alias_name)
            .expect("alias name must exist at this point")
            .len()
            != 1
        {
            // multiple column names associated with the same alias are not allowed
            return Err(ConversionError::AmbiguousOrderByError(
                alias_name.to_string(),
            ));
        }

        Ok(())
    }

    fn try_to_remap_column_name_name_to_alias(
        &self,
        column_name: &Identifier,
    ) -> ConversionResult<Identifier> {
        let result_column_alias_graph = self.result_column_alias_graph.as_ref().unwrap();

        // the same column name can be associated with multiple aliases
        // this is the case when we have a `select a as b, c, a as d from T order by a`
        // we pick the first alias available as order by should have the same effect in both cases
        let alias_name = result_column_alias_graph
            .get_name_mapping(column_name)
            .map(|v| *v.iter().next().unwrap());

        // we return an error if the `column_name` is not associated with any existing column name
        alias_name.ok_or(ConversionError::InvalidOrderByError(
            column_name.name().to_string(),
            self.current_table.unwrap().table_id().name().to_string(),
        ))
    }

    fn maybe_remap_column_name_to_alias(
        &self,
        maybe_column_name_or_alias: &Identifier,
    ) -> ConversionResult<Identifier> {
        let result_column_alias_graph = self.result_column_alias_graph.as_ref().unwrap();

        // Check if `maybe_column_name_or_alias` is already associated with an alias name.
        match result_column_alias_graph.get_alias_mapping(maybe_column_name_or_alias) {
            Some(_) => {
                // `maybe_column_name_or_alias` is an alias name.
                // so it will reference the correct column in the result record batch.
                let alias_name = *maybe_column_name_or_alias;

                Ok(alias_name)
            }
            None => {
                // `maybe_column_name_or_alias` may be a column name.
                let maybe_column_name = maybe_column_name_or_alias;

                // thus, we try to remap it to the alias name associated with column name.
                self.try_to_remap_column_name_name_to_alias(maybe_column_name)
            }
        }
    }
}

/// Build result expr
impl Converter {
    fn build_result_expr(&self, ast: &SelectStatement) -> ConversionResult<ResultExpr> {
        let mut result_expr_builder = ResultExprBuilder::default();

        result_expr_builder.add_order_by(self.visit_order_by(&ast.order_by[..])?);

        if let Some(slice) = &ast.slice {
            result_expr_builder.add_slice(slice.number_rows, slice.offset_value);
        }

        Ok(result_expr_builder.build())
    }
}

// Order By
impl Converter {
    fn visit_order_by(&self, by_exprs: &[OrderBy]) -> ConversionResult<Vec<OrderBy>> {
        let by_exprs = by_exprs
            .iter()
            .map(|by_expr| {
                // `by_expr.expr` can be either an alias or a column name
                // - if it's an alias, it will already be associated with the correct column
                // - if it's a column name, we need to remap it to the alias name used in the result record batch
                let alias_name = self.maybe_remap_column_name_to_alias(&by_expr.expr)?;

                // check that the alias name is not ambiguous
                // (i.e. that it's not associated with multiple and different column names)
                self.check_ambiguous_alias_name(&alias_name)?;

                // return a new `OrderBy` with the correct alias name
                Ok(OrderBy {
                    expr: alias_name,
                    direction: by_expr.direction.clone(),
                })
            })
            .collect::<ConversionResult<Vec<_>>>()?;

        Ok(by_exprs)
    }
}

/// Result expression
impl Converter {
    /// Convert a `ResultColumn::All` into a `Vec<FilterResultExpr>`
    fn visit_result_column_all(
        &self,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Vec<FilterResultExpr>> {
        let current_table = *self
            .current_table
            .as_ref()
            .expect("Some table should've already been processed at this point");
        let columns = schema_accessor.lookup_schema(current_table);

        Ok(columns
            .into_iter()
            .map(|(column_name_id, column_type)| {
                FilterResultExpr::new(
                    ColumnRef::new(current_table, column_name_id, column_type),
                    column_name_id,
                )
            })
            .collect())
    }

    /// Convert a `ResultColumn::Expr` into a `FilterResultExpr`
    fn visit_result_column_expression(
        &self,
        column_name: Identifier,
        output_name: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<FilterResultExpr> {
        let result_expr = self.visit_column_identifier(column_name, schema_accessor)?;
        Ok(FilterResultExpr::new(result_expr, output_name))
    }

    /// Convert a `ResultColumn` into a `Vec<FilterResultExpr>`
    fn visit_result_column(
        &self,
        result_column: &ResultColumn,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Vec<FilterResultExpr>> {
        match result_column {
            ResultColumn::All => self.visit_result_column_all(schema_accessor),
            ResultColumn::Expr { expr, output_name } => {
                Ok(vec![self.visit_result_column_expression(
                    *expr,
                    *output_name,
                    schema_accessor,
                )?])
            }
        }
    }

    /// Convert a `ResultColumn slice` into a `Vec<FilterResultExpr>`
    fn visit_result_columns(
        &self,
        result_columns: &[Box<ResultColumn>],
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Vec<FilterResultExpr>> {
        assert!(!result_columns.is_empty());

        let results = result_columns
            .iter()
            .map(|result_column| self.visit_result_column(result_column.deref(), schema_accessor))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(results)
    }
}

/// Where expression
impl Converter {
    /// Convert an `Expression` into a BoolExpr
    fn visit_bool_expression(
        &self,
        expression: &Expression,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Box<dyn BoolExpr>> {
        match expression {
            Expression::Not { expr } => Ok(Box::new(NotExpr::new(
                self.visit_bool_expression(expr.deref(), schema_accessor)?,
            ))),

            Expression::And { left, right } => Ok(Box::new(AndExpr::new(
                self.visit_bool_expression(left.deref(), schema_accessor)?,
                self.visit_bool_expression(right.deref(), schema_accessor)?,
            ))),

            Expression::Or { left, right } => Ok(Box::new(OrExpr::new(
                self.visit_bool_expression(left.deref(), schema_accessor)?,
                self.visit_bool_expression(right.deref(), schema_accessor)?,
            ))),

            Expression::Equal { left, right } => {
                self.visit_equals_expression(*left, right, schema_accessor)
            }
        }
    }

    /// Convert an `Expression` into an EqualsExpr
    fn visit_equals_expression(
        &self,
        left: Identifier,
        right: &Literal,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<Box<dyn BoolExpr>> {
        let (scalar, dtype) = self.visit_literal(right.deref());
        let column_ref = self.visit_column_identifier(left, schema_accessor)?;

        if *column_ref.column_type() != dtype {
            return Err(ConversionError::MismatchTypeError(format!(
                "Literal \"{:?}\" has type {:?} but column \"{:?}\" from table \"{:?}\" has type {:?}",
                right.deref(),
                dtype,
                column_ref.column_id(),
                column_ref.table_ref(),
                column_ref.column_type()
            )));
        }

        Ok(Box::new(EqualsExpr::new(column_ref, scalar)))
    }
}

/// Tokens (literals and id's)
impl Converter {
    fn visit_literal(&self, literal: &Literal) -> (Scalar, ColumnType) {
        match literal {
            Literal::BigInt(val) => (val.to_scalar(), ColumnType::BigInt),
            Literal::VarChar(val) => (val.to_scalar(), ColumnType::VarChar),
        }
    }

    /// Convert a `Name` into an identifier string (i.e. a string)
    fn visit_column_identifier(
        &self,
        column_name: Identifier,
        schema_accessor: &dyn SchemaAccessor,
    ) -> ConversionResult<ColumnRef> {
        let current_table = *self
            .current_table
            .as_ref()
            .expect("Some table should've already been processed at this point");

        let column_type = schema_accessor.lookup_column(current_table, column_name);
        let column_type = column_type.ok_or_else(|| {
            ConversionError::MissingColumnError(format!(
                "Column \"{}\" is not found in table \"{}\"",
                column_name,
                current_table.table_id()
            ))
        })?;

        Ok(ColumnRef::new(current_table, column_name, column_type))
    }
}
