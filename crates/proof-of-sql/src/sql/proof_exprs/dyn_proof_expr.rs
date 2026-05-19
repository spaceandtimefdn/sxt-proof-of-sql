use super::{
    AddExpr, AndExpr, CastExpr, ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr, MultiplyExpr,
    NotExpr, OrExpr, PlaceholderExpr, ProofExpr, ScalingCastExpr, SubtractExpr,
};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, NullableColumn, Table},
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

    /// Evaluate expressions whose SQL nullability is simple operand-presence propagation.
    ///
    /// This returns `None` for expressions such as `AND` / `OR`, where PostgreSQL
    /// three-valued boolean result nullability depends on both values and
    /// presence rather than just operand presence.
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
            Self::And(_) | Self::Or(_) => None,
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
    fn nullable_first_round_evaluation_does_not_guess_and_or_nullability() {
        let alloc = Bump::new();
        let table = nullable_test_table(&alloc);
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

        let result = expression
            .first_round_evaluate_nullable_propagating(&alloc, &table, &[])
            .unwrap();

        assert!(result.is_none());
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
