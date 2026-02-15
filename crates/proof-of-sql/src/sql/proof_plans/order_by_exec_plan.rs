use crate::{
    base::{
        database::{
            Column, ColumnField, ColumnRef, ColumnType, LiteralValue, Table, TableEvaluation,
            TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
        PlaceholderResult,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_exprs::{DynProofExpr, ProofExpr},
        proof_gadgets::{
            final_round_evaluate_monotonic, final_round_evaluate_permutation_check,
            first_round_evaluate_monotonic, first_round_evaluate_permutation_check,
            verify_monotonic, verify_permutation_check,
        },
        proof_plans::DynProofPlan,
    },
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT ... FROM ... ORDER BY <order_by_expr>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderByExec {
    pub(super) input: Box<DynProofPlan>,
    pub(super) order_by_exprs: Vec<(DynProofExpr, bool)>,
}

impl OrderByExec {
    /// Creates a new order by expression.
    pub fn try_new(
        input: Box<DynProofPlan>,
        order_by_exprs: Vec<(DynProofExpr, bool)>,
    ) -> Option<Self> {
        let is_one_order_by = order_by_exprs.len() == 1;
        let all_exprs_valid_type = order_by_exprs.iter().all(|(expr, _)| {
            let column_type = expr.data_type();
            column_type != ColumnType::VarChar && column_type != ColumnType::VarBinary
        });
        (is_one_order_by && all_exprs_valid_type).then_some(Self {
            input,
            order_by_exprs,
        })
    }
}

impl ProofPlan for OrderByExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let input_evals = self
            .input
            .verifier_evaluate(builder, accessor, chi_eval_map, params)?;
        let accessor = self
            .input
            .get_column_result_fields()
            .iter()
            .map(ColumnField::name)
            .zip(input_evals.column_evals().iter().copied())
            .collect::<IndexMap<_, _>>();
        let sort_expr_evals: Vec<(S, bool)> = self
            .order_by_exprs
            .iter()
            .map(|(expr, asc)| -> Result<(S, bool), ProofError> {
                Ok((
                    expr.verifier_evaluate(builder, &accessor, input_evals.chi_eval(), params)?,
                    *asc,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (order_by_eval, asc) = sort_expr_evals
            .first()
            .copied()
            .expect("Only one column is being used for now.");
        let columns_to_permute: Vec<_> = input_evals
            .column_evals()
            .iter()
            .copied()
            .chain(core::iter::once(order_by_eval))
            .collect();
        let mut permuted_columns = verify_permutation_check(
            builder,
            alpha,
            beta,
            input_evals.chi_eval(),
            &columns_to_permute,
        )?;
        let permuted_order_by_eval = permuted_columns.pop().expect("At least once column exists");

        if asc {
            // For ASC, we want to verify that the order by column is monotonically increasing
            verify_monotonic::<S, false, true>(
                builder,
                alpha,
                beta,
                permuted_order_by_eval,
                input_evals.chi_eval(),
            )?;
        } else {
            // For DESC, we want to verify that the order by column is monotonically decreasing
            verify_monotonic::<S, false, false>(
                builder,
                alpha,
                beta,
                permuted_order_by_eval,
                input_evals.chi_eval(),
            )?;
        }

        Ok(TableEvaluation::new(permuted_columns, input_evals.chi()))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.input.get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.input.get_column_references()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

fn get_sorting_permuation<S: Scalar>(column: &Column<'_, S>, asc: bool) -> Vec<usize> {
    let mut values = column
        .to_scalar()
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
    values.sort_by_key(|(_, value)| *value);
    let permutation: Vec<usize> = values.into_iter().map(|(index, _)| index).collect();
    if asc {
        permutation
    } else {
        permutation.into_iter().rev().collect()
    }
}

impl ProverEvaluate for OrderByExec {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        builder.request_post_result_challenges(2);
        let input_columns = self
            .input
            .first_round_evaluate(builder, alloc, table_map, params)?;
        let chi = alloc.alloc_slice_fill_copy(input_columns.num_rows(), true);
        let order_by_columns = self
            .order_by_exprs
            .iter()
            .map(|(expr, asc)| {
                expr.first_round_evaluate(alloc, &input_columns, params)
                    .map(|col| (col, *asc))
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let (order_by_column, asc) = order_by_columns
            .first()
            .copied()
            .expect("Only one column is being used for now.");
        let permutation = get_sorting_permuation(&order_by_column, asc);
        let columns_to_permute: Vec<_> = input_columns
            .columns()
            .copied()
            .chain(core::iter::once(order_by_column))
            .collect();
        let mut permuted_columns = first_round_evaluate_permutation_check(
            builder,
            alloc,
            chi,
            &columns_to_permute,
            &permutation,
        );
        let sorted_order_by_column = alloc.alloc_slice_copy(
            &permuted_columns
                .pop()
                .expect("At least once column exists")
                .to_scalar(),
        );
        first_round_evaluate_monotonic(builder, alloc, sorted_order_by_column);
        Ok(Table::try_new(
            input_columns
                .column_names()
                .cloned()
                .zip(permuted_columns)
                .collect(),
        )
        .expect("Construction confirmed to be valid"))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        let input_columns = self
            .input
            .final_round_evaluate(builder, alloc, table_map, params)?;
        let chi = alloc.alloc_slice_fill_copy(input_columns.num_rows(), true);
        let order_by_columns = self
            .order_by_exprs
            .iter()
            .map(|(expr, asc)| {
                expr.final_round_evaluate(builder, alloc, &input_columns, params)
                    .map(|col| (col, *asc))
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let (order_by_column, asc) = order_by_columns
            .first()
            .copied()
            .expect("Only one column is being used for now.");
        let permutation = get_sorting_permuation(&order_by_column, asc);
        let columns_to_permute: Vec<_> = input_columns
            .columns()
            .copied()
            .chain(core::iter::once(order_by_column))
            .collect();
        let mut permuted_columns = final_round_evaluate_permutation_check(
            builder,
            alloc,
            alpha,
            beta,
            chi,
            &columns_to_permute,
            &permutation,
        );
        let sorted_order_by_column = alloc.alloc_slice_copy(
            &permuted_columns
                .pop()
                .expect("At least once column exists")
                .to_scalar(),
        );
        if asc {
            // For ASC, we want to verify that the order by column is monotonically increasing
            final_round_evaluate_monotonic::<_, false, true>(
                builder,
                alloc,
                alpha,
                beta,
                sorted_order_by_column,
            );
        } else {
            // For DESC, we want to verify that the order by column is monotonically decreasing
            final_round_evaluate_monotonic::<_, false, false>(
                builder,
                alloc,
                alpha,
                beta,
                sorted_order_by_column,
            );
        }
        Ok(Table::try_new(
            input_columns
                .column_names()
                .cloned()
                .zip(permuted_columns)
                .collect(),
        )
        .expect("Construction confirmed to be valid"))
    }
}
