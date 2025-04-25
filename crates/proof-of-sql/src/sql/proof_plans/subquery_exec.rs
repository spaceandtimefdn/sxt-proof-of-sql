use super::DynProofPlan;
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, LiteralValue, OwnedTable, Table, TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
        PlaceholderResult,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SubqueryExec {
    pub(super) alias: TableRef,
    pub(super) input: Box<DynProofPlan>,
}

impl SubqueryExec {
    /// Creates a new subquery expression.
    pub fn new(alias: TableRef, input: Box<DynProofPlan>) -> Self {
        Self { alias, input }
    }

    /// Get a reference to the input plan
    pub fn input(&self) -> &DynProofPlan {
        &self.input
    }

    /// Get a reference to the alias
    pub fn alias(&self) -> &TableRef {
        &self.alias
    }
}

impl ProofPlan for SubqueryExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        self.input
            .verifier_evaluate(builder, accessor, result, chi_eval_map, params)
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.input.get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.input
            .get_column_references()
            .into_iter()
            .chain(
                self.get_column_result_fields()
                    .iter()
                    .map(|field| field.into_ref(self.alias.clone())),
            )
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input
            .get_table_references()
            .into_iter()
            .chain([self.alias.clone()])
            .collect()
    }
}

impl ProverEvaluate for SubqueryExec {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        self.input
            .first_round_evaluate(builder, alloc, table_map, params)
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        self.input
            .final_round_evaluate(builder, alloc, table_map, params)
    }
}
