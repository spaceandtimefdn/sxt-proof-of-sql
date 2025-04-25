use super::{DecimalProofExpr, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_multiply_column_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::multiply_columns,
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable numerical * expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiplyExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl MultiplyExpr {
    /// Create numerical `*` expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_multiply_column_types(left_datatype, right_datatype)
            .map(|_| Self { lhs, rhs })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
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

impl ProofExpr for MultiplyExpr {
    fn data_type(&self) -> ColumnType {
        try_multiply_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to multiply column types")
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let res = multiply_columns(&lhs_column, &rhs_column, alloc);
        Ok(Column::Decimal75(self.precision(), self.scale(), res))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.multiply_expr.final_round_evaluate",
        level = "info",
        skip_all
    )]
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

        // lhs_times_rhs
        let lhs_times_rhs: &'a [S] = multiply_columns(&lhs_column, &rhs_column, alloc);
        builder.produce_intermediate_mle(lhs_times_rhs);

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(lhs_times_rhs)]),
                (-S::one(), vec![Box::new(lhs_column), Box::new(rhs_column)]),
            ],
        );
        let res = Column::Decimal75(self.precision(), self.scale(), lhs_times_rhs);

        log::log_memory_usage("End");

        Ok(res)
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

        // lhs_times_rhs
        let lhs_times_rhs = builder.try_consume_final_round_mle_evaluation()?;

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_times_rhs - lhs * rhs,
            2,
        )?;

        // selection
        Ok(lhs_times_rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

impl DecimalProofExpr for MultiplyExpr {}
