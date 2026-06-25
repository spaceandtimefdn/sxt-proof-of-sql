use super::{
    numerical_util::{cast_column_with_scaling, try_get_scaling_factor_with_precision_and_scale},
    DynProofExpr, ProofExpr,
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

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests_scaling_cast {
    use crate::{
        base::{database::{ColumnType, LiteralValue}, math::decimal::Precision},
        sql::proof_exprs::{DynProofExpr, ProofExpr, ScalingCastExpr},
    };

    fn bigint_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(5))
    }

    fn decimal75_type() -> ColumnType {
        // BigInt has precision=19, scale=0; Decimal75(75, 2) has 75-2=73 >= 19-0=19
        ColumnType::Decimal75(Precision::new(75).unwrap(), 2)
    }

    #[test]
    fn try_new_valid_cast_returns_ok() {
        assert!(ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).is_ok());
    }

    #[test]
    fn try_new_invalid_cast_returns_err() {
        // Boolean → Decimal75 is not a valid scale cast
        let bool_expr = DynProofExpr::new_literal(LiteralValue::Boolean(true));
        assert!(ScalingCastExpr::try_new(alloc::boxed::Box::new(bool_expr), decimal75_type()).is_err());
    }

    #[test]
    fn data_type_returns_to_type() {
        let e = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        assert_eq!(e.data_type(), decimal75_type());
    }

    #[test]
    fn to_type_accessor() {
        let e = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        assert_eq!(*e.to_type(), decimal75_type());
    }

    #[test]
    fn from_expr_has_bigint_type() {
        let e = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        assert_eq!(e.get_from_expr().data_type(), ColumnType::BigInt);
    }

    #[test]
    fn scaling_factor_is_100_for_scale_2() {
        // 10^(2-0) = 100 as a U256 → [u64; 4] = [100, 0, 0, 0]
        let e = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        assert_eq!(e.scaling_factor()[0], 100);
    }

    #[test]
    fn equality_holds() {
        let a = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        let b = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e = ScalingCastExpr::try_new(alloc::boxed::Box::new(bigint_expr()), decimal75_type()).unwrap();
        assert!(alloc::format!("{e:?}").contains("ScalingCastExpr"));
    }
}
