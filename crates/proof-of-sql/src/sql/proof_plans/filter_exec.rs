use super::fold_vals;
use crate::{
    base::{
        database::{
            filter_util::filter_columns, Column, ColumnField, ColumnRef, LiteralValue, OwnedTable,
            Table, TableEvaluation, TableOptions, TableRef,
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
        proof_gadgets::{final_round_evaluate_filter, verify_evaluate_filter},
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

        let output_chi_eval = builder.try_consume_chi_evaluation()?;
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
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
        // 3. filtered_columns
        let filtered_columns_evals =
            builder.try_consume_first_round_mle_evaluations(self.aliased_results.len())?;
        assert!(filtered_columns_evals.len() == self.aliased_results.len());

        let c_fold_eval = alpha * fold_vals(beta, &columns_evals);
        let d_fold_eval = alpha * fold_vals(beta, &filtered_columns_evals);

        verify_evaluate_filter(
            builder,
            c_fold_eval,
            d_fold_eval,
            input_chi_eval.0,
            output_chi_eval.0,
            selection_eval,
        )?;
        Ok(TableEvaluation::new(
            filtered_columns_evals,
            output_chi_eval,
        ))
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
        let output_length = selection.iter().filter(|b| **b).count();
        builder.produce_chi_evaluation_length(output_length);
        builder.request_post_result_challenges(2);

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr.expr.first_round_evaluate(alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;

        // Compute filtered_columns and indexes
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias.clone())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");

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
        let output_length = selection.iter().filter(|b| **b).count();
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // Compute filtered_columns
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);

        final_round_evaluate_filter::<S>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            selection,
            &filtered_columns,
            table.num_rows(),
            result_len,
        );
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias.clone())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        Ok(res)
    }
}
