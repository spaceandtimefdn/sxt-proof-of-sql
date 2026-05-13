use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable SQL `IS NULL` expression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsNullExpr {
    expr: Box<DynProofExpr>,
}

impl IsNullExpr {
    /// Create a SQL `IS NULL` expression.
    #[must_use]
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        Self { expr }
    }
}

impl ProofExpr for IsNullExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input = self
            .expr
            .first_round_evaluate_nullable(alloc, table, params)?;
        Ok(Column::Boolean(match input.presence() {
            None => alloc.alloc_slice_fill_copy(input.len(), false),
            Some(presence) => alloc.alloc_slice_fill_with(input.len(), |i| !presence[i]),
        }))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input = self
            .expr
            .final_round_evaluate_nullable(builder, alloc, table, params)?;
        Ok(Column::Boolean(match input.presence() {
            None => alloc.alloc_slice_fill_copy(input.len(), false),
            Some(presence) => alloc.alloc_slice_fill_with(input.len(), |i| !presence[i]),
        }))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let input = self
            .expr
            .verifier_evaluate_nullable(builder, accessor, chi_eval, params)?;
        Ok(input
            .presence_eval()
            .map_or_else(S::zero, |presence| chi_eval - presence))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}

/// Provable SQL `IS NOT NULL` expression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsNotNullExpr {
    expr: Box<DynProofExpr>,
}

impl IsNotNullExpr {
    /// Create a SQL `IS NOT NULL` expression.
    #[must_use]
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        Self { expr }
    }
}

impl ProofExpr for IsNotNullExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input = self
            .expr
            .first_round_evaluate_nullable(alloc, table, params)?;
        Ok(Column::Boolean(match input.presence() {
            None => alloc.alloc_slice_fill_copy(input.len(), true),
            Some(presence) => presence,
        }))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input = self
            .expr
            .final_round_evaluate_nullable(builder, alloc, table, params)?;
        Ok(Column::Boolean(match input.presence() {
            None => alloc.alloc_slice_fill_copy(input.len(), true),
            Some(presence) => presence,
        }))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let input = self
            .expr
            .verifier_evaluate_nullable(builder, accessor, chi_eval, params)?;
        Ok(input.presence_eval().unwrap_or(chi_eval))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}

/// Provable SQL `IS TRUE` expression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IsTrueExpr {
    expr: Box<DynProofExpr>,
}

impl IsTrueExpr {
    /// Create a SQL `IS TRUE` expression.
    #[must_use]
    pub fn new(expr: Box<DynProofExpr>) -> Self {
        Self { expr }
    }
}

impl ProofExpr for IsTrueExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input = self
            .expr
            .first_round_evaluate_nullable(alloc, table, params)?;
        Ok(Column::Boolean(
            super::first_round_evaluate_nullable_boolean_is_true(alloc, input),
        ))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input = self
            .expr
            .final_round_evaluate_nullable(builder, alloc, table, params)?;
        Ok(Column::Boolean(
            super::final_round_evaluate_nullable_boolean_is_true(builder, alloc, input),
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let input = self
            .expr
            .verifier_evaluate_nullable(builder, accessor, chi_eval, params)?;
        super::verifier_evaluate_nullable_boolean_is_true(
            builder,
            input.value_eval(),
            input.presence_eval(),
        )
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}
