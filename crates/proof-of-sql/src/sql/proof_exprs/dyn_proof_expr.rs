use super::{
    AddExpr, AndExpr, CastExpr, ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr, MultiplyExpr,
    NotExpr, OrExpr, PlaceholderExpr, ProofExpr, ScalingCastExpr, SubtractExpr,
};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeResult,
    },
};
use alloc::boxed::Box;
use bumpalo::Bump;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Enum of AST column expression types that implement `ProofExpr`. Is itself a `ProofExpr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[enum_dispatch::enum_dispatch]
pub enum DynProofExpr {
    /// Column
    Column(ColumnExpr),
    /// Provable logical AND expression
    And(AndExpr),
    /// Provable logical OR expression
    Or(OrExpr),
    /// Provable logical NOT expression
    Not(NotExpr),
    /// Provable CONST expression
    Literal(LiteralExpr),
    /// Provable placeholder expression
    Placeholder(PlaceholderExpr),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr),
    /// Provable numeric `+` expression
    Add(AddExpr),
    /// Provable numeric `-` expression
    Subtract(SubtractExpr),
    /// Provable numeric `*` expression
    Multiply(MultiplyExpr),
    /// Provable CAST expression
    Cast(CastExpr),
    /// Provable expression for casting numeric expressions to decimal expressions
    ScalingCast(ScalingCastExpr),
}
impl DynProofExpr {
    /// Create column expression
    #[must_use]
    pub fn new_column(column_ref: ColumnRef) -> Self {
        Self::Column(ColumnExpr::new(column_ref))
    }
    /// Create logical AND expression
    pub fn try_new_and(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        AndExpr::try_new(Box::new(lhs), Box::new(rhs)).map(DynProofExpr::And)
    }
    /// Create logical OR expression
    pub fn try_new_or(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        OrExpr::try_new(Box::new(lhs), Box::new(rhs)).map(DynProofExpr::Or)
    }
    /// Create logical NOT expression
    pub fn try_new_not(expr: DynProofExpr) -> AnalyzeResult<Self> {
        NotExpr::try_new(Box::new(expr)).map(DynProofExpr::Not)
    }
    /// Create CONST expression
    #[must_use]
    pub fn new_literal(value: LiteralValue) -> Self {
        Self::Literal(LiteralExpr::new(value))
    }
    /// Create placeholder expression
    pub fn try_new_placeholder(id: usize, column_type: ColumnType) -> AnalyzeResult<Self> {
        Ok(Self::Placeholder(PlaceholderExpr::try_new(
            id,
            column_type,
        )?))
    }
    /// Create a new equals expression
    pub fn try_new_equals(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        EqualsExpr::try_new(Box::new(lhs), Box::new(rhs)).map(DynProofExpr::Equals)
    }
    /// Create a new inequality expression
    pub fn try_new_inequality(
        lhs: DynProofExpr,
        rhs: DynProofExpr,
        is_lt: bool,
    ) -> AnalyzeResult<Self> {
        InequalityExpr::try_new(Box::new(lhs), Box::new(rhs), is_lt).map(DynProofExpr::Inequality)
    }

    /// Create a new add expression
    pub fn try_new_add(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        AddExpr::try_new(Box::new(lhs), Box::new(rhs)).map(DynProofExpr::Add)
    }

    /// Create a new subtract expression
    pub fn try_new_subtract(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        SubtractExpr::try_new(Box::new(lhs), Box::new(rhs)).map(DynProofExpr::Subtract)
    }

    /// Create a new multiply expression
    pub fn try_new_multiply(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        MultiplyExpr::try_new(Box::new(lhs), Box::new(rhs)).map(DynProofExpr::Multiply)
    }

    /// Create a new cast expression
    pub fn try_new_cast(from_column: DynProofExpr, to_datatype: ColumnType) -> AnalyzeResult<Self> {
        CastExpr::try_new(Box::new(from_column), to_datatype).map(DynProofExpr::Cast)
    }

    /// Create a new decimal scale cast expression
    pub fn try_new_scaling_cast(
        from_expr: DynProofExpr,
        to_datatype: ColumnType,
    ) -> AnalyzeResult<Self> {
        ScalingCastExpr::try_new(Box::new(from_expr), to_datatype).map(DynProofExpr::ScalingCast)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::proof_exprs::{DynProofExpr, ProofExpr},
    };

    fn bool_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::Boolean(true))
    }

    fn bigint_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(42))
    }

    #[test]
    fn new_literal_boolean_has_boolean_type() {
        assert_eq!(bool_expr().data_type(), ColumnType::Boolean);
    }

    #[test]
    fn new_literal_bigint_has_bigint_type() {
        assert_eq!(bigint_expr().data_type(), ColumnType::BigInt);
    }

    #[test]
    fn try_new_placeholder_valid_index_succeeds() {
        let expr = DynProofExpr::try_new_placeholder(1, ColumnType::Boolean).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn try_new_placeholder_zero_index_is_error() {
        assert!(DynProofExpr::try_new_placeholder(0, ColumnType::Boolean).is_err());
    }

    #[test]
    fn try_new_not_of_boolean_has_boolean_type() {
        let expr = DynProofExpr::try_new_not(bool_expr()).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn try_new_not_of_bigint_is_error() {
        assert!(DynProofExpr::try_new_not(bigint_expr()).is_err());
    }

    #[test]
    fn try_new_and_of_two_booleans_succeeds() {
        let expr = DynProofExpr::try_new_and(bool_expr(), bool_expr()).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn try_new_or_of_two_booleans_succeeds() {
        let expr = DynProofExpr::try_new_or(bool_expr(), bool_expr()).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn try_new_add_of_two_bigints_gives_decimal_type() {
        let expr = DynProofExpr::try_new_add(bigint_expr(), bigint_expr()).unwrap();
        assert!(matches!(expr.data_type(), ColumnType::Decimal75(..)));
    }

    #[test]
    fn try_new_add_boolean_and_bigint_is_error() {
        assert!(DynProofExpr::try_new_add(bool_expr(), bigint_expr()).is_err());
    }

    #[test]
    fn try_new_subtract_of_two_bigints_succeeds() {
        assert!(DynProofExpr::try_new_subtract(bigint_expr(), bigint_expr()).is_ok());
    }

    #[test]
    fn try_new_multiply_of_two_bigints_succeeds() {
        assert!(DynProofExpr::try_new_multiply(bigint_expr(), bigint_expr()).is_ok());
    }

    #[test]
    fn clone_creates_equal_expression() {
        let expr = bool_expr();
        assert_eq!(expr.clone(), expr);
    }

    #[test]
    fn equal_literals_compare_equal() {
        assert_eq!(bool_expr(), bool_expr());
    }

    #[test]
    fn different_literal_types_compare_unequal() {
        assert_ne!(bool_expr(), bigint_expr());
    }
}
