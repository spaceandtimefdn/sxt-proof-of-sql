use super::{fold_columns, fold_vals, DynProofPlan};
use crate::{
    base::{
        database::{
            union_util::table_union, Column, ColumnField, ColumnRef, OwnedTable, Table,
            TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        polynomial::MultilinearExtension,
        proof::ProofError,
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
    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let input_table_evals = self
            .inputs
            .iter()
            .map(|input| input.verifier_evaluate(builder, accessor, None, chi_eval_map))
            .collect::<Result<Vec<_>, _>>()?;
        let num_parts = self.inputs.len();
        let input_column_evals = input_table_evals
            .iter()
            .map(TableEvaluation::column_evals)
            .collect::<Vec<_>>();
        let output_column_evals =
            builder.try_consume_final_round_mle_evaluations(self.schema.len())?;
        let chi_n_evals = input_table_evals
            .iter()
            .map(TableEvaluation::chi_eval)
            .collect::<Vec<_>>();
        let chi_m_eval = builder.try_consume_chi_evaluation()?;
        let gamma = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        verify_union(
            builder,
            gamma,
            beta,
            &input_column_evals,
            &output_column_evals,
            &chi_n_evals,
            chi_m_eval,
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
    ) -> Table<'a, S> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| input.first_round_evaluate(builder, alloc, table_map))
            .collect::<Vec<_>>();
        let res = table_union(&inputs, alloc, self.schema.clone()).expect("Failed to union tables");
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(res.num_rows());
        res
    }

    #[tracing::instrument(name = "UnionExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let inputs = self
            .inputs
            .iter()
            .map(|input| input.final_round_evaluate(builder, alloc, table_map))
            .collect::<Vec<_>>();
        let input_lengths = inputs.iter().map(Table::num_rows).collect::<Vec<_>>();
        let res = table_union(&inputs, alloc, self.schema.clone()).expect("Failed to union tables");
        let gamma = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        let input_columns: Vec<Vec<Column<'a, S>>> = inputs
            .iter()
            .map(|table| table.columns().copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let output_columns: Vec<Column<'a, S>> = res.columns().copied().collect::<Vec<_>>();
        // Produce intermediate MLEs for the union
        output_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        // Produce the proof for the union
        prove_union(
            builder,
            alloc,
            gamma,
            beta,
            &input_columns,
            &output_columns,
            &input_lengths,
            res.num_rows(),
        );
        res
    }
}

/// Verifies the union of tables.
///
/// # Panics
/// Should never panic if the code is correct.
#[allow(clippy::too_many_arguments)]
fn verify_union<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    gamma: S,
    beta: S,
    input_evals: &[&[S]],
    output_eval: &[S],
    chi_n_evals: &[S],
    chi_m_eval: S,
) -> Result<(), ProofError> {
    assert_eq!(input_evals.len(), chi_n_evals.len());
    let c_star_evals = input_evals
        .iter()
        .zip(chi_n_evals)
        .map(|(&input_eval, &input_chi_eval)| -> Result<_, ProofError> {
            let c_fold_eval = gamma * fold_vals(beta, input_eval);
            let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
            // c_star + c_fold * c_star - chi_n_i = 0
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                c_star_eval + c_fold_eval * c_star_eval - input_chi_eval,
                2,
            )?;
            Ok(c_star_eval)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let d_bar_fold_eval = gamma * fold_vals(beta, output_eval);
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

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
    Ok(())
}

/// Proves the union of tables.
///
/// # Panics
/// Should never panic if the code is correct.
#[allow(clippy::too_many_arguments)]
fn prove_union<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    gamma: S,
    beta: S,
    input_tables: &[Vec<Column<'a, S>>],
    output_table: &[Column<'a, S>],
    input_lengths: &[usize],
    output_length: usize,
) {
    // Number of `ProofPlan`s should be a constant
    assert_eq!(input_tables.len(), input_lengths.len());
    let c_stars = input_lengths
        .iter()
        .zip(input_tables.iter())
        .map(|(&input_length, input_table)| {
            // Indicator vector for the input table
            let chi_n_i = alloc.alloc_slice_fill_copy(input_length, true);

            let c_fold = alloc.alloc_slice_fill_copy(input_length, Zero::zero());
            fold_columns(c_fold, gamma, beta, input_table);

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
            c_star_copy
        })
        .collect::<Vec<_>>();
    // No need to produce intermediate MLEs for `d_fold` because it is
    // the sum of `c_fold`
    let d_fold = alloc.alloc_slice_fill_copy(output_length, Zero::zero());
    fold_columns(d_fold, gamma, beta, output_table);

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
}
