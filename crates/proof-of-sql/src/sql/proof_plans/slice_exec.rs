use super::DynProofPlan;
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, LiteralValue, OwnedTable, Table, TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_gadgets::{
            final_round_evaluate_filter, first_round_evaluate_filter, verify_evaluate_filter,
        },
    },
    utils::log,
};
use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use core::iter::repeat;
use itertools::repeat_n;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> LIMIT <fetch> [OFFSET <skip>]
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SliceExec {
    pub(super) input: Box<DynProofPlan>,
    pub(super) skip: usize,
    pub(super) fetch: Option<usize>,
}

/// Get the boolean slice selection from the number of rows, skip and fetch
fn get_slice_select(num_rows: usize, skip: usize, fetch: Option<usize>) -> Vec<bool> {
    repeat_n(false, skip)
        .chain(repeat_n(true, fetch.unwrap_or(num_rows)))
        .chain(repeat(false))
        .take(num_rows)
        .collect()
}

impl SliceExec {
    /// Creates a new slice execution plan.
    pub fn new(input: Box<DynProofPlan>, skip: usize, fetch: Option<usize>) -> Self {
        Self { input, skip, fetch }
    }

    /// Get a reference to the input plan
    pub fn input(&self) -> &DynProofPlan {
        &self.input
    }

    /// Get the skip value
    pub fn skip(&self) -> usize {
        self.skip
    }

    /// Get the fetch value
    pub fn fetch(&self) -> Option<usize> {
        self.fetch
    }
}

impl ProofPlan for SliceExec
where
    SliceExec: ProverEvaluate,
{
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        let input_table_eval =
            self.input
                .verifier_evaluate(builder, accessor, None, chi_eval_map, params)?;
        let columns_evals = input_table_eval.column_evals();
        // 2. selection
        // The selected range is (offset_index, max_index]
        let offset_chi_eval = builder.try_consume_chi_evaluation()?.0;
        let max_chi_eval = builder.try_consume_chi_evaluation()?.0;
        let selection_eval = max_chi_eval - offset_chi_eval;

        verify_evaluate_filter(
            builder,
            columns_evals,
            input_table_eval.chi_eval(),
            selection_eval,
        )
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.input.get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.input.get_column_references()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for SliceExec {
    #[tracing::instrument(name = "SliceExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        // 1. columns
        let input = self
            .input
            .first_round_evaluate(builder, alloc, table_map, params)?;
        let input_length = input.num_rows();
        let columns = input.columns().copied().collect::<Vec<_>>();
        // 2. select
        let selection =
            alloc.alloc_slice_copy(&get_slice_select(input_length, self.skip, self.fetch));
        // The selected range is (offset_index, max_index]
        let offset_index = self.skip.min(input_length);
        let max_index = if let Some(fetch) = self.fetch {
            (self.skip + fetch).min(input_length)
        } else {
            input_length
        };
        let output_idents = self
            .get_column_result_fields()
            .into_iter()
            .map(|expr| expr.name())
            .collect();
        builder.produce_chi_evaluation_length(offset_index);
        builder.produce_chi_evaluation_length(max_index);

        let res = first_round_evaluate_filter(builder, alloc, selection, &columns, output_idents);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "SliceExec::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        // 1. columns
        let input = self
            .input
            .final_round_evaluate(builder, alloc, table_map, params)?;
        let columns = input.columns().copied().collect::<Vec<_>>();
        // 2. select
        let select = get_slice_select(input.num_rows(), self.skip, self.fetch);
        let selection: &'a [_] = alloc.alloc_slice_copy(&select);
        let output_idents = self
            .get_column_result_fields()
            .into_iter()
            .map(|expr| expr.name())
            .collect();

        let res = final_round_evaluate_filter(
            builder,
            alloc,
            &columns,
            output_idents,
            selection,
            input.num_rows(),
        );

        log::log_memory_usage("End");

        Ok(res)
    }
}
