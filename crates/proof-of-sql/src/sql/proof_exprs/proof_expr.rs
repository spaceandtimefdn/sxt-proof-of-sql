use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, NullableColumn, Table},
        map::{IndexMap, IndexSet},
        math::decimal::Precision,
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use core::fmt::Debug;
use sqlparser::ast::Ident;

/// Evaluations of a nullable expression at a verifier point.
///
/// `presence_eval == None` means the expression is non-nullable. `Some(eval)` is the MLE
/// evaluation of the boolean presence column where `true` means the row is present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullableColumnEvaluation<S: Scalar> {
    value_eval: S,
    presence_eval: Option<S>,
}

impl<S: Scalar> NullableColumnEvaluation<S> {
    /// Create a non-nullable evaluation.
    #[must_use]
    pub fn non_nullable(value_eval: S) -> Self {
        Self {
            value_eval,
            presence_eval: None,
        }
    }

    /// Create an evaluation with optional presence.
    #[must_use]
    pub fn new(value_eval: S, presence_eval: Option<S>) -> Self {
        Self {
            value_eval,
            presence_eval,
        }
    }

    /// Return the value-column evaluation.
    #[must_use]
    pub fn value_eval(&self) -> S {
        self.value_eval
    }

    /// Return the optional presence-column evaluation.
    #[must_use]
    pub fn presence_eval(&self) -> Option<S> {
        self.presence_eval
    }
}

/// Provable AST column expression that evaluates to a `Column`
#[enum_dispatch::enum_dispatch(DynProofExpr)]
pub trait ProofExpr: Debug + Send + Sync {
    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType;

    /// This returns the result of evaluating the expression on the given table, and returns
    /// a column of values. This result slice is guaranteed to have length `table_length`.
    /// Implementations must ensure that the returned slice has length `table_length`.
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>>;

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of values
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>>;

    /// Compute the evaluation of a multilinear extension from this expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// [`VerificationBuilder<S>`]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError>;

    /// Return whether this expression can evaluate to SQL NULL.
    fn is_nullable(&self) -> bool {
        false
    }

    /// Evaluate the expression and carry optional row-presence metadata.
    fn first_round_evaluate_nullable<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<NullableColumn<'a, S>> {
        Ok(NullableColumn::non_nullable(
            self.first_round_evaluate(alloc, table, params)?,
        ))
    }

    /// Evaluate the expression, add proof components, and carry optional row-presence metadata.
    fn final_round_evaluate_nullable<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<NullableColumn<'a, S>> {
        Ok(NullableColumn::non_nullable(
            self.final_round_evaluate(builder, alloc, table, params)?,
        ))
    }

    /// Verify the expression's value and optional presence evaluation.
    fn verifier_evaluate_nullable<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<NullableColumnEvaluation<S>, ProofError> {
        Ok(NullableColumnEvaluation::non_nullable(
            self.verifier_evaluate(builder, accessor, chi_eval, params)?,
        ))
    }

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>);
}

/// Evaluate a boolean AND without producing proof data.
pub(crate) fn first_round_evaluate_boolean_and<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    assert_eq!(table_length, rhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] && rhs[i])
}

/// Evaluate and prove a boolean AND.
pub(crate) fn final_round_evaluate_boolean_and<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    let n = lhs.len();
    assert_eq!(n, rhs.len());

    let lhs_and_rhs: &'a [bool] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
    builder.produce_intermediate_mle(lhs_and_rhs);
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(lhs_and_rhs)]),
            (-S::one(), vec![Box::new(lhs), Box::new(rhs)]),
        ],
    );
    lhs_and_rhs
}

/// Verify a boolean AND.
pub(crate) fn verifier_evaluate_boolean_and<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs: S,
    rhs: S,
) -> Result<S, ProofError> {
    let lhs_and_rhs = builder.try_consume_final_round_mle_evaluation()?;
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        lhs_and_rhs - lhs * rhs,
        2,
    )?;
    Ok(lhs_and_rhs)
}

/// Evaluate a boolean OR without producing proof data.
pub(crate) fn first_round_evaluate_boolean_or<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    assert_eq!(table_length, rhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] || rhs[i])
}

/// Evaluate and prove a boolean OR.
pub(crate) fn final_round_evaluate_boolean_or<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    let n = lhs.len();
    assert_eq!(n, rhs.len());

    let _ = final_round_evaluate_boolean_and(builder, alloc, lhs, rhs);
    alloc.alloc_slice_fill_with(n, |i| lhs[i] || rhs[i])
}

/// Verify a boolean OR.
pub(crate) fn verifier_evaluate_boolean_or<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs: S,
    rhs: S,
) -> Result<S, ProofError> {
    let lhs_and_rhs = verifier_evaluate_boolean_and(builder, lhs, rhs)?;
    Ok(lhs + rhs - lhs_and_rhs)
}

fn first_round_evaluate_boolean_not<'a>(
    table_length: usize,
    alloc: &'a Bump,
    values: &'a [bool],
) -> &'a [bool] {
    assert_eq!(table_length, values.len());
    alloc.alloc_slice_fill_with(table_length, |i| !values[i])
}

/// Combine optional presence masks using SQL binary-expression nullability rules.
#[must_use]
pub(crate) fn first_round_evaluate_nullable_presence<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: Option<&'a [bool]>,
    rhs: Option<&'a [bool]>,
) -> Option<&'a [bool]> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (Some(lhs), Some(rhs)) => Some(first_round_evaluate_boolean_and(
            table_length,
            alloc,
            lhs,
            rhs,
        )),
    }
}

/// Combine optional presence masks and prove the conjunction when both sides are nullable.
pub(crate) fn final_round_evaluate_nullable_presence<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: Option<&'a [bool]>,
    rhs: Option<&'a [bool]>,
) -> Option<&'a [bool]> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (Some(lhs), Some(rhs)) => Some(final_round_evaluate_boolean_and(builder, alloc, lhs, rhs)),
    }
}

/// Verify optional presence-mask conjunction.
pub(crate) fn verifier_evaluate_nullable_presence<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs: Option<S>,
    rhs: Option<S>,
) -> Result<Option<S>, ProofError> {
    match (lhs, rhs) {
        (None, None) => Ok(None),
        (Some(lhs), None) => Ok(Some(lhs)),
        (None, Some(rhs)) => Ok(Some(rhs)),
        (Some(lhs), Some(rhs)) => Ok(Some(verifier_evaluate_boolean_and(builder, lhs, rhs)?)),
    }
}

/// Compute SQL three-valued `AND` result presence for nullable boolean operands.
#[must_use]
pub(crate) fn first_round_evaluate_nullable_boolean_and_presence<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs_values: &'a [bool],
    lhs_presence: Option<&'a [bool]>,
    rhs_values: &'a [bool],
    rhs_presence: Option<&'a [bool]>,
) -> Option<&'a [bool]> {
    match (lhs_presence, rhs_presence) {
        (None, None) => None,
        (Some(lhs_presence), None) => {
            let rhs_false = first_round_evaluate_boolean_not(table_length, alloc, rhs_values);
            Some(first_round_evaluate_boolean_or(
                table_length,
                alloc,
                lhs_presence,
                rhs_false,
            ))
        }
        (None, Some(rhs_presence)) => {
            let lhs_false = first_round_evaluate_boolean_not(table_length, alloc, lhs_values);
            Some(first_round_evaluate_boolean_or(
                table_length,
                alloc,
                rhs_presence,
                lhs_false,
            ))
        }
        (Some(lhs_presence), Some(rhs_presence)) => {
            let both_present =
                first_round_evaluate_boolean_and(table_length, alloc, lhs_presence, rhs_presence);
            let lhs_false = first_round_evaluate_boolean_not(table_length, alloc, lhs_values);
            let lhs_false_present =
                first_round_evaluate_boolean_and(table_length, alloc, lhs_presence, lhs_false);
            let rhs_false = first_round_evaluate_boolean_not(table_length, alloc, rhs_values);
            let rhs_false_present =
                first_round_evaluate_boolean_and(table_length, alloc, rhs_presence, rhs_false);
            let left = first_round_evaluate_boolean_or(
                table_length,
                alloc,
                both_present,
                lhs_false_present,
            );
            Some(first_round_evaluate_boolean_or(
                table_length,
                alloc,
                left,
                rhs_false_present,
            ))
        }
    }
}

/// Compute SQL three-valued `OR` result presence for nullable boolean operands.
#[must_use]
pub(crate) fn first_round_evaluate_nullable_boolean_or_presence<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs_values: &'a [bool],
    lhs_presence: Option<&'a [bool]>,
    rhs_values: &'a [bool],
    rhs_presence: Option<&'a [bool]>,
) -> Option<&'a [bool]> {
    match (lhs_presence, rhs_presence) {
        (None, None) => None,
        (Some(lhs_presence), None) => Some(first_round_evaluate_boolean_or(
            table_length,
            alloc,
            lhs_presence,
            rhs_values,
        )),
        (None, Some(rhs_presence)) => Some(first_round_evaluate_boolean_or(
            table_length,
            alloc,
            rhs_presence,
            lhs_values,
        )),
        (Some(lhs_presence), Some(rhs_presence)) => {
            let both_present =
                first_round_evaluate_boolean_and(table_length, alloc, lhs_presence, rhs_presence);
            let lhs_true_present =
                first_round_evaluate_boolean_and(table_length, alloc, lhs_presence, lhs_values);
            let rhs_true_present =
                first_round_evaluate_boolean_and(table_length, alloc, rhs_presence, rhs_values);
            let left = first_round_evaluate_boolean_or(
                table_length,
                alloc,
                both_present,
                lhs_true_present,
            );
            Some(first_round_evaluate_boolean_or(
                table_length,
                alloc,
                left,
                rhs_true_present,
            ))
        }
    }
}

/// Compute and prove SQL three-valued `AND` result presence for nullable boolean operands.
pub(crate) fn final_round_evaluate_nullable_boolean_and_presence<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs_values: &'a [bool],
    lhs_presence: Option<&'a [bool]>,
    rhs_values: &'a [bool],
    rhs_presence: Option<&'a [bool]>,
) -> Option<&'a [bool]> {
    match (lhs_presence, rhs_presence) {
        (None, None) => None,
        (Some(lhs_presence), None) => {
            let rhs_false = first_round_evaluate_boolean_not(lhs_presence.len(), alloc, rhs_values);
            Some(final_round_evaluate_boolean_or(
                builder,
                alloc,
                lhs_presence,
                rhs_false,
            ))
        }
        (None, Some(rhs_presence)) => {
            let lhs_false = first_round_evaluate_boolean_not(rhs_presence.len(), alloc, lhs_values);
            Some(final_round_evaluate_boolean_or(
                builder,
                alloc,
                rhs_presence,
                lhs_false,
            ))
        }
        (Some(lhs_presence), Some(rhs_presence)) => {
            let both_present =
                final_round_evaluate_boolean_and(builder, alloc, lhs_presence, rhs_presence);
            let lhs_false = first_round_evaluate_boolean_not(lhs_presence.len(), alloc, lhs_values);
            let lhs_false_present =
                final_round_evaluate_boolean_and(builder, alloc, lhs_presence, lhs_false);
            let rhs_false = first_round_evaluate_boolean_not(rhs_presence.len(), alloc, rhs_values);
            let rhs_false_present =
                final_round_evaluate_boolean_and(builder, alloc, rhs_presence, rhs_false);
            let left =
                final_round_evaluate_boolean_or(builder, alloc, both_present, lhs_false_present);
            Some(final_round_evaluate_boolean_or(
                builder,
                alloc,
                left,
                rhs_false_present,
            ))
        }
    }
}

/// Compute and prove SQL three-valued `OR` result presence for nullable boolean operands.
pub(crate) fn final_round_evaluate_nullable_boolean_or_presence<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs_values: &'a [bool],
    lhs_presence: Option<&'a [bool]>,
    rhs_values: &'a [bool],
    rhs_presence: Option<&'a [bool]>,
) -> Option<&'a [bool]> {
    match (lhs_presence, rhs_presence) {
        (None, None) => None,
        (Some(lhs_presence), None) => Some(final_round_evaluate_boolean_or(
            builder,
            alloc,
            lhs_presence,
            rhs_values,
        )),
        (None, Some(rhs_presence)) => Some(final_round_evaluate_boolean_or(
            builder,
            alloc,
            rhs_presence,
            lhs_values,
        )),
        (Some(lhs_presence), Some(rhs_presence)) => {
            let both_present =
                final_round_evaluate_boolean_and(builder, alloc, lhs_presence, rhs_presence);
            let lhs_true_present =
                final_round_evaluate_boolean_and(builder, alloc, lhs_presence, lhs_values);
            let rhs_true_present =
                final_round_evaluate_boolean_and(builder, alloc, rhs_presence, rhs_values);
            let left =
                final_round_evaluate_boolean_or(builder, alloc, both_present, lhs_true_present);
            Some(final_round_evaluate_boolean_or(
                builder,
                alloc,
                left,
                rhs_true_present,
            ))
        }
    }
}

/// Verify SQL three-valued `AND` result presence for nullable boolean operands.
pub(crate) fn verifier_evaluate_nullable_boolean_and_presence<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    chi_eval: S,
    lhs_value_eval: S,
    lhs_presence_eval: Option<S>,
    rhs_value_eval: S,
    rhs_presence_eval: Option<S>,
) -> Result<Option<S>, ProofError> {
    match (lhs_presence_eval, rhs_presence_eval) {
        (None, None) => Ok(None),
        (Some(lhs_presence_eval), None) => Ok(Some(verifier_evaluate_boolean_or(
            builder,
            lhs_presence_eval,
            chi_eval - rhs_value_eval,
        )?)),
        (None, Some(rhs_presence_eval)) => Ok(Some(verifier_evaluate_boolean_or(
            builder,
            rhs_presence_eval,
            chi_eval - lhs_value_eval,
        )?)),
        (Some(lhs_presence_eval), Some(rhs_presence_eval)) => {
            let both_present =
                verifier_evaluate_boolean_and(builder, lhs_presence_eval, rhs_presence_eval)?;
            let lhs_false_present = verifier_evaluate_boolean_and(
                builder,
                lhs_presence_eval,
                chi_eval - lhs_value_eval,
            )?;
            let rhs_false_present = verifier_evaluate_boolean_and(
                builder,
                rhs_presence_eval,
                chi_eval - rhs_value_eval,
            )?;
            let left = verifier_evaluate_boolean_or(builder, both_present, lhs_false_present)?;
            Ok(Some(verifier_evaluate_boolean_or(
                builder,
                left,
                rhs_false_present,
            )?))
        }
    }
}

/// Verify SQL three-valued `OR` result presence for nullable boolean operands.
pub(crate) fn verifier_evaluate_nullable_boolean_or_presence<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs_value_eval: S,
    lhs_presence_eval: Option<S>,
    rhs_value_eval: S,
    rhs_presence_eval: Option<S>,
) -> Result<Option<S>, ProofError> {
    match (lhs_presence_eval, rhs_presence_eval) {
        (None, None) => Ok(None),
        (Some(lhs_presence_eval), None) => Ok(Some(verifier_evaluate_boolean_or(
            builder,
            lhs_presence_eval,
            rhs_value_eval,
        )?)),
        (None, Some(rhs_presence_eval)) => Ok(Some(verifier_evaluate_boolean_or(
            builder,
            rhs_presence_eval,
            lhs_value_eval,
        )?)),
        (Some(lhs_presence_eval), Some(rhs_presence_eval)) => {
            let both_present =
                verifier_evaluate_boolean_and(builder, lhs_presence_eval, rhs_presence_eval)?;
            let lhs_true_present =
                verifier_evaluate_boolean_and(builder, lhs_presence_eval, lhs_value_eval)?;
            let rhs_true_present =
                verifier_evaluate_boolean_and(builder, rhs_presence_eval, rhs_value_eval)?;
            let left = verifier_evaluate_boolean_or(builder, both_present, lhs_true_present)?;
            Ok(Some(verifier_evaluate_boolean_or(
                builder,
                left,
                rhs_true_present,
            )?))
        }
    }
}

/// Evaluate a nullable boolean as SQL `IS TRUE`, which is what `WHERE` clauses select.
pub(crate) fn first_round_evaluate_nullable_boolean_is_true<'a, S: Scalar>(
    alloc: &'a Bump,
    column: NullableColumn<'a, S>,
) -> &'a [bool] {
    let values = column
        .values()
        .as_boolean()
        .expect("expression is not boolean");
    match column.presence() {
        None => values,
        Some(presence) => first_round_evaluate_boolean_and(column.len(), alloc, values, presence),
    }
}

/// Evaluate and prove nullable boolean `IS TRUE`.
pub(crate) fn final_round_evaluate_nullable_boolean_is_true<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    column: NullableColumn<'a, S>,
) -> &'a [bool] {
    let values = column
        .values()
        .as_boolean()
        .expect("expression is not boolean");
    match column.presence() {
        None => values,
        Some(presence) => final_round_evaluate_boolean_and(builder, alloc, values, presence),
    }
}

/// Verify nullable boolean `IS TRUE`.
pub(crate) fn verifier_evaluate_nullable_boolean_is_true<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    value_eval: S,
    presence_eval: Option<S>,
) -> Result<S, ProofError> {
    match presence_eval {
        None => Ok(value_eval),
        Some(presence_eval) => verifier_evaluate_boolean_and(builder, value_eval, presence_eval),
    }
}

/// A trait for `ProofExpr`s that always return a decimal type
pub(crate) trait DecimalProofExpr: ProofExpr {
    /// Get the precision of the expression
    ///
    /// # Panics
    /// This panics if the precision is invalid
    fn precision(&self) -> Precision {
        Precision::new(
            self.data_type()
                .precision_value()
                .expect("Precision should be valid"),
        )
        .expect("Precision should be valid")
    }

    /// Get the scale of the expression
    ///
    /// # Panics
    /// This panics if the scale is invalid
    fn scale(&self) -> i8 {
        self.data_type().scale().expect("Scale should be valid")
    }
}
