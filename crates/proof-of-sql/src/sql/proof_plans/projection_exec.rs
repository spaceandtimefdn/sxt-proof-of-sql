use super::DynProofPlan;
use crate::{
    base::{
        database::{
            Column, ColumnField, ColumnRef, LiteralValue, Table, TableEvaluation, TableOptions,
            TableRef,
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
        let output_column_evals = self
            .aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, &current_accessor, chi.0, params)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TableEvaluation::new(output_column_evals, chi))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
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

        let cols = self
            .aliased_results
            .iter()
            .map(
                |aliased_expr| -> PlaceholderResult<(Ident, Column<'a, S>)> {
                    Ok((
                        aliased_expr.alias.clone(),
                        aliased_expr
                            .expr
                            .first_round_evaluate(alloc, &input, params)?,
                    ))
                },
            )
            .collect::<PlaceholderResult<IndexMap<_, _>>>()?;

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
        let cols = self
            .aliased_results
            .iter()
            .map(
                |aliased_expr| -> PlaceholderResult<(Ident, Column<'a, S>)> {
                    Ok((
                        aliased_expr.alias.clone(),
                        aliased_expr
                            .expr
                            .final_round_evaluate(builder, alloc, &input, params)?,
                    ))
                },
            )
            .collect::<PlaceholderResult<IndexMap<_, _>>>()?;

        let res =
            Table::<'a, S>::try_new_with_options(cols, TableOptions::new(Some(input.num_rows())))
                .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectionExec;
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::{
            proof::ProofPlan,
            proof_exprs::{AliasedDynProofExpr, DynProofExpr},
            proof_plans::DynProofPlan,
        },
    };

    #[test]
    fn projection_constructor_preserves_input_results_and_output_fields() {
        let input = DynProofPlan::new_empty();
        let aliased_results = vec![
            AliasedDynProofExpr {
                expr: DynProofExpr::new_literal(LiteralValue::BigInt(7)),
                alias: "total".into(),
            },
            AliasedDynProofExpr {
                expr: DynProofExpr::new_literal(LiteralValue::Boolean(true)),
                alias: "flag".into(),
            },
        ];

        let projection = ProjectionExec::new(aliased_results.clone(), Box::new(input.clone()));

        assert_eq!(projection.input(), &input);
        assert_eq!(projection.aliased_results(), aliased_results.as_slice());
        let fields = projection.get_column_result_fields();
        assert_eq!(fields[0].name().value, "total");
        assert_eq!(fields[0].data_type(), ColumnType::BigInt);
        assert_eq!(fields[1].name().value, "flag");
        assert_eq!(fields[1].data_type(), ColumnType::Boolean);
    }
}
