use super::{fold_columns, fold_vals};
use crate::{
    base::{
        database::{
            group_by_util::{aggregate_columns, AggregatedColumns},
            order_by_util::compare_indexes_by_owned_columns,
            Column, ColumnField, ColumnRef, ColumnType, LiteralValue, OwnedTable, Table,
            TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, ProofExpr, TableExpr},
        proof_gadgets::{
            final_round_evaluate_monotonic, first_round_evaluate_monotonic, verify_monotonic,
        },
    },
    utils::log,
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::iter;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <group_by_expr1>, ..., <group_by_exprM>,
///         SUM(<sum_expr1>.expr) as <sum_expr1>.alias, ..., SUM(<sum_exprN>.expr) as <sum_exprN>.alias,
///         COUNT(*) as count_alias
///     FROM <table>
///     WHERE <where_clause>
///     GROUP BY <group_by_expr1>, ..., <group_by_exprM>
/// ```
///
/// Note: if `group_by_exprs` is empty, then the query is equivalent to removing the `GROUP BY` clause.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GroupByExec {
    pub(super) group_by_exprs: Vec<ColumnExpr>,
    pub(super) sum_expr: Vec<AliasedDynProofExpr>,
    pub(super) count_alias: Ident,
    pub(super) table: TableExpr,
    pub(super) where_clause: DynProofExpr,
}

impl GroupByExec {
    /// Creates a new `group_by` expression.
    pub fn new(
        group_by_exprs: Vec<ColumnExpr>,
        sum_expr: Vec<AliasedDynProofExpr>,
        count_alias: Ident,
        table: TableExpr,
        where_clause: DynProofExpr,
    ) -> Self {
        Self {
            group_by_exprs,
            sum_expr,
            count_alias,
            table,
            where_clause,
        }
    }

    /// Get a reference to the table expression
    pub fn table(&self) -> &TableExpr {
        &self.table
    }

    /// Get a reference to the where clause
    pub fn where_clause(&self) -> &DynProofExpr {
        &self.where_clause
    }

    /// Get a reference to the group by expressions
    pub fn group_by_exprs(&self) -> &[ColumnExpr] {
        &self.group_by_exprs
    }

    /// Get a reference to the sum expressions
    pub fn sum_expr(&self) -> &[AliasedDynProofExpr] {
        &self.sum_expr
    }

    /// Get a reference to the count alias
    pub fn count_alias(&self) -> &Ident {
        &self.count_alias
    }

    /// Checks if the group by expression can prove uniqueness
    /// This is true if there is only one group by column and its type is not `VarChar` and not `VarBinary`
    pub fn is_uniqueness_provable(&self) -> bool {
        if self.group_by_exprs.len() != 1 {
            return false;
        }

        let column_type = self.group_by_exprs[0].data_type();
        !matches!(column_type, ColumnType::VarChar | ColumnType::VarBinary)
    }
}

impl ProofPlan for GroupByExec {
    #[expect(clippy::too_many_lines)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let input_chi_eval = chi_eval_map
            .get(&self.table.table_ref)
            .expect("Chi eval not found")
            .0;
        let accessor = accessor
            .get(&self.table.table_ref)
            .cloned()
            .unwrap_or_else(|| [].into_iter().collect());

        // Compute g_in_star
        let group_by_evals = self
            .group_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, &accessor, input_chi_eval, params))
            .collect::<Result<Vec<_>, _>>()?;
        let g_in_fold_eval = alpha * fold_vals(beta, &group_by_evals);
        let g_in_star_eval = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            g_in_star_eval + g_in_star_eval * g_in_fold_eval - input_chi_eval,
            2,
        )?;
        // End compute g_in_star

        let where_eval =
            self.where_clause
                .verifier_evaluate(builder, &accessor, input_chi_eval, params)?;

        let aggregate_evals = self
            .sum_expr
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, &accessor, input_chi_eval, params)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // 3. filtered_columns
        let group_by_result_columns_evals =
            builder.try_consume_final_round_mle_evaluations(self.group_by_exprs.len())?;
        let sum_result_columns_evals =
            builder.try_consume_final_round_mle_evaluations(self.sum_expr.len())?;
        let count_column_eval = builder.try_consume_final_round_mle_evaluation()?;

        let output_chi_eval = builder.try_consume_chi_evaluation()?.0;

        let is_uniqueness_provable = self.is_uniqueness_provable();
        let g_out_fold_eval = alpha * fold_vals(beta, &group_by_result_columns_evals);
        let sum_in_fold_eval = input_chi_eval + beta * fold_vals(beta, &aggregate_evals);
        let sum_out_fold_eval =
            count_column_eval + beta * fold_vals(beta, &sum_result_columns_evals);

        let g_out_star_eval = builder.try_consume_final_round_mle_evaluation()?;

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            g_in_star_eval * where_eval * sum_in_fold_eval - g_out_star_eval * sum_out_fold_eval,
            3,
        )?;

        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            g_out_star_eval + g_out_star_eval * g_out_fold_eval - output_chi_eval,
            2,
        )?;

        if is_uniqueness_provable {
            assert_eq!(
                group_by_result_columns_evals.len(),
                1,
                "Expected exactly one group by column for uniqueness check"
            );
            verify_monotonic::<S, true, true>(
                builder,
                alpha,
                beta,
                group_by_result_columns_evals[0],
                output_chi_eval,
            )?;
        }
        match (is_uniqueness_provable, result) {
            (true, _) => (),
            (false, Some(table)) => {
                let cols = self
                    .group_by_exprs
                    .iter()
                    .map(|col| table.inner_table().get(&col.column_id()))
                    .collect::<Option<Vec<_>>>()
                    .ok_or(ProofError::VerificationError {
                        error: "Result does not all correct group by columns.",
                    })?;
                let num_rows = table.num_rows();
                if num_rows > 0
                    && (0..num_rows - 1)
                        .any(|i| compare_indexes_by_owned_columns(&cols, i, i + 1).is_ge())
                {
                    Err(ProofError::VerificationError {
                        error: "Result of group by not ordered as expected.",
                    })?;
                }
            }
            (false, None) => {
                Err(ProofError::UnsupportedQueryPlan {
                    error: "GroupByExec without provable uniqueness check currently only supported at top level of query plan.",
                })?;
            }
        }

        let column_evals = group_by_result_columns_evals
            .into_iter()
            .chain(sum_result_columns_evals)
            .chain(iter::once(count_column_eval))
            .collect::<Vec<_>>();
        Ok(TableEvaluation::new(column_evals, output_chi_eval))
    }

    #[expect(clippy::redundant_closure_for_method_calls)]
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.group_by_exprs
            .iter()
            .map(|col| col.get_column_field())
            .chain(self.sum_expr.iter().map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
            }))
            .chain(iter::once(ColumnField::new(
                self.count_alias.clone(),
                ColumnType::BigInt,
            )))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();

        for col in &self.group_by_exprs {
            columns.insert(col.get_column_reference());
        }
        for aliased_expr in &self.sum_expr {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref.clone()])
    }
}

impl ProverEvaluate for GroupByExec {
    #[tracing::instrument(name = "GroupByExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        builder.request_post_result_challenges(2);

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");

        // Compute g_in_star
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| -> PlaceholderResult<Column<'a, S>> {
                expr.first_round_evaluate(alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // End compute g_in_star

        let selection_column: Column<'a, S> = self
            .where_clause
            .first_round_evaluate(alloc, table, params)?;

        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr.expr.first_round_evaluate(alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // Compute filtered_columns
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
            ..
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, &[], &[], selection)
            .expect("columns should be aggregatable");
        let sum_result_columns_iter = sum_result_columns.iter().map(|col| Column::Scalar(col));
        let res = Table::<'a, S>::try_from_iter(
            self.get_column_result_fields()
                .into_iter()
                .map(|field| field.name())
                .zip(
                    group_by_result_columns
                        .into_iter()
                        .chain(sum_result_columns_iter)
                        .chain(iter::once(Column::BigInt(count_column))),
                ),
        )
        .expect("Failed to create table from column references");
        builder.produce_chi_evaluation_length(count_column.len());
        // Prove result uniqueness if possible
        if self.is_uniqueness_provable() {
            first_round_evaluate_monotonic(builder, res.num_rows());
        }

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "GroupByExec::final_round_evaluate", level = "debug", skip_all)]
    #[expect(clippy::too_many_lines)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");

        let n = table.num_rows();
        let chi_n = alloc.alloc_slice_fill_copy(n, true);

        // Compute g_in_star
        let group_by_columns = self
            .group_by_exprs
            .iter()
            .map(|expr| -> PlaceholderResult<Column<'a, S>> {
                expr.final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let g_in_fold = alloc.alloc_slice_fill_copy(n, Zero::zero());
        fold_columns(g_in_fold, alpha, beta, &group_by_columns);
        let g_in_star = alloc.alloc_slice_copy(g_in_fold);
        slice_ops::add_const::<S, S>(g_in_star, One::one());
        slice_ops::batch_inversion(g_in_star);
        builder.produce_intermediate_mle(g_in_star as &[_]);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(g_in_star as &[_])]),
                (
                    S::one(),
                    vec![Box::new(g_in_star as &[_]), Box::new(g_in_fold as &[_])],
                ),
                (-S::one(), vec![Box::new(chi_n as &[_])]),
            ],
        );
        // End compute g_in_star

        let selection_column: Column<'a, S> = self
            .where_clause
            .final_round_evaluate(builder, alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        let sum_columns = self
            .sum_expr
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // 3. Compute filtered_columns
        let AggregatedColumns {
            group_by_columns: group_by_result_columns,
            sum_columns: sum_result_columns,
            count_column,
            ..
        } = aggregate_columns(alloc, &group_by_columns, &sum_columns, &[], &[], selection)
            .expect("columns should be aggregatable");

        // 4. Tally results
        let sum_result_columns_iter = sum_result_columns.iter().map(|col| Column::Scalar(col));
        let columns = group_by_result_columns
            .clone()
            .into_iter()
            .chain(sum_result_columns_iter)
            .chain(iter::once(Column::BigInt(count_column)));
        let res = Table::<'a, S>::try_from_iter(
            self.get_column_result_fields()
                .into_iter()
                .map(|field| field.name())
                .zip(columns.clone()),
        )
        .expect("Failed to create table from column references");
        // 5. Produce MLEs
        for column in columns {
            builder.produce_intermediate_mle(column);
        }
        // 6. Prove group by
        let check_uniqueness = self.is_uniqueness_provable();
        let m = count_column.len();
        let chi_m = alloc.alloc_slice_fill_copy(m, true);

        let g_out_fold = alloc.alloc_slice_fill_copy(m, Zero::zero());
        fold_columns(g_out_fold, alpha, beta, &group_by_result_columns);

        let sum_in_fold = alloc.alloc_slice_fill_copy(n, One::one());
        fold_columns(sum_in_fold, beta, beta, &sum_columns);

        let sum_out_fold = alloc.alloc_slice_fill_default(m);
        slice_ops::slice_cast_mut(count_column, sum_out_fold);
        fold_columns(sum_out_fold, beta, beta, &sum_result_columns);

        let g_out_star = alloc.alloc_slice_copy(g_out_fold);
        slice_ops::add_const::<S, S>(g_out_star, One::one());
        slice_ops::batch_inversion(g_out_star);

        builder.produce_intermediate_mle(g_out_star as &[_]);

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::ZeroSum,
            vec![
                (
                    S::one(),
                    vec![
                        Box::new(g_in_star as &[_]),
                        Box::new(selection),
                        Box::new(sum_in_fold as &[_]),
                    ],
                ),
                (
                    -S::one(),
                    vec![Box::new(g_out_star as &[_]), Box::new(sum_out_fold as &[_])],
                ),
            ],
        );

        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(g_out_star as &[_])]),
                (
                    S::one(),
                    vec![Box::new(g_out_star as &[_]), Box::new(g_out_fold as &[_])],
                ),
                (-S::one(), vec![Box::new(chi_m as &[_])]),
            ],
        );

        if check_uniqueness {
            assert_eq!(
                group_by_result_columns.len(),
                1,
                "Expected exactly one group by column for uniqueness check"
            );
            let g_out_scalars = group_by_result_columns[0].to_scalar();
            let alloc_g_out_scalars = alloc.alloc_slice_copy(&g_out_scalars);
            final_round_evaluate_monotonic::<S, true, true>(
                builder,
                alloc,
                alpha,
                beta,
                alloc_g_out_scalars,
            );
        }

        log::log_memory_usage("End");

        Ok(res)
    }
}
