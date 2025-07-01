use crate::{
    base::{
        database::{
            Column, ColumnField, ColumnRef, LiteralValue, OwnedTable, Table, TableEvaluation,
            TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, HonestProver, ProofPlan, ProverEvaluate,
            ProverHonestyMarker, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr, TableExpr},
        proof_gadgets::{
            final_round_evaluate_filter, first_round_evaluate_filter, verify_evaluate_filter,
        },
    },
    utils::log,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::marker::PhantomData;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
///
/// This differs from the [`FilterExec`] in that the result is not a sparse table.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OstensibleFilterExec<H: ProverHonestyMarker> {
    aliased_results: Vec<AliasedDynProofExpr>,
    table: TableExpr,
    /// TODO: add docs
    where_clause: DynProofExpr,
    phantom: PhantomData<H>,
}

impl<H: ProverHonestyMarker> OstensibleFilterExec<H> {
    /// Creates a new filter expression.
    pub fn new(
        aliased_results: Vec<AliasedDynProofExpr>,
        table: TableExpr,
        where_clause: DynProofExpr,
    ) -> Self {
        Self {
            aliased_results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }

    /// Get the aliased results
    pub fn aliased_results(&self) -> &[AliasedDynProofExpr] {
        &self.aliased_results
    }

    /// Get the table expression
    pub fn table(&self) -> &TableExpr {
        &self.table
    }

    /// Get the where clause expression
    pub fn where_clause(&self) -> &DynProofExpr {
        &self.where_clause
    }
}

impl<H: ProverHonestyMarker> ProofPlan for OstensibleFilterExec<H>
where
    OstensibleFilterExec<H>: ProverEvaluate,
{
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let input_chi_eval = *chi_eval_map
            .get(&self.table.table_ref)
            .expect("Chi eval not found");
        let accessor = accessor
            .get(&self.table.table_ref)
            .cloned()
            .unwrap_or_else(|| [].into_iter().collect());
        // 1. selection
        let selection_eval =
            self.where_clause
                .verifier_evaluate(builder, &accessor, input_chi_eval.0, params)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| {
                    aliased_expr.expr.verifier_evaluate(
                        builder,
                        &accessor,
                        input_chi_eval.0,
                        params,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        );

        verify_evaluate_filter(builder, &columns_evals, input_chi_eval.0, selection_eval)
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
            })
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();

        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref.clone()])
    }
}

/// Alias for a filter expression with a honest prover.
pub type FilterExec = OstensibleFilterExec<HonestProver>;

impl ProverEvaluate for FilterExec {
    #[tracing::instrument(name = "FilterExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self
            .where_clause
            .first_round_evaluate(alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let (columns, output_idents): (Vec<_>, Vec<_>) = self
            .aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .first_round_evaluate(alloc, table, params)
                    .map(|col| (col, aliased_expr.alias.clone()))
            })
            .collect::<PlaceholderResult<Vec<_>>>()?
            .into_iter()
            .unzip();

        // Compute filtered_columns and indexes
        let res = first_round_evaluate_filter(builder, alloc, selection, &columns, output_idents);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "FilterExec::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self
            .where_clause
            .final_round_evaluate(builder, alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let (columns, output_idents): (Vec<_>, Vec<_>) = self
            .aliased_results
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<_> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, table, params)
                    .map(|col| (col, aliased_expr.alias.clone()))
            })
            .collect::<PlaceholderResult<Vec<_>>>()?
            .into_iter()
            .unzip();

        let res = final_round_evaluate_filter(
            builder,
            alloc,
            &columns,
            output_idents,
            selection,
            table.num_rows(),
        );

        log::log_memory_usage("End");

        Ok(res)
    }
}
