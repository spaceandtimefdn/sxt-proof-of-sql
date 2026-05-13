use super::DynProofPlan;
use crate::{
    base::{
        database::{
            presence_column_id, Column, ColumnField, ColumnRef, ColumnType, LiteralValue, Table,
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
        proof_exprs::{AliasedDynProofExpr, ProofExpr},
    },
    utils::log,
};
use alloc::{boxed::Box, vec::Vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <input>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ProjectionExec {
    pub(super) aliased_results: Vec<AliasedDynProofExpr>,
    pub(super) input: Box<DynProofPlan>,
}

impl ProjectionExec {
    /// Creates a new projection expression.
    pub fn new(aliased_results: Vec<AliasedDynProofExpr>, input: Box<DynProofPlan>) -> Self {
        Self {
            aliased_results,
            input,
        }
    }

    /// Get a reference to the input plan
    pub fn input(&self) -> &DynProofPlan {
        &self.input
    }

    /// Get a reference to the aliased results
    pub fn aliased_results(&self) -> &[AliasedDynProofExpr] {
        &self.aliased_results
    }
}

impl ProofPlan for ProjectionExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // For projections input and output have the same length and hence the same chi eval
        let input_eval = self
            .input
            .verifier_evaluate(builder, accessor, chi_eval_map, params)?;
        let chi = input_eval.chi();
        // Build new accessors
        // TODO: Make this work with inputs with multiple tables such as join
        // and union results
        let input_schema = self.input.get_column_result_fields();
        let current_accessor = input_schema
            .iter()
            .zip(input_eval.column_evals())
            .map(|(field, eval)| (field.name().clone(), *eval))
            .collect::<IndexMap<_, _>>();
        let output_column_evals = self.aliased_results.iter().try_fold(
            Vec::new(),
            |mut output_column_evals, aliased_expr| {
                let eval = aliased_expr.expr.verifier_evaluate_nullable(
                    builder,
                    &current_accessor,
                    chi.0,
                    params,
                )?;
                output_column_evals.push(eval.value_eval());
                if let Some(presence_eval) = eval.presence_eval() {
                    output_column_evals.push(presence_eval);
                }
                Ok::<_, ProofError>(output_column_evals)
            },
        )?;
        Ok(TableEvaluation::new(output_column_evals, chi))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .flat_map(|aliased_expr| {
                let mut fields = Vec::with_capacity(if aliased_expr.expr.is_nullable() {
                    2
                } else {
                    1
                });
                fields.push(if aliased_expr.expr.is_nullable() {
                    ColumnField::new_nullable(
                        aliased_expr.alias.clone(),
                        aliased_expr.expr.data_type(),
                    )
                } else {
                    ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
                });
                if aliased_expr.expr.is_nullable() {
                    fields.push(ColumnField::new(
                        presence_column_id(&aliased_expr.alias),
                        ColumnType::Boolean,
                    ));
                }
                fields
            })
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        // For projections any output column reference is a reference to an input column
        self.input.get_column_references()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for ProjectionExec {
    #[tracing::instrument(
        name = "ProjectionExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let input = self
            .input
            .first_round_evaluate(builder, alloc, table_map, params)?;

        let cols = self.aliased_results.iter().try_fold(
            IndexMap::default(),
            |mut cols, aliased_expr| -> PlaceholderResult<_> {
                let column = aliased_expr
                    .expr
                    .first_round_evaluate_nullable(alloc, &input, params)?;
                cols.insert(aliased_expr.alias.clone(), column.values());
                if let Some(presence) = column.presence() {
                    cols.insert(
                        presence_column_id(&aliased_expr.alias),
                        Column::Boolean(presence),
                    );
                }
                Ok(cols)
            },
        )?;

        let res =
            Table::<'a, S>::try_new_with_options(cols, TableOptions::new(Some(input.num_rows())))
                .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "ProjectionExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let input = self
            .input
            .final_round_evaluate(builder, alloc, table_map, params)?;

        // Evaluate result expressions
        let cols = self.aliased_results.iter().try_fold(
            IndexMap::default(),
            |mut cols, aliased_expr| -> PlaceholderResult<_> {
                let column = aliased_expr
                    .expr
                    .final_round_evaluate_nullable(builder, alloc, &input, params)?;
                cols.insert(aliased_expr.alias.clone(), column.values());
                if let Some(presence) = column.presence() {
                    cols.insert(
                        presence_column_id(&aliased_expr.alias),
                        Column::Boolean(presence),
                    );
                }
                Ok(cols)
            },
        )?;

        let res =
            Table::<'a, S>::try_new_with_options(cols, TableOptions::new(Some(input.num_rows())))
                .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        Ok(res)
    }
}
