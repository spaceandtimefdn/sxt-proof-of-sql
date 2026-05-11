use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{can_not_type, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable `IS NULL` or `IS NOT NULL` expression over a nullable-column validity mask.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsNullExpr {
    validity_expr: Box<DynProofExpr>,
    is_not_null: bool,
}

impl IsNullExpr {
    /// Create an `IS NULL` or `IS NOT NULL` expression from a boolean validity expression.
    pub fn try_new(validity_expr: Box<DynProofExpr>, is_not_null: bool) -> AnalyzeResult<Self> {
        let expr_type = validity_expr.data_type();
        can_not_type(expr_type)
            .then_some(Self {
                validity_expr,
                is_not_null,
            })
            .ok_or(AnalyzeError::InvalidDataType { expr_type })
    }

    /// Get the input validity expression.
    pub fn validity_expr(&self) -> &DynProofExpr {
        &self.validity_expr
    }

    /// Return true for `IS NOT NULL`, false for `IS NULL`.
    pub fn is_not_null(&self) -> bool {
        self.is_not_null
    }

    fn evaluate_validity<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        validity_column: Column<'a, S>,
    ) -> Column<'a, S> {
        let validity = validity_column
            .as_boolean()
            .expect("validity expression is not boolean");
        if self.is_not_null {
            Column::Boolean(validity)
        } else {
            Column::Boolean(alloc.alloc_slice_fill_with(validity.len(), |i| !validity[i]))
        }
    }
}

impl ProofExpr for IsNullExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "IsNullExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let validity_column = self
            .validity_expr
            .first_round_evaluate(alloc, table, params)?;
        let res = self.evaluate_validity(alloc, validity_column);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "IsNullExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let validity_column = self
            .validity_expr
            .final_round_evaluate(builder, alloc, table, params)?;
        let res = self.evaluate_validity(alloc, validity_column);

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
        let validity_eval = self
            .validity_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        if self.is_not_null {
            Ok(validity_eval)
        } else {
            Ok(chi_eval - validity_eval)
        }
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.validity_expr.get_column_references(columns);
    }
}
