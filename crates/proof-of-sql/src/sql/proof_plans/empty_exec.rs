use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, LiteralValue, Table, TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
    utils::log,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Source [`ProofPlan`] for (sub)queries without table source such as `SELECT "No table here" as msg;`
/// Inspired by [`DataFusion EmptyExec`](https://docs.rs/datafusion/latest/datafusion/physical_plan/empty/struct.EmptyExec.html)
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct EmptyExec {}

impl Default for EmptyExec {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyExec {
    /// Creates a new empty plan.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl ProofPlan for EmptyExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        _params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        Ok(TableEvaluation::new(
            Vec::<S>::new(),
            (builder.singleton_chi_evaluation(), 1),
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        Vec::new()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        IndexSet::default()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::default()
    }
}

impl ProverEvaluate for EmptyExec {
    #[tracing::instrument(name = "EmptyExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        // Create an empty table with one row
        let res =
            Table::<'a, S>::try_new_with_options(IndexMap::default(), TableOptions::new(Some(1)))
                .unwrap();

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "EmptyExec::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        // Create an empty table with one row
        let res =
            Table::<'a, S>::try_new_with_options(IndexMap::default(), TableOptions::new(Some(1)))
                .unwrap();

        log::log_memory_usage("End");

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::scalar::test_scalar::TestScalar,
        sql::proof::{ProofPlan, ProverEvaluate},
    };
    use alloc::collections::VecDeque;

    #[test]
    fn we_can_create_default_empty_exec_with_no_references() {
        let plan = EmptyExec::default();

        assert_eq!(plan, EmptyExec::new());
        assert!(plan.get_column_result_fields().is_empty());
        assert!(plan.get_column_references().is_empty());
        assert!(plan.get_table_references().is_empty());
    }

    #[test]
    fn empty_exec_first_round_returns_single_empty_row_without_builder_outputs() {
        let plan = EmptyExec::new();
        let alloc = Bump::new();
        let table_map = IndexMap::<TableRef, Table<'_, TestScalar>>::default();
        let mut builder = FirstRoundBuilder::<TestScalar>::new(1);

        let result = plan
            .first_round_evaluate(&mut builder, &alloc, &table_map, &[])
            .unwrap();

        assert!(result.is_empty());
        assert_eq!(result.num_columns(), 0);
        assert_eq!(result.num_rows(), 1);
        assert!(builder.pcs_proof_mles().is_empty());
        assert!(builder.chi_evaluation_lengths().is_empty());
        assert!(builder.rho_evaluation_lengths().is_empty());
    }

    #[test]
    fn empty_exec_final_round_returns_single_empty_row_without_builder_outputs() {
        let plan = EmptyExec::new();
        let alloc = Bump::new();
        let table_map = IndexMap::<TableRef, Table<'_, TestScalar>>::default();
        let mut builder = FinalRoundBuilder::<TestScalar>::new(1, VecDeque::new());

        let result = plan
            .final_round_evaluate(&mut builder, &alloc, &table_map, &[])
            .unwrap();

        assert!(result.is_empty());
        assert_eq!(result.num_columns(), 0);
        assert_eq!(result.num_rows(), 1);
        assert_eq!(builder.num_sumcheck_variables(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 0);
        assert!(builder.pcs_proof_mles().is_empty());
        assert!(builder.bit_distributions().is_empty());
        assert!(builder.sumcheck_subpolynomials().is_empty());
    }
}
