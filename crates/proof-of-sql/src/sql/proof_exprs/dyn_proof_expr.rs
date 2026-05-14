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
    use super::*;
    use crate::{
        base::{database::TableRef, math::decimal::Precision, proof::PlaceholderError},
        sql::AnalyzeError,
    };

    #[test]
    fn constructors_wrap_the_expected_variants_and_types() {
        let table_ref = TableRef::new("sxt", "t");
        let column_ref = ColumnRef::new(table_ref, "amount".into(), ColumnType::BigInt);
        let column_expr = DynProofExpr::new_column(column_ref.clone());
        assert!(
            matches!(&column_expr, DynProofExpr::Column(expr) if expr.column_ref() == &column_ref)
        );
        assert_eq!(column_expr.data_type(), ColumnType::BigInt);

        let literal_expr = DynProofExpr::new_literal(LiteralValue::Boolean(true));
        assert!(matches!(literal_expr, DynProofExpr::Literal(_)));
        assert_eq!(literal_expr.data_type(), ColumnType::Boolean);

        let placeholder_expr = DynProofExpr::try_new_placeholder(2, ColumnType::VarChar).unwrap();
        assert!(matches!(&placeholder_expr, DynProofExpr::Placeholder(expr) if expr.index() == 1));
        assert_eq!(placeholder_expr.data_type(), ColumnType::VarChar);
    }

    #[test]
    fn boolean_constructors_reject_non_boolean_inputs() {
        let numeric_expr = DynProofExpr::new_literal(LiteralValue::BigInt(7));
        let boolean_expr = DynProofExpr::new_literal(LiteralValue::Boolean(true));

        assert!(DynProofExpr::try_new_not(numeric_expr.clone()).is_err());
        assert!(DynProofExpr::try_new_and(numeric_expr.clone(), boolean_expr.clone()).is_err());
        assert!(DynProofExpr::try_new_or(boolean_expr, numeric_expr).is_err());
    }

    #[test]
    fn comparison_and_arithmetic_constructors_wrap_valid_inputs() {
        let lhs = DynProofExpr::new_literal(LiteralValue::BigInt(7));
        let rhs = DynProofExpr::new_literal(LiteralValue::BigInt(11));

        assert!(matches!(
            DynProofExpr::try_new_equals(lhs.clone(), rhs.clone()).unwrap(),
            DynProofExpr::Equals(_)
        ));
        assert!(matches!(
            DynProofExpr::try_new_inequality(lhs.clone(), rhs.clone(), true).unwrap(),
            DynProofExpr::Inequality(_)
        ));
        assert!(matches!(
            DynProofExpr::try_new_add(lhs.clone(), rhs.clone()).unwrap(),
            DynProofExpr::Add(_)
        ));
        assert!(matches!(
            DynProofExpr::try_new_subtract(lhs.clone(), rhs.clone()).unwrap(),
            DynProofExpr::Subtract(_)
        ));
        assert!(matches!(
            DynProofExpr::try_new_multiply(lhs, rhs).unwrap(),
            DynProofExpr::Multiply(_)
        ));
    }

    #[test]
    fn cast_constructors_wrap_valid_inputs() {
        let bigint_expr = DynProofExpr::new_literal(LiteralValue::BigInt(7));
        let decimal_type = ColumnType::Decimal75(Precision::new(21).unwrap(), 2);

        assert!(matches!(
            DynProofExpr::try_new_cast(bigint_expr.clone(), ColumnType::Int128).unwrap(),
            DynProofExpr::Cast(_)
        ));
        assert!(matches!(
            DynProofExpr::try_new_scaling_cast(bigint_expr, decimal_type).unwrap(),
            DynProofExpr::ScalingCast(_)
        ));
    }

    #[test]
    fn placeholder_constructor_forwards_zero_id_errors() {
        assert!(matches!(
            DynProofExpr::try_new_placeholder(0, ColumnType::Boolean),
            Err(AnalyzeError::PlaceholderError {
                source: PlaceholderError::ZeroPlaceholderId
            })
        ));
    }
}
