use super::{BoolExpr, FilterResultExpr, TableExpr};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor},
        proof::ProofError,
    },
    sql::proof::{
        CountBuilder, HonestProver, Indexes, ProofBuilder, ProofExpr, ProverEvaluate,
        ProverHonestyMarker, VerificationBuilder,
    },
};
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, marker::PhantomData};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OstensibleFilterExpr<H: ProverHonestyMarker> {
    pub(super) results: Vec<FilterResultExpr>,
    pub(super) table: TableExpr,
    pub(super) where_clause: Box<dyn BoolExpr>,
    phantom: PhantomData<H>,
}

// This is required because derive(DynPartialEq) does not work with generics
impl<H: ProverHonestyMarker> DynPartialEq for OstensibleFilterExpr<H> {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn box_eq(&self, other: &dyn core::any::Any) -> bool {
        other.downcast_ref::<Self>().map_or(false, |a| self == a)
    }
}

impl<H: ProverHonestyMarker> OstensibleFilterExpr<H> {
    /// Creates a new filter expression.
    pub fn new(
        results: Vec<FilterResultExpr>,
        table: TableExpr,
        where_clause: Box<dyn BoolExpr>,
    ) -> Self {
        Self {
            results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }

    /// Returns the result expressions.
    pub fn get_results(&self) -> &[FilterResultExpr] {
        &self.results[..]
    }
}

impl<H: ProverHonestyMarker> ProofExpr for OstensibleFilterExpr<H>
where
    OstensibleFilterExpr<H>: ProverEvaluate,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        self.where_clause.count(builder)?;
        for expr in self.results.iter() {
            expr.count(builder);
        }
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table.table_ref)
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table.table_ref)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.filter_expr.verifier_evaluate",
        level = "debug",
        skip_all
    )]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<(), ProofError> {
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        for expr in self.results.iter() {
            expr.verifier_evaluate(builder, accessor, &selection_eval);
        }
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let mut columns = Vec::with_capacity(self.results.len());
        for col in self.results.iter() {
            columns.push(col.get_column_field());
        }
        columns
    }

    fn get_column_references(&self) -> HashSet<ColumnRef> {
        let mut columns = HashSet::new();

        for col in self.results.iter() {
            columns.insert(col.get_column_reference());
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }
}

pub type FilterExpr = OstensibleFilterExpr<HonestProver>;
impl ProverEvaluate for FilterExpr {
    #[tracing::instrument(
        name = "proofs.sql.ast.filter_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) {
        // evaluate where clause
        let selection = self.where_clause.prover_evaluate(builder, alloc, accessor);

        // set result indexes
        let indexes = selection
            .iter()
            .enumerate()
            .filter(|(_, &b)| b)
            .map(|(i, _)| i as u64)
            .collect();
        builder.set_result_indexes(Indexes::Sparse(indexes));

        // evaluate result columns
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, accessor, selection);
        }
    }
}
