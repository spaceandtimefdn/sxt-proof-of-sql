use super::{fold_columns, fold_vals, DynProofPlan};
use crate::{
    base::{
        database::{
            union_util::table_union, Column, ColumnField, ColumnRef, LiteralValue, OwnedTable,
            Table, TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        polynomial::MultilinearExtension,
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, SumcheckSubpolynomialType,
        VerificationBuilder,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan>
///     UNION ALL
///     <ProofPlan>
///     ...
///     UNION ALL
///     <ProofPlan>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UnionExec {
    pub(super) inputs: Vec<DynProofPlan>,
    pub(super) schema: Vec<ColumnField>,
}

impl UnionExec {
    /// Creates a new union execution plan.
    pub fn new(inputs: Vec<DynProofPlan>, schema: Vec<ColumnField>) -> Self {
        Self { inputs, schema }
    }
}

impl ProofPlan for UnionExec
where
    UnionExec: ProverEvaluate,
{
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let gamma = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let c_star_evals = self
            .inputs
            .iter()
            .map(|input| -> Result<_, ProofError> {
                let table_evaluation =
                    input.verifier_evaluate(builder, accessor, None, chi_eval_map, params)?;
                let c_fold_eval = gamma * fold_vals(beta, table_evaluation.column_evals());
                let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
                // c_star + c_fold * c_star - chi_n_i = 0
                builder.try_produce_sumcheck_subpolynomial_evaluation(
                    SumcheckSubpolynomialType::Identity,
                    c_star_eval + c_fold_eval * c_star_eval - table_evaluation.chi_eval(),
                    2,
                )?;
                Ok(c_star_eval)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let output_column_evals =
            builder.try_consume_first_round_mle_evaluations(self.schema.len())?;

        let d_bar_fold_eval = gamma * fold_vals(beta, &output_column_evals);
        let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;
        let chi_m_eval = builder.try_consume_chi_evaluation()?.0;

        // d_star + d_bar_fold * d_star - chi_m = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            d_star_eval + d_bar_fold_eval * d_star_eval - chi_m_eval,
            2,
        )?;

        // sum (sum c_star) - d_star = 0
        let zero_sum_terms_eval = c_star_evals
            .into_iter()
            .chain(core::iter::once(-d_star_eval))
            .sum::<S>();
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            zero_sum_terms_eval,
            1,
        )?;
        Ok(TableEvaluation::new(output_column_evals, chi_m_eval))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.schema.clone()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.inputs
            .iter()
            .flat_map(ProofPlan::get_column_references)
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.inputs
            .iter()
            .flat_map(ProofPlan::get_table_references)
            .collect()
    }
}

impl ProverEvaluate for UnionExec {
    #[tracing::instrument(name = "UnionExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        builder.request_post_result_challenges(2);
        let inputs = self
            .inputs
            .iter()
            .map(|input| -> PlaceholderResult<Table<'a, S>> {
                input.first_round_evaluate(builder, alloc, table_map, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let res = table_union(&inputs, alloc, self.schema.clone()).expect("Failed to union tables");

        // Produce intermediate MLEs for the union
        res.columns().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        builder.produce_chi_evaluation_length(res.num_rows());
        Ok(res)
    }

    #[tracing::instrument(name = "UnionExec::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let gamma = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        // Produce the proof for the union
        let (inputs, c_stars): (Vec<_>, Vec<_>) = self
            .inputs
            .iter()
            .map(|input| -> PlaceholderResult<_> {
                let table = input.final_round_evaluate(builder, alloc, table_map, params)?;
                let input_length = table.num_rows();
                let input_table = table.columns().copied().collect::<Vec<_>>();
                // Indicator vector for the input table
                let chi_n_i = alloc.alloc_slice_fill_copy(input_length, true);

                let c_fold = alloc.alloc_slice_fill_copy(input_length, Zero::zero());
                fold_columns(c_fold, gamma, beta, &input_table);

                let c_star = alloc.alloc_slice_copy(c_fold);
                slice_ops::add_const::<S, S>(c_star, One::one());
                slice_ops::batch_inversion(&mut c_star[..input_length]);
                let c_star_copy = alloc.alloc_slice_copy(c_star);
                builder.produce_intermediate_mle(c_star as &[_]);

                // c_star + c_fold * c_star - chi_n_i = 0
                builder.produce_sumcheck_subpolynomial(
                    SumcheckSubpolynomialType::Identity,
                    vec![
                        (S::one(), vec![Box::new(c_star as &[_])]),
                        (
                            S::one(),
                            vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
                        ),
                        (-S::one(), vec![Box::new(chi_n_i as &[_])]),
                    ],
                );
                Ok((table, c_star_copy))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let res = table_union(&inputs, alloc, self.schema.clone()).expect("Failed to union tables");
        let output_columns: Vec<Column<'a, S>> = res.columns().copied().collect::<Vec<_>>();
        let output_length = res.num_rows();
        // No need to produce intermediate MLEs for `d_fold` because it is
        // the sum of `c_fold`
        let d_fold = alloc.alloc_slice_fill_copy(output_length, Zero::zero());
        fold_columns(d_fold, gamma, beta, &output_columns);

        let d_star = alloc.alloc_slice_copy(d_fold);
        slice_ops::add_const::<S, S>(d_star, One::one());
        slice_ops::batch_inversion(d_star);
        builder.produce_intermediate_mle(d_star as &[_]);
        // d_star + d_fold * d_star - chi_m = 0
        let chi_m = alloc.alloc_slice_fill_copy(output_length, true);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(d_star as &[_])]),
                (
                    S::one(),
                    vec![Box::new(d_star as &[_]), Box::new(d_fold as &[_])],
                ),
                (-S::one(), vec![Box::new(chi_m as &[_])]),
            ],
        );

        // sum (sum c_star) - d_star = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::ZeroSum,
            c_stars
                .into_iter()
                .map(|c_star| {
                    let boxed_c_star: Box<dyn MultilinearExtension<S>> = Box::new(c_star as &[_]);
                    (S::one(), vec![boxed_c_star])
                })
                .chain(core::iter::once({
                    let boxed_d_star: Box<dyn MultilinearExtension<S>> = Box::new(d_star as &[_]);
                    (-S::one(), vec![boxed_d_star])
                }))
                .collect(),
        );
        Ok(res)
    }
}
