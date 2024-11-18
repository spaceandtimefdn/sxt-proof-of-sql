use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableOptions, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ProofExpr, TableExpr},
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::iter::repeat_with;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionExec {
    pub(super) aliased_results: Vec<AliasedDynProofExpr>,
    pub(super) table: TableExpr,
}

impl ProjectionExec {
    /// Creates a new projection expression.
    pub fn new(aliased_results: Vec<AliasedDynProofExpr>, table: TableExpr) -> Self {
        Self {
            aliased_results,
            table,
        }
    }
}

impl ProofPlan for ProjectionExec {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.count(builder)?;
            builder.count_intermediate_mles(1);
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<IndexMap<ColumnRef, S>, ProofError> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
            .collect::<Result<Vec<_>, _>>()?;
        let evals_map: IndexMap<ColumnRef, S> = self
            .get_column_result_fields()
            .into_iter()
            .map(|field| ColumnRef::new(self.table.table_ref, field.name(), field.data_type()))
            .zip(
                repeat_with(|| builder.consume_intermediate_mle()).take(self.aliased_results.len()),
            )
            .collect();
        Ok(evals_map)
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();
        self.aliased_results.iter().for_each(|aliased_expr| {
            aliased_expr.expr.get_column_references(&mut columns);
        });
        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref])
    }
}

impl ProverEvaluate for ProjectionExec {
    #[tracing::instrument(name = "ProjectionExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|aliased_expr| {
                (
                    aliased_expr.alias,
                    aliased_expr.expr.result_evaluate(alloc, table),
                )
            }),
            TableOptions::new(Some(table.num_rows())),
        )
        .expect("Failed to create table from iterator")
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    #[tracing::instrument(
        name = "ProjectionExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. Evaluate result expressions
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|aliased_expr| {
                (
                    aliased_expr.alias,
                    aliased_expr.expr.prover_evaluate(builder, alloc, table),
                )
            }),
            TableOptions::new(Some(table.num_rows())),
        )
        .expect("Failed to create table from iterator");
        // 2. Produce MLEs
        res.inner_table().values().for_each(|column| {
            builder.produce_intermediate_mle(column.as_scalar(alloc));
        });
        res
    }
}
