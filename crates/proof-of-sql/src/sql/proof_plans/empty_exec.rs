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
    use super::EmptyExec;
    use crate::{
        base::{
            database::{Table, TableEvaluation, TableRef},
            map::IndexMap,
            scalar::test_scalar::TestScalar,
        },
        sql::proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckMleEvaluations, VerificationBuilderImpl,
        },
    };
    use alloc::vec::Vec;
    use bumpalo::Bump;
    use std::collections::VecDeque;

    fn singleton_verification_builder(
        singleton_chi: TestScalar,
    ) -> VerificationBuilderImpl<'static, TestScalar> {
        let mle_evaluations = SumcheckMleEvaluations {
            singleton_chi_evaluation: singleton_chi,
            first_round_pcs_proof_evaluations: &[],
            final_round_pcs_proof_evaluations: &[],
            ..Default::default()
        };
        VerificationBuilderImpl::new(
            mle_evaluations,
            &[],
            &[],
            VecDeque::new(),
            Vec::new(),
            Vec::new(),
            0,
        )
    }

    #[test]
    fn empty_exec_metadata_is_empty() {
        let plan = EmptyExec::default();

        assert_eq!(plan, EmptyExec::new());
        assert!(plan.get_column_result_fields().is_empty());
        assert!(plan.get_column_references().is_empty());
        assert!(plan.get_table_references().is_empty());
    }

    #[test]
    fn empty_exec_verifier_returns_singleton_empty_table_evaluation() {
        let plan = EmptyExec::new();
        let mut builder = singleton_verification_builder(TestScalar::from(7u64));
        let result = plan
            .verifier_evaluate(
                &mut builder,
                &IndexMap::default(),
                &IndexMap::default(),
                &[],
            )
            .unwrap();

        assert_eq!(
            result,
            TableEvaluation::new(Vec::new(), (TestScalar::from(7u64), 1))
        );
        assert!(result.column_evals().is_empty());
        assert_eq!(result.chi(), (TestScalar::from(7u64), 1));
    }

    #[test]
    fn empty_exec_prover_rounds_return_single_row_empty_tables() {
        let alloc = Bump::new();
        let plan = EmptyExec::new();
        let table_map: IndexMap<TableRef, Table<'_, TestScalar>> = IndexMap::default();
        let mut first_round_builder = FirstRoundBuilder::new(1);
        let mut final_round_builder = FinalRoundBuilder::new(1, VecDeque::new());

        let first_round_table = plan
            .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
            .unwrap();
        let final_round_table = plan
            .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
            .unwrap();

        assert!(first_round_table.is_empty());
        assert_eq!(first_round_table.num_rows(), 1);
        assert!(final_round_table.is_empty());
        assert_eq!(final_round_table.num_rows(), 1);
    }
}
