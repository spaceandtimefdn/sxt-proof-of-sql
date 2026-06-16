use super::DynProofPlan;
use crate::{
    base::{
        database::{
            filter_util::filter_columns, ColumnField, ColumnRef, LiteralValue, Table,
            TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_gadgets::{final_round_evaluate_filter, verify_evaluate_filter},
        proof_plans::fold_vals,
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
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        let input_table_eval =
            self.input
                .verifier_evaluate(builder, accessor, chi_eval_map, params)?;
        let (input_eval, input_length) = input_table_eval.chi();
        let (output_chi_eval, output_length) = builder.try_consume_chi_evaluation()?;
        let columns_evals = input_table_eval.column_evals();
        // 2. selection
        // The selected range is (offset_index, max_index]
        let (offset_chi_eval, offset) = builder.try_consume_chi_evaluation()?;
        let (max_chi_eval, max) = builder.try_consume_chi_evaluation()?;

        if output_length != max - offset {
            return Err(ProofError::VerificationError {
                error: "output length does not match selection length",
            });
        }
        if self.skip.min(input_length) != offset {
            return Err(ProofError::VerificationError {
                error: "offset length does not match plan value",
            });
        }
        if max
            != self
                .fetch
                .map_or(input_length, |f| (f + self.skip).min(input_length))
        {
            return Err(ProofError::VerificationError {
                error: "max length does not match expected value",
            });
        }

        let selection_eval = max_chi_eval - offset_chi_eval;
        // 3. filtered_columns
        let filtered_columns_evals =
            builder.try_consume_first_round_mle_evaluations(columns_evals.len())?;
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;

        let c_fold_eval = alpha * fold_vals(beta, columns_evals);
        let d_fold_eval = alpha * fold_vals(beta, &filtered_columns_evals);

        verify_evaluate_filter(
            builder,
            c_fold_eval,
            d_fold_eval,
            input_eval,
            output_chi_eval,
            selection_eval,
        )?;
        Ok(TableEvaluation::new(
            filtered_columns_evals,
            (output_chi_eval, output_length),
        ))
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
        let select = get_slice_select(input_length, self.skip, self.fetch);
        // The selected range is (offset_index, max_index]
        let offset_index = self.skip.min(input_length);
        let max_index = if let Some(fetch) = self.fetch {
            (self.skip + fetch).min(input_length)
        } else {
            input_length
        };
        let output_length = max_index - offset_index;
        // Compute filtered_columns
        let (filtered_columns, _) = filter_columns(alloc, &columns, &select);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.get_column_result_fields()
                .into_iter()
                .map(|expr| expr.name())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(output_length);
        builder.produce_chi_evaluation_length(offset_index);
        builder.produce_chi_evaluation_length(max_index);

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
        let select_ref: &'a [_] = alloc.alloc_slice_copy(&select);
        let output_length = select.iter().filter(|b| **b).count();
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, &select);
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        final_round_evaluate_filter::<S>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            select_ref,
            &filtered_columns,
            input.num_rows(),
            result_len,
        );
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.get_column_result_fields()
                .into_iter()
                .map(|expr| expr.name())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── get_slice_select ─────────────────────────────────────────────────────

    #[test]
    fn slice_select_skip_only() {
        // skip=2, no fetch → skip first 2, take rest
        let sel = get_slice_select(5, 2, None);
        assert_eq!(sel, vec![false, false, true, true, true]);
    }

    #[test]
    fn slice_select_fetch_only() {
        // skip=0, fetch=3 → take first 3
        let sel = get_slice_select(5, 0, Some(3));
        assert_eq!(sel, vec![true, true, true, false, false]);
    }

    #[test]
    fn slice_select_skip_and_fetch() {
        // skip=1, fetch=2 → skip 1, take 2
        let sel = get_slice_select(5, 1, Some(2));
        assert_eq!(sel, vec![false, true, true, false, false]);
    }

    #[test]
    fn slice_select_zero_rows() {
        let sel = get_slice_select(0, 1, Some(3));
        assert_eq!(sel, vec![]);
    }

    #[test]
    fn slice_select_skip_beyond_len() {
        let sel = get_slice_select(3, 10, Some(5));
        assert_eq!(sel, vec![false, false, false]);
    }

    #[test]
    fn slice_select_fetch_zero() {
        let sel = get_slice_select(5, 0, Some(0));
        assert_eq!(sel, vec![false, false, false, false, false]);
    }

    #[test]
    fn slice_select_no_skip_no_fetch_returns_all_true() {
        let sel = get_slice_select(4, 0, None);
        assert!(sel.iter().all(|&b| b));
        assert_eq!(sel.len(), 4);
    }

    // ── SliceExec accessors ──────────────────────────────────────────────────

    #[test]
    fn slice_exec_accessors() {
        use crate::sql::proof_plans::DynProofPlan;
        let input = DynProofPlan::new_empty();
        let exec = SliceExec::new(Box::new(input), 3, Some(7));
        assert_eq!(exec.skip(), 3);
        assert_eq!(exec.fetch(), Some(7));
    }

    #[test]
    fn slice_exec_no_fetch_accessor() {
        use crate::sql::proof_plans::DynProofPlan;
        let input = DynProofPlan::new_empty();
        let exec = SliceExec::new(Box::new(input), 0, None);
        assert_eq!(exec.skip(), 0);
        assert_eq!(exec.fetch(), None);
    }

    #[test]
    fn slice_exec_roundtrip_equality() {
        use crate::sql::proof_plans::DynProofPlan;
        let a = SliceExec::new(Box::new(DynProofPlan::new_empty()), 1, Some(5));
        let b = SliceExec::new(Box::new(DynProofPlan::new_empty()), 1, Some(5));
        assert_eq!(a, b);
    }

    #[test]
    fn slice_exec_different_skip_not_equal() {
        use crate::sql::proof_plans::DynProofPlan;
        let a = SliceExec::new(Box::new(DynProofPlan::new_empty()), 1, Some(5));
        let b = SliceExec::new(Box::new(DynProofPlan::new_empty()), 2, Some(5));
        assert_ne!(a, b);
    }
}
