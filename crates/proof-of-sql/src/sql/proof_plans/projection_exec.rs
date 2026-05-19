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

    fn physical_aliased_results(&self) -> Vec<AliasedDynProofExpr> {
        self.aliased_results
            .iter()
            .flat_map(AliasedDynProofExpr::physical_result_exprs)
            .collect()
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
        let input_schema = self.input.get_column_result_fields_with_presence();
        let current_accessor = input_schema
            .iter()
            .zip(input_eval.column_evals())
            .map(|(field, eval)| (field.name().clone(), *eval))
            .collect::<IndexMap<_, _>>();
        let output_column_evals = self
            .physical_aliased_results()
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
            .map(AliasedDynProofExpr::result_field)
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        // Projection results can reference intermediate columns produced by child plans. Source
        // commitments should come from the input plan, whose physical result schema already
        // includes generated nullable presence columns when needed.
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
            .physical_aliased_results()
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
            .physical_aliased_results()
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
    use super::*;
    use crate::{
        base::{
            database::{table_utility::*, ColumnField, ColumnRef, ColumnType},
            math::decimal::Precision,
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::ProofPlan,
            proof_exprs::{AliasedDynProofExpr, DynProofExpr},
        },
    };

    #[test]
    fn projection_result_schema_preserves_nullable_propagating_fields() {
        let table_ref = TableRef::new("sxt", "orders");
        let amount_ref =
            ColumnRef::new_nullable(table_ref.clone(), "amount".into(), ColumnType::BigInt);
        let fee_ref = ColumnRef::new(table_ref.clone(), "fee".into(), ColumnType::BigInt);
        let total = DynProofExpr::try_new_add(
            DynProofExpr::new_column(amount_ref.clone()),
            DynProofExpr::new_column(fee_ref),
        )
        .unwrap();
        let provable_ast = ProjectionExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(amount_ref),
                    alias: "amount".into(),
                },
                AliasedDynProofExpr {
                    expr: total,
                    alias: "total".into(),
                },
            ],
            Box::new(DynProofPlan::new_table(
                table_ref,
                vec![
                    ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
                    ColumnField::new("fee".into(), ColumnType::BigInt),
                ],
            )),
        );

        let column_fields = provable_ast.get_column_result_fields();

        assert_eq!(
            column_fields,
            vec![
                ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
                ColumnField::new_nullable(
                    "total".into(),
                    ColumnType::Decimal75(Precision::new(20).unwrap(), 0)
                ),
            ]
        );
    }

    #[test]
    fn projection_physical_result_schema_adds_presence_fields_for_nullable_results() {
        let table_ref = TableRef::new("sxt", "orders");
        let amount_ref =
            ColumnRef::new_nullable(table_ref.clone(), "amount".into(), ColumnType::BigInt);
        let fee_ref = ColumnRef::new(table_ref.clone(), "fee".into(), ColumnType::BigInt);
        let total = DynProofExpr::try_new_add(
            DynProofExpr::new_column(amount_ref.clone()),
            DynProofExpr::new_column(fee_ref),
        )
        .unwrap();
        let provable_ast = ProjectionExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::new_column(amount_ref),
                    alias: "amount".into(),
                },
                AliasedDynProofExpr {
                    expr: total,
                    alias: "total".into(),
                },
            ],
            Box::new(DynProofPlan::new_table(
                table_ref,
                vec![
                    ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
                    ColumnField::new("fee".into(), ColumnType::BigInt),
                ],
            )),
        );

        let column_fields = provable_ast.get_column_result_fields_with_presence();

        assert_eq!(
            column_fields,
            vec![
                ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
                ColumnField::new("__posql_presence_amount".into(), ColumnType::Boolean),
                ColumnField::new_nullable(
                    "total".into(),
                    ColumnType::Decimal75(Precision::new(20).unwrap(), 0)
                ),
                ColumnField::new("__posql_presence_total".into(), ColumnType::Boolean),
            ]
        );
    }

    #[test]
    fn projection_first_round_evaluates_physical_nullable_results() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "orders");
        let flag_ref =
            ColumnRef::new_nullable(table_ref.clone(), "flag".into(), ColumnType::Boolean);
        let flag_and_false = DynProofExpr::try_new_and(
            DynProofExpr::new_column(flag_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::Boolean(false)),
        )
        .unwrap();
        let plan = ProjectionExec::new(
            vec![AliasedDynProofExpr {
                expr: flag_and_false,
                alias: "flag_known_false".into(),
            }],
            Box::new(DynProofPlan::new_table(
                table_ref.clone(),
                vec![ColumnField::new_nullable(
                    "flag".into(),
                    ColumnType::Boolean,
                )],
            )),
        );
        let table_map = IndexMap::from_iter([(
            table_ref,
            table([
                borrowed_boolean("flag", [true, true, false], &alloc),
                borrowed_boolean("__posql_presence_flag", [true, false, true], &alloc),
            ]),
        )]);
        let mut builder = FirstRoundBuilder::new(3);

        let result: Table<TestScalar> = plan
            .first_round_evaluate(&mut builder, &alloc, &table_map, &[])
            .unwrap();

        assert_eq!(
            result.inner_table().keys().cloned().collect::<Vec<_>>(),
            vec![
                "flag_known_false".into(),
                "__posql_presence_flag_known_false".into(),
            ]
        );
        assert_eq!(
            *result
                .inner_table()
                .get(&Ident::new("flag_known_false"))
                .unwrap(),
            Column::Boolean(&[false, false, false])
        );
        assert_eq!(
            *result
                .inner_table()
                .get(&Ident::new("__posql_presence_flag_known_false"))
                .unwrap(),
            Column::Boolean(&[true, true, true])
        );
    }
}
