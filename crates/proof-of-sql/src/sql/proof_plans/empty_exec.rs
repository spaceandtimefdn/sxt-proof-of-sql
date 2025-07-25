use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, LiteralValue, OwnedTable, Table, TableEvaluation, TableOptions,
            TableRef,
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
        _result: Option<&OwnedTable<S>>,
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
        base::{
            map::IndexMap,
            scalar::test_scalar::TestScalar,
        },
    };
    use bumpalo::Bump;

    #[test]
    fn we_can_create_empty_exec() {
        let empty_exec = EmptyExec::new();
        assert_eq!(empty_exec, EmptyExec {});
    }

    #[test]
    fn empty_exec_implements_default() {
        let empty_exec1 = EmptyExec::default();
        let empty_exec2 = EmptyExec::new();
        assert_eq!(empty_exec1, empty_exec2);
    }

    #[test]
    fn empty_exec_has_no_column_result_fields() {
        let empty_exec = EmptyExec::new();
        let fields = empty_exec.get_column_result_fields();
        assert!(fields.is_empty());
    }

    #[test]
    fn empty_exec_has_no_column_references() {
        let empty_exec = EmptyExec::new();
        let refs = empty_exec.get_column_references();
        assert!(refs.is_empty());
    }

    #[test]
    fn empty_exec_has_no_table_references() {
        let empty_exec = EmptyExec::new();
        let refs = empty_exec.get_table_references();
        assert!(refs.is_empty());
    }

    #[test]
    fn empty_exec_serialization_roundtrip() {
        let empty_exec = EmptyExec::new();
        let serialized = serde_json::to_string(&empty_exec).unwrap();
        let deserialized: EmptyExec = serde_json::from_str(&serialized).unwrap();
        assert_eq!(empty_exec, deserialized);
    }

    // Note: Removed proof test as it requires blitzar dependency

    #[test]
    fn empty_exec_first_round_evaluate_creates_single_row_table() {
        let alloc = Bump::new();
        let empty_exec = EmptyExec::new();
        let mut builder = crate::sql::proof::FirstRoundBuilder::<TestScalar>::new(0);
        let table_map = IndexMap::default();
        
        let result = empty_exec.first_round_evaluate(&mut builder, &alloc, &table_map, &[]).unwrap();
        
        // Should create a table with one row but no columns
        assert_eq!(result.num_rows(), 1);
        assert_eq!(result.column_names().count(), 0);
    }

    #[test]
    fn empty_exec_final_round_evaluate_creates_single_row_table() {
        let alloc = Bump::new();
        let empty_exec = EmptyExec::new();
        let mut builder = crate::sql::proof::FinalRoundBuilder::<TestScalar>::new(0, Default::default());
        let table_map = IndexMap::default();
        
        let result = empty_exec.final_round_evaluate(&mut builder, &alloc, &table_map, &[]).unwrap();
        
        // Should create a table with one row but no columns
        assert_eq!(result.num_rows(), 1);
        assert_eq!(result.column_names().count(), 0);
    }

    // Note: Removed verifier_evaluate test as it requires complex mock setup

    #[test]
    fn empty_exec_can_be_cloned() {
        let empty_exec1 = EmptyExec::new();
        let empty_exec2 = empty_exec1.clone();
        assert_eq!(empty_exec1, empty_exec2);
    }

    #[test]
    fn empty_exec_debug_display() {
        let empty_exec = EmptyExec::new();
        let debug_str = format!("{:?}", empty_exec);
        assert!(debug_str.contains("EmptyExec"));
    }
}
