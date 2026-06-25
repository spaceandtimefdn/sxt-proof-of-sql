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
    use super::DynProofExpr;
    use crate::{
        base::database::{ColumnRef, ColumnType, LiteralValue, TableRef},
        sql::proof_exprs::ProofExpr,
    };
    use sqlparser::ast::Ident;

    fn bigint_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(42))
    }

    fn bool_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::Boolean(true))
    }

    #[test]
    fn new_literal_creates_literal_variant() {
        let expr = DynProofExpr::new_literal(LiteralValue::BigInt(1));
        assert!(matches!(expr, DynProofExpr::Literal(_)));
    }

    #[test]
    fn new_literal_bigint_has_correct_data_type() {
        let expr = bigint_expr();
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn new_literal_boolean_has_correct_data_type() {
        let expr = bool_expr();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn new_column_creates_column_variant() {
        let col_ref = ColumnRef::new(TableRef::new("s", "t"), Ident::new("col"), ColumnType::BigInt);
        let expr = DynProofExpr::new_column(col_ref);
        assert!(matches!(expr, DynProofExpr::Column(_)));
    }

    #[test]
    fn try_new_and_with_booleans_returns_ok() {
        let result = DynProofExpr::try_new_and(bool_expr(), bool_expr());
        assert!(result.is_ok());
    }

    #[test]
    fn try_new_and_creates_and_variant() {
        let expr = DynProofExpr::try_new_and(bool_expr(), bool_expr()).unwrap();
        assert!(matches!(expr, DynProofExpr::And(_)));
    }

    #[test]
    fn try_new_not_with_boolean_returns_ok() {
        let result = DynProofExpr::try_new_not(bool_expr());
        assert!(result.is_ok());
    }

    #[test]
    fn try_new_equals_with_same_types_returns_ok() {
        let result = DynProofExpr::try_new_equals(bigint_expr(), bigint_expr());
        assert!(result.is_ok());
    }

    #[test]
    fn try_new_add_with_bigints_returns_ok() {
        let result = DynProofExpr::try_new_add(bigint_expr(), bigint_expr());
        assert!(result.is_ok());
    }

    #[test]
    fn try_new_subtract_with_bigints_returns_ok() {
        let result = DynProofExpr::try_new_subtract(bigint_expr(), bigint_expr());
        assert!(result.is_ok());
    }

    #[test]
    fn try_new_multiply_with_bigints_returns_ok() {
        let result = DynProofExpr::try_new_multiply(bigint_expr(), bigint_expr());
        assert!(result.is_ok());
    }

    #[test]
    fn debug_formatting_contains_expr_type() {
        let expr = bigint_expr();
        assert!(alloc::format!("{expr:?}").contains("Literal") || alloc::format!("{expr:?}").contains("BigInt"));
    }

    #[test]
    fn equality_holds_for_same_literals() {
        assert_eq!(bigint_expr(), bigint_expr());
    }

    #[test]
    fn clone_produces_equal_value() {
        let expr = bigint_expr();
        assert_eq!(expr.clone(), expr);
    }
}
