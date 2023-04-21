use super::{BoolExpr, FilterResultExpr, TableExpr};
use std::collections::HashSet;

use crate::base::database::{
    ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
};
use crate::base::math::log2_up;
use crate::sql::proof::{ProofBuilder, ProofCounts, ProofExpr, VerificationBuilder};

use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use std::cmp;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
#[derive(Debug, DynPartialEq, PartialEq)]
#[allow(dead_code)]
pub struct FilterExpr {
    results: Vec<FilterResultExpr>,
    table: TableExpr,
    where_clause: Box<dyn BoolExpr>,
}

impl FilterExpr {
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
        }
    }

    /// Returns the result expressions.
    pub fn get_results(&self) -> &[FilterResultExpr] {
        &self.results[..]
    }
}

impl ProofExpr for FilterExpr {
    #[tracing::instrument(name = "proofs.sql.ast.filter_expr.count", level = "debug", skip_all)]
    fn count(&self, counts: &mut ProofCounts, accessor: &dyn MetadataAccessor) {
        let n = accessor.get_length(self.table.table_ref);
        counts.table_length = n;
        counts.offset_generators = accessor.get_offset(self.table.table_ref);
        if n > 0 {
            counts.sumcheck_variables = cmp::max(log2_up(n), 1);
        } else {
            counts.sumcheck_variables = 0;
        }
        self.where_clause.count(counts);
        for expr in self.results.iter() {
            expr.count(counts);
        }
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.filter_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) {
        // evaluate where clause
        let selection = self
            .where_clause
            .prover_evaluate(builder, alloc, counts, accessor);

        // set result indexes
        let mut cnt: usize = 0;
        for b in selection {
            cnt += *b as usize;
        }
        let indexes = alloc.alloc_slice_fill_default::<u64>(cnt);
        cnt = 0;
        for (i, b) in selection.iter().enumerate() {
            if *b {
                indexes[cnt] = i as u64;
                cnt += 1;
            }
        }
        builder.set_result_indexes(indexes);

        // evaluate result columns
        for expr in self.results.iter() {
            expr.prover_evaluate(builder, alloc, counts, accessor, selection);
        }
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.filter_expr.verifier_evaluate",
        level = "debug",
        skip_all
    )]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) {
        let selection_eval = self
            .where_clause
            .verifier_evaluate_ark(builder, counts, accessor);
        for expr in self.results.iter() {
            expr.verifier_evaluate_ark(builder, counts, accessor, &selection_eval);
        }
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
