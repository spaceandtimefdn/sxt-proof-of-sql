use super::{ConversionError, ConversionResult, QueryContext};
use crate::{
    base::{
        database::{ColumnRef, ColumnType, SchemaAccessor, TableRef},
        map::IndexSet,
        math::{
            decimal::{DecimalError, Precision},
            BigDecimalExt,
        },
    },
    sql::util::check_dtypes,
};
use alloc::{boxed::Box, format, string::ToString, vec::Vec};
use proof_of_sql_parser::{
    intermediate_ast::{
        AggregationOperator, AliasedResultExpr, Expression, Literal, SelectResultExpr, Slice,
        TableExpression,
    },
    Identifier,
};
use sqlparser::ast::{BinaryOperator, Expr, Ident, OrderByExpr, UnaryOperator};
pub struct QueryContextBuilder<'a> {
    context: QueryContext,
    schema_accessor: &'a dyn SchemaAccessor,
}

// Public interface
impl<'a> QueryContextBuilder<'a> {
    pub fn new(schema_accessor: &'a dyn SchemaAccessor) -> Self {
        Self {
            context: QueryContext::default(),
            schema_accessor,
        }
    }

    #[expect(clippy::missing_panics_doc)]
    pub fn visit_table_expr(
        mut self,
        table_expr: &[Box<TableExpression>],
        default_schema: Ident,
    ) -> Self {
        assert_eq!(table_expr.len(), 1);

        match *table_expr[0] {
            TableExpression::Named { table, schema } => {
                let actual_schema = schema.map_or(default_schema, Ident::from);
                let table_ref = TableRef::from_idents(Some(actual_schema), Ident::from(table));
                self.context.set_table_ref(table_ref);
            }
        }

        self
    }

    pub fn visit_where_expr(
        mut self,
        mut where_expr: Option<Box<Expression>>,
    ) -> ConversionResult<Self> {
        if let Some(expr) = where_expr.as_deref_mut() {
            self.visit_expr(expr)?;
        }
        self.context.set_where_expr(where_expr);
        Ok(self)
    }

    pub fn visit_result_exprs(
        mut self,
        result_exprs: Vec<SelectResultExpr>,
    ) -> ConversionResult<Self> {
        self.context.toggle_result_scope();
        for column in result_exprs {
            match column {
                SelectResultExpr::ALL => self.visit_select_all_expr()?,
                SelectResultExpr::AliasedResultExpr(expr) => self.visit_aliased_expr(expr)?,
            }
        }
        self.context.toggle_result_scope();

        Ok(self)
    }

    pub fn visit_order_by_exprs(
        mut self,
        order_by_exprs: Vec<OrderByExpr>,
    ) -> ConversionResult<Self> {
        let ident_direction_pairs: Vec<(Ident, bool)> = order_by_exprs
            .into_iter()
            .map(|order_by_expr| -> ConversionResult<(Ident, bool)> {
                let ident_direction_pair = match order_by_expr {
                    OrderByExpr {
                        expr: Expr::Identifier(id),
                        asc: Some(true) | None,
                        ..
                    } => Ok((id, true)),
                    OrderByExpr {
                        expr: Expr::Identifier(id),
                        asc: Some(false),
                        ..
                    } => Ok((id, false)),
                    _ => Err(ConversionError::UnsupportedOperation {
                        message: "Order by columns other than result columns not supported yet"
                            .to_string(),
                    }),
                }?;
                Ok(ident_direction_pair)
            })
            .collect::<ConversionResult<Vec<_>>>()?;
        // Collect all the result aliases
        let result_aliases = self
            .context
            .get_aliased_result_exprs()?
            .iter()
            .map(|aliased_expr| aliased_expr.alias.into())
            .collect::<IndexSet<Ident>>();
        let index_direction_pairs: Vec<(usize, bool)> = ident_direction_pairs
            .into_iter()
            .map(|(ident, direction)| -> ConversionResult<(usize, bool)> {
                let index = result_aliases.get_index_of(&ident).ok_or(
                    ConversionError::MissingColumnWithoutTable {
                        identifier: Box::new(ident),
                    },
                )?;
                Ok((index, direction))
            })
            .collect::<ConversionResult<Vec<_>>>()?;
        self.context.set_order_by_exprs(index_direction_pairs);
        Ok(self)
    }

    pub fn visit_slice_expr(mut self, slice: Option<Slice>) -> Self {
        self.context.set_slice_expr(slice);
        self
    }

    pub fn visit_group_by_exprs(mut self, group_by_exprs: Vec<Ident>) -> ConversionResult<Self> {
        for id in &group_by_exprs {
            self.visit_column_identifier(id)?;
        }
        self.context.set_group_by_exprs(group_by_exprs);
        Ok(self)
    }

    #[expect(clippy::unnecessary_wraps)]
    pub fn build(self) -> ConversionResult<QueryContext> {
        Ok(self.context)
    }
}

// Private interface
impl QueryContextBuilder<'_> {
    #[expect(
        clippy::missing_panics_doc,
        reason = "The assertion ensures there is at least one column, and this is a fundamental requirement for schema retrieval."
    )]
    fn lookup_schema(&self) -> Vec<(Ident, ColumnType)> {
        let table_ref = self.context.get_table_ref();
        let columns = self.schema_accessor.lookup_schema(table_ref.clone());
        assert!(!columns.is_empty(), "At least one column must exist");
        columns
    }

    fn visit_select_all_expr(&mut self) -> ConversionResult<()> {
        for (column_name, _) in self.lookup_schema() {
            let column_identifier = Identifier::try_from(column_name).map_err(|e| {
                ConversionError::IdentifierConversionError {
                    error: format!("Failed to convert Ident to Identifier: {e}"),
                }
            })?;
            let col_expr = Expression::Column(column_identifier);
            self.visit_aliased_expr(AliasedResultExpr::new(col_expr, column_identifier))?;
        }
        Ok(())
    }

    fn visit_aliased_expr(&mut self, aliased_expr: AliasedResultExpr) -> ConversionResult<()> {
        self.visit_expr(&aliased_expr.expr)?;
        self.context.push_aliased_result_expr(aliased_expr)?;
        Ok(())
    }

    /// Visits the expression and returns its data type.
    fn visit_expr(&mut self, expr: &Expression) -> ConversionResult<ColumnType> {
        match expr {
            Expression::Wildcard => Ok(ColumnType::BigInt), // Since COUNT(*) = COUNT(1)
            Expression::Literal(literal) => self.visit_literal(literal),
            Expression::Column(_) => self.visit_column_expr(expr),
            Expression::Unary { op, expr } => self.visit_unary_expr((*op).into(), expr),
            Expression::Binary { op, left, right } => {
                self.visit_binary_expr(&(*op).into(), left, right)
            }
            Expression::Aggregation { op, expr } => self.visit_agg_expr(*op, expr),
        }
    }

    /// # Panics
    /// Panics if the expression is not a column expression.
    fn visit_column_expr(&mut self, expr: &Expression) -> ConversionResult<ColumnType> {
        let identifier = match expr {
            Expression::Column(identifier) => *identifier,
            _ => panic!("Must be a column expression"),
        };

        self.visit_column_identifier(&identifier.into())
    }

    fn visit_binary_expr(
        &mut self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> ConversionResult<ColumnType> {
        let left_dtype = self.visit_expr(left)?;
        let right_dtype = self.visit_expr(right)?;
        Ok(check_dtypes(left_dtype, right_dtype, op)?)
    }

    fn visit_unary_expr(
        &mut self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> ConversionResult<ColumnType> {
        match op {
            UnaryOperator::Not => {
                let dtype = self.visit_expr(expr)?;
                if dtype != ColumnType::Boolean {
                    return Err(ConversionError::InvalidDataType {
                        expected: ColumnType::Boolean,
                        actual: dtype,
                    });
                }
                Ok(ColumnType::Boolean)
            }
            // Handle unsupported operators
            _ => Err(ConversionError::UnsupportedOperation {
                message: format!("{op:?}"),
            }),
        }
    }

    fn visit_agg_expr(
        &mut self,
        op: AggregationOperator,
        expr: &Expression,
    ) -> ConversionResult<ColumnType> {
        self.context.set_in_agg_scope(true)?;

        let expr_dtype = self.visit_expr(expr)?;

        // We only support sum/max/min aggregations on numeric columns.
        if op != AggregationOperator::Count && expr_dtype == ColumnType::VarChar {
            return Err(ConversionError::non_numeric_expr_in_agg(
                expr_dtype.to_string(),
                op.to_string(),
            ));
        }

        self.context.set_in_agg_scope(false)?;

        // Count aggregation always results in an integer type
        if op == AggregationOperator::Count {
            Ok(ColumnType::BigInt)
        } else {
            Ok(expr_dtype)
        }
    }

    #[expect(clippy::unused_self)]
    fn visit_literal(&self, literal: &Literal) -> Result<ColumnType, ConversionError> {
        match literal {
            Literal::Boolean(_) => Ok(ColumnType::Boolean),
            Literal::BigInt(_) => Ok(ColumnType::BigInt),
            Literal::Int128(_) => Ok(ColumnType::Int128),
            Literal::VarChar(_) => Ok(ColumnType::VarChar),
            Literal::VarBinary(_) => Ok(ColumnType::VarBinary),
            Literal::Decimal(d) => {
                let precision = Precision::try_from(d.precision())?;
                let scale = d.scale();
                Ok(ColumnType::Decimal75(
                    precision,
                    scale.try_into().map_err(|_| DecimalError::InvalidScale {
                        scale: scale.to_string(),
                    })?,
                ))
            }
            Literal::Timestamp(its) => Ok(ColumnType::TimestampTZ(
                its.timeunit().into(),
                its.timezone().into(),
            )),
        }
    }

    fn visit_column_identifier(&mut self, column_name: &Ident) -> ConversionResult<ColumnType> {
        let table_ref = self.context.get_table_ref();
        let column_type = self
            .schema_accessor
            .lookup_column(table_ref.clone(), column_name.clone());

        let column_type = column_type.ok_or_else(|| ConversionError::MissingColumn {
            identifier: Box::new(column_name.clone()),
            table_ref: table_ref.clone(),
        })?;

        let column = ColumnRef::new(table_ref.clone(), column_name.clone(), column_type);

        self.context.push_column_ref(column_name.clone(), column);

        Ok(column_type)
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::QueryContextBuilder;
    use crate::{
        base::{
            database::{ColumnType, TableRef},
            map::indexmap,
            math::decimal::Precision,
        },
        sql::parse::query_expr_tests::schema_accessor_from_table_ref_with_schema,
    };
    use proof_of_sql_parser::{
        intermediate_ast::{Expression, TableExpression},
        Identifier,
    };
    use sqlparser::ast::{BinaryOperator, Ident};

    #[test]
    fn we_can_correctly_visit_binary_expr_for_bigger_right_type() {
        let t = TableRef::new("sxt", "tab");
        let accessor = schema_accessor_from_table_ref_with_schema(
            &t,
            indexmap! {
                "a".into() => ColumnType::Int,
                "b".into() => ColumnType::BigInt,
            },
        );
        let query_context_builder = QueryContextBuilder::new(&accessor);
        let res = query_context_builder
            .visit_table_expr(
                &[Box::new(TableExpression::Named {
                    table: "tab".parse().unwrap(),
                    schema: None,
                })],
                Ident::new("sxt"),
            )
            .visit_binary_expr(
                &BinaryOperator::Plus,
                &Expression::Column(Identifier::try_new("a").unwrap()),
                &Expression::Column(Identifier::try_new("b").unwrap()),
            )
            .unwrap();
        assert_eq!(res, ColumnType::Decimal75(Precision::new(20).unwrap(), 0));
    }
}
