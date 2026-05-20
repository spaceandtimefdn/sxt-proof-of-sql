use super::{
    AddExpr, AndExpr, CastExpr, ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr, MultiplyExpr,
    NotExpr, OrExpr, PlaceholderExpr, ProofExpr, ScalingCastExpr, SubtractExpr,
};
use crate::{
    base::{
        database::{
            Column, ColumnField, ColumnRef, ColumnType, LiteralValue, NullableColumn, Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::{boxed::Box, string::ToString};
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

    /// Return the output field for expressions whose nullability can be derived locally.
    #[must_use]
    pub(crate) fn nullable_propagating_result_field(&self, alias: Ident) -> ColumnField {
        if self
            .nullable_propagating_result_is_nullable()
            .unwrap_or(false)
        {
            ColumnField::new_nullable(alias, self.data_type())
        } else {
            ColumnField::new(alias, self.data_type())
        }
    }

    fn nullable_propagating_result_is_nullable(&self) -> Option<bool> {
        match self {
            Self::Column(column) => Some(column.column_ref().is_nullable()),
            Self::Literal(_) | Self::Placeholder(_) => Some(false),
            Self::Add(expr) => Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs()),
            Self::Subtract(expr) => {
                Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs())
            }
            Self::Multiply(expr) => {
                Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs())
            }
            Self::Equals(expr) => Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs()),
            Self::Inequality(expr) => {
                Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs())
            }
            Self::Not(expr) => expr.input().nullable_propagating_result_is_nullable(),
            Self::Cast(expr) => expr
                .get_from_expr()
                .nullable_propagating_result_is_nullable(),
            Self::ScalingCast(expr) => expr
                .get_from_expr()
                .nullable_propagating_result_is_nullable(),
            Self::And(expr) => Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs()),
            Self::Or(expr) => Self::nullable_binary_result_is_nullable(expr.lhs(), expr.rhs()),
        }
    }

    fn nullable_binary_result_is_nullable(lhs: &Self, rhs: &Self) -> Option<bool> {
        Some(
            lhs.nullable_propagating_result_is_nullable()?
                || rhs.nullable_propagating_result_is_nullable()?,
        )
    }

    /// Return a proof expression for this nullable-propagating result's row presence.
    ///
    /// `None` means the expression is statically non-nullable.
    #[must_use]
    pub fn nullable_result_presence_expr(&self) -> Option<Self> {
        match self {
            Self::Column(column) => column
                .column_ref()
                .is_nullable()
                .then(|| Self::new_column(column.column_ref().presence_column_ref())),
            Self::Literal(_) | Self::Placeholder(_) => None,
            Self::Add(expr) => Self::nullable_binary_result_presence_expr(expr.lhs(), expr.rhs()),
            Self::Subtract(expr) => {
                Self::nullable_binary_result_presence_expr(expr.lhs(), expr.rhs())
            }
            Self::Multiply(expr) => {
                Self::nullable_binary_result_presence_expr(expr.lhs(), expr.rhs())
            }
            Self::Equals(expr) => {
                Self::nullable_binary_result_presence_expr(expr.lhs(), expr.rhs())
            }
            Self::Inequality(expr) => {
                Self::nullable_binary_result_presence_expr(expr.lhs(), expr.rhs())
            }
            Self::Not(expr) => expr.input().nullable_result_presence_expr(),
            Self::Cast(expr) => expr.get_from_expr().nullable_result_presence_expr(),
            Self::ScalingCast(expr) => expr.get_from_expr().nullable_result_presence_expr(),
            Self::And(expr) => {
                Self::nullable_boolean_and_result_presence_expr(expr.lhs(), expr.rhs())
            }
            Self::Or(expr) => {
                Self::nullable_boolean_or_result_presence_expr(expr.lhs(), expr.rhs())
            }
        }
    }

    fn nullable_binary_result_presence_expr(lhs: &Self, rhs: &Self) -> Option<Self> {
        match (
            lhs.nullable_result_presence_expr(),
            rhs.nullable_result_presence_expr(),
        ) {
            (None, None) => None,
            (Some(presence), None) | (None, Some(presence)) => Some(presence),
            (Some(lhs_presence), Some(rhs_presence)) => Some(
                Self::try_new_and(lhs_presence, rhs_presence)
                    .expect("Nullable binary presence expressions are boolean"),
            ),
        }
    }

    fn nullable_boolean_and_result_presence_expr(lhs: &Self, rhs: &Self) -> Option<Self> {
        let lhs_presence = lhs.nullable_result_presence_expr();
        let rhs_presence = rhs.nullable_result_presence_expr();
        if lhs_presence.is_none() && rhs_presence.is_none() {
            return None;
        }

        let lhs_present = Self::presence_expr_or_true(lhs_presence);
        let rhs_present = Self::presence_expr_or_true(rhs_presence);
        let both_present = Self::try_new_and(lhs_present.clone(), rhs_present.clone())
            .expect("Nullable boolean presence expressions are boolean");
        let lhs_present_and_lhs_false = Self::try_new_and(
            lhs_present,
            Self::try_new_not(lhs.clone()).expect("AND operands are boolean"),
        )
        .expect("Nullable boolean presence expressions are boolean");
        let rhs_present_and_rhs_false = Self::try_new_and(
            rhs_present,
            Self::try_new_not(rhs.clone()).expect("AND operands are boolean"),
        )
        .expect("Nullable boolean presence expressions are boolean");

        Some(
            Self::try_new_or(
                Self::try_new_or(both_present, lhs_present_and_lhs_false)
                    .expect("Nullable boolean presence expressions are boolean"),
                rhs_present_and_rhs_false,
            )
            .expect("Nullable boolean presence expressions are boolean"),
        )
    }

    fn nullable_boolean_or_result_presence_expr(lhs: &Self, rhs: &Self) -> Option<Self> {
        let lhs_presence = lhs.nullable_result_presence_expr();
        let rhs_presence = rhs.nullable_result_presence_expr();
        if lhs_presence.is_none() && rhs_presence.is_none() {
            return None;
        }

        let lhs_present = Self::presence_expr_or_true(lhs_presence);
        let rhs_present = Self::presence_expr_or_true(rhs_presence);
        let both_present = Self::try_new_and(lhs_present.clone(), rhs_present.clone())
            .expect("Nullable boolean presence expressions are boolean");
        let lhs_present_and_lhs_true = Self::try_new_and(lhs_present, lhs.clone())
            .expect("Nullable boolean presence expressions are boolean");
        let rhs_present_and_rhs_true = Self::try_new_and(rhs_present, rhs.clone())
            .expect("Nullable boolean presence expressions are boolean");

        Some(
            Self::try_new_or(
                Self::try_new_or(both_present, lhs_present_and_lhs_true)
                    .expect("Nullable boolean presence expressions are boolean"),
                rhs_present_and_rhs_true,
            )
            .expect("Nullable boolean presence expressions are boolean"),
        )
    }

    fn presence_expr_or_true(expr: Option<Self>) -> Self {
        expr.unwrap_or_else(|| Self::new_literal(LiteralValue::Boolean(true)))
    }

    /// Evaluate expressions whose SQL nullability can be derived from local operands.
    pub fn first_round_evaluate_nullable_propagating<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Option<NullableColumn<'a, S>>> {
        let values = self.first_round_evaluate(alloc, table, params)?;
        let result = match self {
            Self::Column(column) => {
                let presence = column.column_ref().is_nullable().then(|| {
                    let presence_column_id = column.column_ref().presence_column_ref().column_id();
                    match table
                        .inner_table()
                        .get(&presence_column_id)
                        .expect("Nullable column presence data should be available")
                    {
                        Column::Boolean(presence) => *presence,
                        _ => panic!("Nullable column presence data should be boolean"),
                    }
                });
                Some(
                    NullableColumn::try_new(values, presence)
                        .expect("Nullable column values and presence should match"),
                )
            }
            Self::Literal(_) | Self::Placeholder(_) => {
                Some(NullableColumn::new_nonnullable(values))
            }
            Self::Add(expr) => Self::nullable_binary_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
            )?,
            Self::Subtract(expr) => Self::nullable_binary_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
            )?,
            Self::Multiply(expr) => Self::nullable_binary_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
            )?,
            Self::Equals(expr) => Self::nullable_binary_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
            )?,
            Self::Inequality(expr) => Self::nullable_binary_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
            )?,
            Self::Not(expr) => {
                Self::nullable_unary_first_round_result(values, expr.input(), alloc, table, params)?
            }
            Self::Cast(expr) => Self::nullable_unary_first_round_result(
                values,
                expr.get_from_expr(),
                alloc,
                table,
                params,
            )?,
            Self::ScalingCast(expr) => Self::nullable_unary_first_round_result(
                values,
                expr.get_from_expr(),
                alloc,
                table,
                params,
            )?,
            Self::And(expr) => Self::nullable_boolean_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
                |lhs_present, lhs_value, rhs_present, rhs_value| {
                    (lhs_present && rhs_present)
                        || (lhs_present && !lhs_value)
                        || (rhs_present && !rhs_value)
                },
            )?,
            Self::Or(expr) => Self::nullable_boolean_first_round_result(
                values,
                expr.lhs(),
                expr.rhs(),
                alloc,
                table,
                params,
                |lhs_present, lhs_value, rhs_present, rhs_value| {
                    (lhs_present && rhs_present)
                        || (lhs_present && lhs_value)
                        || (rhs_present && rhs_value)
                },
            )?,
        };
        Ok(result)
    }

    fn nullable_binary_first_round_result<'a, S: Scalar>(
        values: Column<'a, S>,
        lhs: &Self,
        rhs: &Self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Option<NullableColumn<'a, S>>> {
        let Some(lhs) = lhs.first_round_evaluate_nullable_propagating(alloc, table, params)? else {
            return Ok(None);
        };
        let Some(rhs) = rhs.first_round_evaluate_nullable_propagating(alloc, table, params)? else {
            return Ok(None);
        };
        Ok(Some(
            NullableColumn::try_new(
                values,
                lhs.propagated_binary_presence(&rhs, alloc)
                    .expect("Nullable operand lengths should match"),
            )
            .expect("Nullable expression values and presence should match"),
        ))
    }

    fn nullable_unary_first_round_result<'a, S: Scalar>(
        values: Column<'a, S>,
        input: &Self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Option<NullableColumn<'a, S>>> {
        Ok(input
            .first_round_evaluate_nullable_propagating(alloc, table, params)?
            .map(|input| {
                NullableColumn::try_new(values, input.presence())
                    .expect("Nullable expression values and presence should match")
            }))
    }

    fn nullable_boolean_first_round_result<'a, S: Scalar>(
        values: Column<'a, S>,
        lhs: &Self,
        rhs: &Self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
        is_present: impl Fn(bool, bool, bool, bool) -> bool,
    ) -> PlaceholderResult<Option<NullableColumn<'a, S>>> {
        let Some(lhs) = lhs.first_round_evaluate_nullable_propagating(alloc, table, params)? else {
            return Ok(None);
        };
        let Some(rhs) = rhs.first_round_evaluate_nullable_propagating(alloc, table, params)? else {
            return Ok(None);
        };

        assert_eq!(lhs.len(), rhs.len(), "Boolean operand lengths should match");
        assert_eq!(
            values.len(),
            lhs.len(),
            "Boolean expression values and operand lengths should match"
        );
        let lhs_values = lhs.values().as_boolean().expect("lhs is not boolean");
        let rhs_values = rhs.values().as_boolean().expect("rhs is not boolean");

        let presence: Option<&'a [bool]> = if lhs.presence().is_none() && rhs.presence().is_none() {
            None
        } else {
            Some(&*alloc.alloc_slice_fill_iter(
                lhs_values.iter().zip(rhs_values.iter()).enumerate().map(
                    |(i, (lhs_value, rhs_value))| {
                        let lhs_present = lhs.presence().map_or(true, |presence| presence[i]);
                        let rhs_present = rhs.presence().map_or(true, |presence| presence[i]);
                        is_present(lhs_present, *lhs_value, rhs_present, *rhs_value)
                    },
                ),
            ))
        };

        Ok(Some(NullableColumn::try_new(values, presence).expect(
            "Nullable boolean expression values and presence should match",
        )))
    }

    /// Create a SQL `IS NULL` expression for a column reference.
    ///
    /// Nullable columns are represented by their generated row-presence column.
    /// Non-nullable columns fold to constant `false`.
    #[must_use]
    pub fn new_is_null(column_ref: ColumnRef) -> Self {
        if column_ref.is_nullable() {
            Self::try_new_not(Self::new_column(column_ref.presence_column_ref()))
                .expect("Presence columns are boolean")
        } else {
            Self::new_literal(LiteralValue::Boolean(false))
        }
    }

    /// Create a SQL `IS NOT NULL` expression for a column reference.
    ///
    /// Nullable columns are represented by their generated row-presence column.
    /// Non-nullable columns fold to constant `true`.
    #[must_use]
    pub fn new_is_not_null(column_ref: ColumnRef) -> Self {
        if column_ref.is_nullable() {
            Self::new_column(column_ref.presence_column_ref())
        } else {
            Self::new_literal(LiteralValue::Boolean(true))
        }
    }

    /// Create a SQL `IS TRUE` expression for a boolean column reference.
    ///
    /// Nullable boolean columns are true only when the value is true and the row
    /// is present. Non-nullable boolean columns are already two-valued.
    pub fn try_new_is_true(column_ref: ColumnRef) -> AnalyzeResult<Self> {
        if column_ref.column_type() != &ColumnType::Boolean {
            return Err(AnalyzeError::DataTypeMismatch {
                left_type: column_ref.column_type().to_string(),
                right_type: ColumnType::Boolean.to_string(),
            });
        }

        let value_expr = Self::new_column(column_ref.clone());
        if column_ref.is_nullable() {
            Self::try_new_and(
                value_expr,
                Self::new_column(column_ref.presence_column_ref()),
            )
        } else {
            Ok(value_expr)
        }
    }

    /// Create a SQL `IS FALSE` expression for a boolean column reference.
    ///
    /// Nullable boolean columns are false only when the value is false and the
    /// row is present. Non-nullable boolean columns are already two-valued.
    pub fn try_new_is_false(column_ref: ColumnRef) -> AnalyzeResult<Self> {
        if column_ref.column_type() != &ColumnType::Boolean {
            return Err(AnalyzeError::DataTypeMismatch {
                left_type: column_ref.column_type().to_string(),
                right_type: ColumnType::Boolean.to_string(),
            });
        }

        let value_expr = Self::try_new_not(Self::new_column(column_ref.clone()))?;
        if column_ref.is_nullable() {
            Self::try_new_and(
                value_expr,
                Self::new_column(column_ref.presence_column_ref()),
            )
        } else {
            Ok(value_expr)
        }
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
        database::{table_utility::*, Column, ColumnRef, ColumnType, Table, TableRef},
        scalar::test_scalar::TestScalar,
    };
    use bumpalo::Bump;

    #[test]
    fn is_null_builders_target_presence_columns_for_nullable_refs() {
        let column_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert_eq!(
            DynProofExpr::new_is_not_null(column_ref.clone()),
            DynProofExpr::new_column(column_ref.presence_column_ref())
        );
        assert_eq!(
            DynProofExpr::new_is_null(column_ref.clone()),
            DynProofExpr::try_new_not(DynProofExpr::new_column(column_ref.presence_column_ref()))
                .unwrap()
        );
    }

    #[test]
    fn is_null_builders_fold_constants_for_non_nullable_refs() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert_eq!(
            DynProofExpr::new_is_null(column_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::Boolean(false))
        );
        assert_eq!(
            DynProofExpr::new_is_not_null(column_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true))
        );
    }

    #[test]
    fn is_true_builder_ands_value_with_presence_for_nullable_boolean_refs() {
        let column_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "is_paid".into(),
            ColumnType::Boolean,
        );

        assert_eq!(
            DynProofExpr::try_new_is_true(column_ref.clone()).unwrap(),
            DynProofExpr::try_new_and(
                DynProofExpr::new_column(column_ref.clone()),
                DynProofExpr::new_column(column_ref.presence_column_ref()),
            )
            .unwrap()
        );
    }

    #[test]
    fn is_true_builder_keeps_non_nullable_boolean_refs_direct() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "is_paid".into(),
            ColumnType::Boolean,
        );

        assert_eq!(
            DynProofExpr::try_new_is_true(column_ref.clone()).unwrap(),
            DynProofExpr::new_column(column_ref)
        );
    }

    #[test]
    fn is_true_builder_rejects_non_boolean_refs() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert!(DynProofExpr::try_new_is_true(column_ref).is_err());
    }

    #[test]
    fn is_false_builder_ands_negated_value_with_presence_for_nullable_boolean_refs() {
        let column_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "is_paid".into(),
            ColumnType::Boolean,
        );

        assert_eq!(
            DynProofExpr::try_new_is_false(column_ref.clone()).unwrap(),
            DynProofExpr::try_new_and(
                DynProofExpr::try_new_not(DynProofExpr::new_column(column_ref.clone())).unwrap(),
                DynProofExpr::new_column(column_ref.presence_column_ref()),
            )
            .unwrap()
        );
    }

    #[test]
    fn is_false_builder_keeps_non_nullable_boolean_refs_direct() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "is_paid".into(),
            ColumnType::Boolean,
        );

        assert_eq!(
            DynProofExpr::try_new_is_false(column_ref.clone()).unwrap(),
            DynProofExpr::try_new_not(DynProofExpr::new_column(column_ref)).unwrap()
        );
    }

    #[test]
    fn is_false_builder_rejects_non_boolean_refs() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert!(DynProofExpr::try_new_is_false(column_ref).is_err());
    }

    #[test]
    fn nullable_first_round_evaluation_reads_generated_presence_columns() {
        let alloc = Bump::new();
        let table = nullable_test_table(&alloc);
        let amount_ref = nullable_amount_ref();

        let result = DynProofExpr::new_column(amount_ref)
            .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
            .unwrap()
            .unwrap();

        assert_eq!(result.values(), Column::BigInt(&[10, 20, 30]));
        assert_eq!(result.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_first_round_evaluation_propagates_arithmetic_comparison_presence() {
        let alloc = Bump::new();
        let table = nullable_test_table(&alloc);
        let amount_ref = nullable_amount_ref();
        let fee_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "fee".into(),
            ColumnType::BigInt,
        );
        let amount_plus_fee = DynProofExpr::try_new_add(
            DynProofExpr::new_column(amount_ref),
            DynProofExpr::new_column(fee_ref),
        )
        .unwrap();
        let expression = DynProofExpr::try_new_inequality(
            amount_plus_fee,
            DynProofExpr::new_literal(LiteralValue::BigInt(30)),
            true,
        )
        .unwrap();

        let result = expression
            .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
            .unwrap()
            .unwrap();

        assert_eq!(result.values(), Column::Boolean(&[true, true, false]));
        assert_eq!(result.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_first_round_evaluation_handles_boolean_and_or_presence() {
        let alloc = Bump::new();
        let table = nullable_test_table(&alloc);
        let flag_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "flag".into(),
            ColumnType::Boolean,
        );
        let and_true = DynProofExpr::try_new_and(
            DynProofExpr::new_column(flag_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        )
        .unwrap()
        .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
        .unwrap()
        .unwrap();
        let and_false = DynProofExpr::try_new_and(
            DynProofExpr::new_column(flag_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::Boolean(false)),
        )
        .unwrap()
        .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
        .unwrap()
        .unwrap();
        let or_false = DynProofExpr::try_new_or(
            DynProofExpr::new_column(flag_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::Boolean(false)),
        )
        .unwrap()
        .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
        .unwrap()
        .unwrap();
        let or_true = DynProofExpr::try_new_or(
            DynProofExpr::new_column(flag_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        )
        .unwrap()
        .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
        .unwrap()
        .unwrap();

        assert_eq!(and_true.values(), Column::Boolean(&[true, true, false]));
        assert_eq!(and_true.presence(), Some([true, false, true].as_slice()));
        assert_eq!(and_false.values(), Column::Boolean(&[false, false, false]));
        assert_eq!(and_false.presence(), Some([true, true, true].as_slice()));
        assert_eq!(or_false.values(), Column::Boolean(&[true, true, false]));
        assert_eq!(or_false.presence(), Some([true, false, true].as_slice()));
        assert_eq!(or_true.values(), Column::Boolean(&[true, true, true]));
        assert_eq!(or_true.presence(), Some([true, true, true].as_slice()));
    }

    #[test]
    fn nullable_result_fields_mark_presence_propagating_outputs() {
        let amount_ref = nullable_amount_ref();
        let fee_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "fee".into(),
            ColumnType::BigInt,
        );
        let expression = DynProofExpr::try_new_add(
            DynProofExpr::new_column(amount_ref),
            DynProofExpr::new_column(fee_ref),
        )
        .unwrap();

        let field = expression.nullable_propagating_result_field("total".into());

        assert_eq!(field.name(), "total".into());
        assert_eq!(field.data_type(), expression.data_type());
        assert!(field.is_nullable());
    }

    #[test]
    fn nullable_result_fields_mark_and_or_outputs_nullable() {
        let flag_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "flag".into(),
            ColumnType::Boolean,
        );
        let expression = DynProofExpr::try_new_and(
            DynProofExpr::new_column(flag_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        )
        .unwrap();

        let field = expression.nullable_propagating_result_field("flag_and_true".into());

        assert_eq!(field.name(), "flag_and_true".into());
        assert_eq!(field.data_type(), ColumnType::Boolean);
        assert!(field.is_nullable());
    }

    #[test]
    fn nullable_presence_expression_ands_binary_operand_presence() {
        let amount_ref = nullable_amount_ref();
        let discount_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "discount".into(),
            ColumnType::BigInt,
        );
        let expression = DynProofExpr::try_new_add(
            DynProofExpr::new_column(amount_ref.clone()),
            DynProofExpr::new_column(discount_ref.clone()),
        )
        .unwrap();

        let presence_expr = expression.nullable_result_presence_expr().unwrap();

        assert_eq!(
            presence_expr,
            DynProofExpr::try_new_and(
                DynProofExpr::new_column(amount_ref.presence_column_ref()),
                DynProofExpr::new_column(discount_ref.presence_column_ref()),
            )
            .unwrap()
        );
    }

    #[test]
    fn nullable_boolean_presence_expression_matches_sql_short_circuit_presence() {
        let alloc = Bump::new();
        let table = nullable_test_table(&alloc);
        let flag_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "flag".into(),
            ColumnType::Boolean,
        );

        let and_false_presence = DynProofExpr::try_new_and(
            DynProofExpr::new_column(flag_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::Boolean(false)),
        )
        .unwrap()
        .nullable_result_presence_expr()
        .unwrap()
        .first_round_evaluate(&alloc, &table, &[])
        .unwrap();
        let or_true_presence = DynProofExpr::try_new_or(
            DynProofExpr::new_column(flag_ref),
            DynProofExpr::new_literal(LiteralValue::Boolean(true)),
        )
        .unwrap()
        .nullable_result_presence_expr()
        .unwrap()
        .first_round_evaluate(&alloc, &table, &[])
        .unwrap();

        assert_eq!(and_false_presence, Column::Boolean(&[true, true, true]));
        assert_eq!(or_true_presence, Column::Boolean(&[true, true, true]));
    }

    fn nullable_test_table<'a>(alloc: &'a Bump) -> Table<'a, TestScalar> {
        table([
            borrowed_bigint("amount", [10, 20, 30], alloc),
            borrowed_boolean("__posql_presence_amount", [true, false, true], alloc),
            borrowed_bigint("fee", [5, 5, 5], alloc),
            borrowed_boolean("flag", [true, true, false], alloc),
            borrowed_boolean("__posql_presence_flag", [true, false, true], alloc),
        ])
    }

    fn nullable_amount_ref() -> ColumnRef {
        ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        )
    }
}
