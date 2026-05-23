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
    use crate::base::{
        database::{ColumnRef, TableRef},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    };

    fn bool_column() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::new("sxt", "t"),
            "a".into(),
            ColumnType::Boolean,
        ))
    }

    fn bigint_column() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::new("sxt", "t"),
            "b".into(),
            ColumnType::BigInt,
        ))
    }

    fn smallint_column() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::new("sxt", "t"),
            "c".into(),
            ColumnType::SmallInt,
        ))
    }

    // new_column
    #[test]
    fn we_can_create_a_column_expr() {
        let expr = bool_column();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    // new_literal
    #[test]
    fn we_can_create_a_literal_expr_for_all_supported_types() {
        let cases = [
            (LiteralValue::Boolean(true), ColumnType::Boolean),
            (LiteralValue::TinyInt(1), ColumnType::TinyInt),
            (LiteralValue::SmallInt(2), ColumnType::SmallInt),
            (LiteralValue::Int(3), ColumnType::Int),
            (LiteralValue::BigInt(4), ColumnType::BigInt),
            (LiteralValue::Int128(5), ColumnType::Int128),
            (LiteralValue::VarChar("hello".into()), ColumnType::VarChar),
            (
                LiteralValue::TimeStampTZ(
                    PoSQLTimeUnit::Second,
                    PoSQLTimeZone::utc(),
                    1_000_000_000,
                ),
                ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
            ),
        ];
        for (value, expected_type) in cases {
            let expr = DynProofExpr::new_literal(value);
            assert_eq!(expr.data_type(), expected_type);
        }
    }

    // try_new_placeholder
    #[test]
    fn we_can_create_a_placeholder_expr() {
        let expr = DynProofExpr::try_new_placeholder(1, ColumnType::BigInt).unwrap();
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_create_multiple_placeholder_exprs_with_different_ids() {
        let expr1 = DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap();
        let expr2 = DynProofExpr::try_new_placeholder(2, ColumnType::SmallInt).unwrap();
        assert_eq!(expr1.data_type(), ColumnType::Int);
        assert_eq!(expr2.data_type(), ColumnType::SmallInt);
    }

    // try_new_and
    #[test]
    fn we_can_create_an_and_expr_from_two_boolean_columns() {
        let lhs = bool_column();
        let rhs = bool_column();
        let expr = DynProofExpr::try_new_and(lhs, rhs).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_cannot_create_an_and_expr_from_non_boolean_columns() {
        let lhs = bigint_column();
        let rhs = bool_column();
        assert!(DynProofExpr::try_new_and(lhs, rhs).is_err());
    }

    // try_new_or
    #[test]
    fn we_can_create_an_or_expr_from_two_boolean_columns() {
        let lhs = bool_column();
        let rhs = bool_column();
        let expr = DynProofExpr::try_new_or(lhs, rhs).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_cannot_create_an_or_expr_from_non_boolean_columns() {
        let lhs = bigint_column();
        let rhs = bigint_column();
        assert!(DynProofExpr::try_new_or(lhs, rhs).is_err());
    }

    // try_new_not
    #[test]
    fn we_can_create_a_not_expr_from_a_boolean_column() {
        let expr = bool_column();
        let not_expr = DynProofExpr::try_new_not(expr).unwrap();
        assert_eq!(not_expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_cannot_create_a_not_expr_from_a_non_boolean_column() {
        let expr = bigint_column();
        assert!(DynProofExpr::try_new_not(expr).is_err());
    }

    // try_new_equals
    #[test]
    fn we_can_create_an_equals_expr_for_bigint_columns() {
        let lhs = bigint_column();
        let rhs = bigint_column();
        let expr = DynProofExpr::try_new_equals(lhs, rhs).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_can_create_an_equals_expr_for_boolean_columns() {
        let lhs = bool_column();
        let rhs = bool_column();
        let expr = DynProofExpr::try_new_equals(lhs, rhs).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    // try_new_inequality
    #[test]
    fn we_can_create_an_inequality_expr_for_bigint_columns() {
        let lhs = bigint_column();
        let rhs = bigint_column();
        let lt_expr = DynProofExpr::try_new_inequality(lhs.clone(), rhs.clone(), true).unwrap();
        let gt_expr = DynProofExpr::try_new_inequality(lhs, rhs, false).unwrap();
        assert_eq!(lt_expr.data_type(), ColumnType::Boolean);
        assert_eq!(gt_expr.data_type(), ColumnType::Boolean);
    }

    // try_new_add
    #[test]
    fn we_can_create_an_add_expr_for_matching_numeric_types() {
        let lhs = bigint_column();
        let rhs = bigint_column();
        let expr = DynProofExpr::try_new_add(lhs, rhs).unwrap();
        assert!(expr.data_type().is_numeric());
    }

    #[test]
    fn we_cannot_create_an_add_expr_for_boolean_columns() {
        let lhs = bool_column();
        let rhs = bool_column();
        assert!(DynProofExpr::try_new_add(lhs, rhs).is_err());
    }

    // try_new_subtract
    #[test]
    fn we_can_create_a_subtract_expr_for_matching_numeric_types() {
        let lhs = bigint_column();
        let rhs = bigint_column();
        let expr = DynProofExpr::try_new_subtract(lhs, rhs).unwrap();
        assert!(expr.data_type().is_numeric());
    }

    // try_new_multiply
    #[test]
    fn we_can_create_a_multiply_expr_for_numeric_types() {
        let lhs = smallint_column();
        let rhs = smallint_column();
        let expr = DynProofExpr::try_new_multiply(lhs, rhs).unwrap();
        assert!(expr.data_type().is_numeric());
    }

    #[test]
    fn we_cannot_create_a_multiply_expr_for_boolean_columns() {
        let lhs = bool_column();
        let rhs = bool_column();
        assert!(DynProofExpr::try_new_multiply(lhs, rhs).is_err());
    }

    // try_new_cast
    #[test]
    fn we_can_create_a_cast_expr_from_smallint_to_bigint() {
        let from = smallint_column();
        let expr = DynProofExpr::try_new_cast(from, ColumnType::BigInt).unwrap();
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    // try_new_scaling_cast
    #[test]
    fn we_can_create_a_scaling_cast_expr_from_smallint_to_decimal() {
        let from = smallint_column();
        let to_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let expr = DynProofExpr::try_new_scaling_cast(from, to_type).unwrap();
        assert_eq!(expr.data_type(), to_type);
    }

    // PartialEq, Clone
    #[test]
    fn we_can_clone_and_compare_dyn_proof_exprs() {
        let expr = bigint_column();
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn we_can_debug_print_a_dyn_proof_expr() {
        let expr = bigint_column();
        let debug_str = format!("{expr:?}");
        assert!(debug_str.contains("Column"));
    }

    // serde round-trip
    #[test]
    fn we_can_serialize_and_deserialize_a_column_dyn_proof_expr() {
        let expr = bigint_column();
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: DynProofExpr = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn we_can_serialize_and_deserialize_a_literal_dyn_proof_expr() {
        let expr = DynProofExpr::new_literal(LiteralValue::BigInt(42));
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: DynProofExpr = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }
}
