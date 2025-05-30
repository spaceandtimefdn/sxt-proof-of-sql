use super::ConversionError;
use crate::{
    base::{
        database::{ColumnRef, LiteralValue},
        map::IndexMap,
        math::{
            decimal::{DecimalError, Precision},
            i256::I256,
            BigDecimalExt,
        },
    },
    sql::{
        parse::{
            dyn_proof_expr_builder::DecimalError::{InvalidPrecision, InvalidScale},
            ConversionError::DecimalConversionError,
        },
        proof_exprs::{ColumnExpr, DynProofExpr},
        scale_cast_binary_op,
    },
};
use alloc::{borrow::ToOwned, boxed::Box, format, string::ToString};
use proof_of_sql_parser::{
    intermediate_ast::{Expression, Literal},
    posql_time::{PoSQLTimeUnit, PoSQLTimestampError},
};
use sqlparser::ast::{BinaryOperator, Ident, UnaryOperator};

/// Builder that enables building a `proofs::sql::proof_exprs::DynProofExpr` from
/// a `proof_of_sql_parser::intermediate_ast::Expression`.
pub struct DynProofExprBuilder<'a> {
    column_mapping: &'a IndexMap<Ident, ColumnRef>,
}

impl<'a> DynProofExprBuilder<'a> {
    /// Creates a new `DynProofExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a IndexMap<Ident, ColumnRef>) -> Self {
        Self { column_mapping }
    }
    /// Builds a `proofs::sql::proof_exprs::DynProofExpr` from a `proof_of_sql_parser::intermediate_ast::Expression`
    pub fn build(&self, expr: &Expression) -> Result<DynProofExpr, ConversionError> {
        self.visit_expr(expr)
    }
}

// Private interface
impl DynProofExprBuilder<'_> {
    fn visit_expr(&self, expr: &Expression) -> Result<DynProofExpr, ConversionError> {
        match expr {
            Expression::Column(identifier) => self.visit_column((*identifier).into()),
            Expression::Literal(lit) => self.visit_literal(lit),
            Expression::Binary { op, left, right } => {
                self.visit_binary_expr(&(*op).into(), left, right)
            }
            Expression::Unary { op, expr } => self.visit_unary_expr((*op).into(), expr),
            _ => Err(ConversionError::Unprovable {
                error: format!("Expression {expr:?} is not supported yet"),
            }),
        }
    }

    fn visit_column(&self, identifier: Ident) -> Result<DynProofExpr, ConversionError> {
        Ok(DynProofExpr::Column(ColumnExpr::new(
            self.column_mapping
                .get(&identifier)
                .ok_or(ConversionError::MissingColumnWithoutTable {
                    identifier: Box::new(identifier),
                })?
                .clone(),
        )))
    }

    #[expect(clippy::unused_self)]
    fn visit_literal(&self, lit: &Literal) -> Result<DynProofExpr, ConversionError> {
        match lit {
            Literal::Boolean(b) => Ok(DynProofExpr::new_literal(LiteralValue::Boolean(*b))),
            Literal::BigInt(i) => Ok(DynProofExpr::new_literal(LiteralValue::BigInt(*i))),
            Literal::Int128(i) => Ok(DynProofExpr::new_literal(LiteralValue::Int128(*i))),
            Literal::VarBinary(bytes) => Ok(DynProofExpr::new_literal(LiteralValue::VarBinary(
                bytes.clone(),
            ))),
            Literal::Decimal(d) => {
                let raw_scale = d.scale();
                let scale = raw_scale.try_into().map_err(|_| InvalidScale {
                    scale: raw_scale.to_string(),
                })?;
                let precision =
                    Precision::try_from(d.precision()).map_err(|_| DecimalConversionError {
                        source: InvalidPrecision {
                            error: d.precision().to_string(),
                        },
                    })?;
                Ok(DynProofExpr::new_literal(LiteralValue::Decimal75(
                    precision,
                    scale,
                    I256::from_num_bigint(
                        &d.try_into_bigint_with_precision_and_scale(precision.value(), scale)?,
                    ),
                )))
            }
            Literal::VarChar(s) => Ok(DynProofExpr::new_literal(LiteralValue::VarChar(s.clone()))),
            Literal::Timestamp(its) => {
                let timestamp = match its.timeunit() {
                    PoSQLTimeUnit::Nanosecond => {
                        its.timestamp().timestamp_nanos_opt().ok_or_else(|| {
                                PoSQLTimestampError::UnsupportedPrecision{ error: "Timestamp out of range: 
                                Valid nanosecond timestamps must be between 1677-09-21T00:12:43.145224192 
                                and 2262-04-11T23:47:16.854775807.".to_owned()
                        }
                        })?
                    }
                    PoSQLTimeUnit::Microsecond => its.timestamp().timestamp_micros(),
                    PoSQLTimeUnit::Millisecond => its.timestamp().timestamp_millis(),
                    PoSQLTimeUnit::Second => its.timestamp().timestamp(),
                };

                Ok(DynProofExpr::new_literal(LiteralValue::TimeStampTZ(
                    its.timeunit().into(),
                    its.timezone().into(),
                    timestamp,
                )))
            }
        }
    }

    fn visit_unary_expr(
        &self,
        op: UnaryOperator,
        expr: &Expression,
    ) -> Result<DynProofExpr, ConversionError> {
        let expr = self.visit_expr(expr);
        match op {
            UnaryOperator::Not => Ok(DynProofExpr::try_new_not(expr?)?),
            // Handle unsupported operators
            _ => Err(ConversionError::UnsupportedOperation {
                message: format!("{op:?}"),
            }),
        }
    }

    fn visit_binary_expr(
        &self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<DynProofExpr, ConversionError> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        // Scaling
        let (left, right) = match op {
            BinaryOperator::Plus
            | BinaryOperator::Minus
            | BinaryOperator::Eq
            | BinaryOperator::Gt
            | BinaryOperator::Lt => scale_cast_binary_op(left, right)?,
            _ => (left, right),
        };
        match op {
            BinaryOperator::And => Ok(DynProofExpr::try_new_and(left, right)?),
            BinaryOperator::Or => Ok(DynProofExpr::try_new_or(left, right)?),
            BinaryOperator::Eq => Ok(DynProofExpr::try_new_equals(left, right)?),
            BinaryOperator::Gt => Ok(DynProofExpr::try_new_inequality(left, right, false)?),
            BinaryOperator::Lt => Ok(DynProofExpr::try_new_inequality(left, right, true)?),
            BinaryOperator::Plus => Ok(DynProofExpr::try_new_add(left, right)?),
            BinaryOperator::Minus => Ok(DynProofExpr::try_new_subtract(left, right)?),
            BinaryOperator::Multiply => Ok(DynProofExpr::try_new_multiply(left, right)?),
            BinaryOperator::Divide => Err(ConversionError::Unprovable {
                error: format!("Binary operator {op:?} is not supported at this location"),
            }),
            _ => {
                // Handle unsupported binary operations
                Err(ConversionError::UnsupportedOperation {
                    message: format!("{op:?}"),
                })
            }
        }
    }
}
