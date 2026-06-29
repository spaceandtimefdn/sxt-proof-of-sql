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

#[cfg(test)]
mod tests {
    use super::EmptyExec;
    use crate::{
        base::{
            database::{Table, TableRef},
            map::IndexMap,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckMleEvaluations, VerificationBuilderImpl,
        },
    };
    use alloc::collections::VecDeque;
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_create_default_empty_exec() {
        assert_eq!(EmptyExec::default(), EmptyExec::new());
    }

    #[test]
    fn we_can_get_empty_exec_references_and_fields() {
        let empty_exec = EmptyExec::new();

        assert!(empty_exec.get_column_result_fields().is_empty());
        assert!(empty_exec.get_column_references().is_empty());
        assert!(empty_exec.get_table_references().is_empty());
    }

    #[test]
    fn we_can_evaluate_empty_exec_verifier_result() {
        let expected_chi = TestScalar::from(17_i64);
        let mle_evaluations = SumcheckMleEvaluations {
            chi_evaluations: IndexMap::default(),
            rho_evaluations: IndexMap::default(),
            singleton_chi_evaluation: expected_chi,
            random_evaluation: TestScalar::ZERO,
            first_round_pcs_proof_evaluations: &[],
            final_round_pcs_proof_evaluations: &[],
            rho_256_evaluation: None,
        };
        let mut builder = VerificationBuilderImpl::new(
            mle_evaluations,
            &[],
            &[],
            VecDeque::new(),
            Vec::new(),
            Vec::new(),
            0,
        );
        let accessor: IndexMap<TableRef, IndexMap<Ident, TestScalar>> = IndexMap::default();
        let chi_eval_map: IndexMap<TableRef, (TestScalar, usize)> = IndexMap::default();

        let table_evaluation = EmptyExec::new()
            .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
            .unwrap();

        assert!(table_evaluation.column_evals().is_empty());
        assert_eq!(table_evaluation.chi(), (expected_chi, 1));
    }

    #[test]
    fn we_can_evaluate_empty_exec_prover_rounds() {
        let empty_exec = EmptyExec::new();
        let alloc = Bump::new();
        let table_map: IndexMap<TableRef, Table<'_, TestScalar>> = IndexMap::default();
        let mut first_round_builder = FirstRoundBuilder::new(1);
        let mut final_round_builder = FinalRoundBuilder::new(1, VecDeque::new());

        let first_round_table = empty_exec
            .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
            .unwrap();
        assert!(first_round_table.is_empty());
        assert_eq!(first_round_table.num_rows(), 1);

        let final_round_table = empty_exec
            .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
            .unwrap();
        assert!(final_round_table.is_empty());
        assert_eq!(final_round_table.num_rows(), 1);
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
