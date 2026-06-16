use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{can_and_or_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable logical AND expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AndExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl AndExpr {
    /// Create logical AND expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        can_and_or_types(left_datatype, right_datatype)
            .then_some(Self { lhs, rhs })
            .ok_or_else(|| AnalyzeError::DataTypeMismatch {
                left_type: left_datatype.to_string(),
                right_type: right_datatype.to_string(),
            })
    }

    /// Get the left-hand side expression
    pub fn lhs(&self) -> &DynProofExpr {
        &self.lhs
    }

    /// Get the right-hand side expression
    pub fn rhs(&self) -> &DynProofExpr {
        &self.rhs
    }
}

impl ProofExpr for AndExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "AndExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result =
            Column::Boolean(alloc.alloc_slice_fill_with(table.num_rows(), |i| lhs[i] && rhs[i]));

        log::log_memory_usage("End");

        Ok(result)
    }

    #[tracing::instrument(name = "AndExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let n = lhs.len();
        assert_eq!(n, rhs.len());

        // lhs_and_rhs
        let lhs_and_rhs: &[bool] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
        builder.produce_intermediate_mle(lhs_and_rhs);

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(lhs_and_rhs)]),
                (-S::one(), vec![Box::new(lhs), Box::new(rhs)]),
            ],
        );
        let result = Column::Boolean(lhs_and_rhs);

        log::log_memory_usage("End");

        Ok(result)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;

        // lhs_and_rhs
        let lhs_and_rhs = builder.try_consume_final_round_mle_evaluation()?;

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_and_rhs - lhs * rhs,
            2,
        )?;

        // selection
        Ok(lhs_and_rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::{proof_exprs::DynProofExpr, AnalyzeError},
    };

    fn bool_literal() -> Box<DynProofExpr> {
        Box::new(DynProofExpr::Literal(LiteralExpr::new(
            LiteralValue::Boolean(true),
        )))
    }

    fn bigint_literal() -> Box<DynProofExpr> {
        Box::new(DynProofExpr::Literal(LiteralExpr::new(
            LiteralValue::BigInt(0),
        )))
    }

    #[test]
    fn try_new_boolean_boolean_succeeds() {
        assert!(AndExpr::try_new(bool_literal(), bool_literal()).is_ok());
    }

    #[test]
    fn try_new_bool_bigint_fails() {
        let result = AndExpr::try_new(bool_literal(), bigint_literal());
        assert!(matches!(result, Err(AnalyzeError::DataTypeMismatch { .. })));
    }

    #[test]
    fn try_new_bigint_bigint_fails() {
        let result = AndExpr::try_new(bigint_literal(), bigint_literal());
        assert!(matches!(result, Err(AnalyzeError::DataTypeMismatch { .. })));
    }

    #[test]
    fn lhs_rhs_are_boolean() {
        let expr = AndExpr::try_new(bool_literal(), bool_literal()).unwrap();
        assert_eq!(expr.lhs().data_type(), ColumnType::Boolean);
        assert_eq!(expr.rhs().data_type(), ColumnType::Boolean);
    }

    #[test]
    fn data_type_is_boolean() {
        let expr = AndExpr::try_new(bool_literal(), bool_literal()).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn equal_exprs_compare_equal() {
        let a = AndExpr::try_new(bool_literal(), bool_literal()).unwrap();
        let b = AndExpr::try_new(bool_literal(), bool_literal()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn clone_equals_original() {
        let a = AndExpr::try_new(bool_literal(), bool_literal()).unwrap();
        assert_eq!(a.clone(), a);
    }
}
