use crate::{base::database::ColumnRef, sql::ast::BoolExprPlan};
use curve25519_dalek::RistrettoPoint;
use proofs_sql::{
    intermediate_ast::{BinaryOperator, Expression, Literal, UnaryOperator},
    Identifier,
};
use std::collections::HashMap;

/// Buildder that enables building a `proofs::sql::ast::BoolExpr` from a `proofs_sql::intermediate_ast::Expression` that is
/// intended to be used as the where clause in a filter expression or group by expression.
pub struct WhereExprBuilder<'a> {
    column_mapping: &'a HashMap<Identifier, ColumnRef>,
}
impl<'a> WhereExprBuilder<'a> {
    /// Creates a new `WhereExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a HashMap<Identifier, ColumnRef>) -> Self {
        Self { column_mapping }
    }
    /// Builds a `proofs::sql::ast::BoolExpr` from a `proofs_sql::intermediate_ast::Expression` that is
    /// intended to be used as the where clause in a filter expression or group by expression.
    pub fn build(
        self,
        where_expr: Option<Box<Expression>>,
    ) -> Option<BoolExprPlan<RistrettoPoint>> {
        where_expr.map(|where_expr| self.visit_expr(*where_expr))
    }
}

// Private interface
impl WhereExprBuilder<'_> {
    fn visit_expr(
        &self,
        expr: proofs_sql::intermediate_ast::Expression,
    ) -> BoolExprPlan<RistrettoPoint> {
        match expr {
            Expression::Binary { op, left, right } => self.visit_binary_expr(op, *left, *right),
            Expression::Unary { op, expr } => self.visit_unary_expr(op, *expr),
            _ => panic!("The parser must ensure that the expression is a boolean expression"),
        }
    }

    fn visit_unary_expr(
        &self,
        op: UnaryOperator,
        expr: Expression,
    ) -> BoolExprPlan<RistrettoPoint> {
        let expr = self.visit_expr(expr);

        match op {
            UnaryOperator::Not => BoolExprPlan::new_not(expr),
        }
    }

    fn visit_binary_expr(
        &self,
        op: BinaryOperator,
        left: Expression,
        right: Expression,
    ) -> BoolExprPlan<RistrettoPoint> {
        match op {
            BinaryOperator::And => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                BoolExprPlan::new_and(left, right)
            }
            BinaryOperator::Or => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);
                BoolExprPlan::new_or(left, right)
            }
            BinaryOperator::Equal => self.visit_equal_expr(left, right),
            _ => panic!("The parser must ensure that the expression is a boolean expression"),
        }
    }

    fn visit_equal_expr(
        &self,
        left: Expression,
        right: Expression,
    ) -> BoolExprPlan<RistrettoPoint> {
        let left = match left {
            Expression::Column(identifier) => *self.column_mapping.get(&identifier).unwrap(),
            _ => panic!("The parser must ensure that the left side is a column"),
        };

        let right = match right {
            Expression::Literal(literal) => match literal {
                Literal::Int128(value) => value.into(),
                Literal::VarChar(value) => value.into(),
            },
            _ => panic!("The parser must ensure that the left side is a literal"),
        };

        BoolExprPlan::new_equals(left, right)
    }
}
