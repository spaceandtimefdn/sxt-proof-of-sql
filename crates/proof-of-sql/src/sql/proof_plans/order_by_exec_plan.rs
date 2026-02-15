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
pub struct OrderByExec<const ASC: bool> {
    pub(super) input: Box<DynProofPlan>,
    pub(super) order_by_exprs: Vec<DynProofExpr>,
}

impl<const ASC: bool> OrderByExec<ASC> {
    /// Creates a new order by expression.
    pub fn try_new(input: Box<DynProofPlan>, order_by_exprs: Vec<DynProofExpr>) -> Option<Self> {
        (order_by_exprs.len() == 1)
            .then(|| order_by_exprs[0].data_type())
            .and_then(|data_type| {
                (data_type != ColumnType::VarChar && data_type != ColumnType::VarBinary).then(
                    || Self {
                        input,
                        order_by_exprs,
                    },
                )
            })
    }
}

impl<const ASC: bool> ProofPlan for OrderByExec<ASC> {
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
        let order_by_evals: Vec<S> = self
            .order_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, &accessor, input_evals.chi_eval(), params))
            .collect::<Result<Vec<_>, _>>()?;
        let order_by_eval = order_by_evals
            .first()
            .expect("Only one column is being used for now.");
        let columns_to_permute: Vec<_> = input_evals
            .column_evals()
            .iter()
            .chain(core::iter::once(order_by_eval))
            .copied()
            .collect();
        let mut permuted_columns = verify_permutation_check(
            builder,
            alpha,
            beta,
            input_evals.chi_eval(),
            &columns_to_permute,
        )?;
        let permuted_order_by_eval = permuted_columns.pop().expect("At least once column exists");

        verify_monotonic::<S, false, ASC>(
            builder,
            alpha,
            beta,
            permuted_order_by_eval,
            input_evals.chi_eval(),
        )?;

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

fn get_sorting_permuation<S: Scalar, const ASC: bool>(column: &Column<'_, S>) -> Vec<usize> {
    let mut values = column
        .to_scalar()
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
    values.sort_by_key(|(_, value)| *value);
    let asc: Vec<usize> = values.into_iter().map(|(index, _)| index).collect();
    if ASC {
        asc
    } else {
        asc.into_iter().rev().collect()
    }
}

impl<const ASC: bool> ProverEvaluate for OrderByExec<ASC> {
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
            .map(|expr| expr.first_round_evaluate(alloc, &input_columns, params))
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let order_by_column = order_by_columns
            .first()
            .expect("Only one column is being used for now.");
        let permutation = get_sorting_permuation::<_, ASC>(order_by_column);
        let columns_to_permute: Vec<_> = input_columns
            .columns()
            .chain(core::iter::once(order_by_column))
            .copied()
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
            .map(|expr| expr.final_round_evaluate(builder, alloc, &input_columns, params))
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let order_by_column = order_by_columns
            .first()
            .expect("Only one column is being used for now.");
        let permutation = get_sorting_permuation::<_, ASC>(order_by_column);
        let columns_to_permute: Vec<_> = input_columns
            .columns()
            .chain(core::iter::once(order_by_column))
            .copied()
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
        final_round_evaluate_monotonic::<_, false, ASC>(
            builder,
            alloc,
            alpha,
            beta,
            sorted_order_by_column,
        );
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
