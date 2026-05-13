use super::{
    numerical_util::{cast_column_with_scaling, try_get_scaling_factor_with_precision_and_scale},
    DynProofExpr, NullableColumnEvaluation, ProofExpr,
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
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ScalingCastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
    scaling_factor: [u64; 4],
}

impl ScalingCastExpr {
    /// Creates a new `ScalingCastExpr`
    pub fn try_new(from_expr: Box<DynProofExpr>, to_type: ColumnType) -> AnalyzeResult<Self> {
        let from_datatype = from_expr.data_type();
        try_get_scaling_factor_with_precision_and_scale(from_datatype, to_type)
            .map(|(scaling_factor, _, _)| Self {
                from_expr,
                to_type,
                scaling_factor: scaling_factor.into(),
            })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: from_datatype.to_string(),
                right_type: to_type.to_string(),
            })
    }

    /// Returns the from expression
    pub fn get_from_expr(&self) -> &DynProofExpr {
        &self.from_expr
    }

    /// Returns the to type
    pub fn to_type(&self) -> &ColumnType {
        &self.to_type
    }

    /// Returns the scaling factor
    pub fn scaling_factor(&self) -> [u64; 4] {
        self.scaling_factor
    }
}

impl ProofExpr for ScalingCastExpr {
    fn data_type(&self) -> ColumnType {
        self.to_type
    }

    fn is_nullable(&self) -> bool {
        self.from_expr.is_nullable()
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self.from_expr.first_round_evaluate(alloc, table, params)?;
        Ok(cast_column_with_scaling(
            alloc,
            uncasted_result,
            self.to_type,
        ))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self
            .from_expr
            .final_round_evaluate(builder, alloc, table, params)?;
        Ok(cast_column_with_scaling(
            alloc,
            uncasted_result,
            self.to_type,
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        self.from_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)
            .map(|unscaled_eval| S::from(self.scaling_factor) * unscaled_eval)
    }

    fn first_round_evaluate_nullable<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<NullableColumn<'a, S>> {
        let uncasted_result = self
            .from_expr
            .first_round_evaluate_nullable(alloc, table, params)?;
        let values = cast_column_with_scaling(alloc, uncasted_result.values(), self.to_type);
        Ok(NullableColumn::try_new(values, uncasted_result.presence())
            .expect("presence length should match values"))
    }

    fn final_round_evaluate_nullable<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<NullableColumn<'a, S>> {
        let uncasted_result = self
            .from_expr
            .final_round_evaluate_nullable(builder, alloc, table, params)?;
        let values = cast_column_with_scaling(alloc, uncasted_result.values(), self.to_type);
        Ok(NullableColumn::try_new(values, uncasted_result.presence())
            .expect("presence length should match values"))
    }

    fn verifier_evaluate_nullable<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<NullableColumnEvaluation<S>, ProofError> {
        let unscaled = self
            .from_expr
            .verifier_evaluate_nullable(builder, accessor, chi_eval, params)?;
        Ok(NullableColumnEvaluation::new(
            S::from(self.scaling_factor) * unscaled.value_eval(),
            unscaled.presence_eval(),
        ))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}
