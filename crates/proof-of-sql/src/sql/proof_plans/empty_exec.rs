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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::collections::VecDeque;

    #[test]
    fn default_matches_new_empty_exec_plan() {
        assert_eq!(EmptyExec::default(), EmptyExec::new());
    }

    #[test]
    fn reports_no_columns_or_table_references() {
        let plan = EmptyExec::new();

        assert!(plan.get_column_result_fields().is_empty());
        assert!(plan.get_column_references().is_empty());
        assert!(plan.get_table_references().is_empty());
    }

    #[test]
    fn first_round_evaluate_returns_one_row_empty_table() {
        let plan = EmptyExec::new();
        let alloc = Bump::new();
        let table_map = IndexMap::default();
        let mut builder = FirstRoundBuilder::new(0);

        let table = plan
            .first_round_evaluate::<TestScalar>(&mut builder, &alloc, &table_map, &[])
            .expect("empty exec first round should succeed");

        assert_eq!(table.num_rows(), 1);
        assert_eq!(table.columns().count(), 0);
    }

    #[test]
    fn final_round_evaluate_returns_one_row_empty_table() {
        let plan = EmptyExec::new();
        let alloc = Bump::new();
        let table_map = IndexMap::default();
        let mut builder = FinalRoundBuilder::new(0, VecDeque::new());

        let table = plan
            .final_round_evaluate::<TestScalar>(&mut builder, &alloc, &table_map, &[])
            .expect("empty exec final round should succeed");

        assert_eq!(table.num_rows(), 1);
        assert_eq!(table.columns().count(), 0);
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
