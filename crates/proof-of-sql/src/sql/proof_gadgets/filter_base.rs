use crate::{
    base::{database::Column, proof::ProofError, scalar::Scalar},
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_gadgets::fold_log_expr::FoldLogExpr,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;

#[expect(clippy::similar_names)]
pub(crate) fn verify_evaluate_filter<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    columns_evals: &[S],
    chi_n_eval: S,
    chi_m_eval: S,
    s_eval: S,
) -> Result<Vec<S>, ProofError> {
    let fold_gadget = FoldLogExpr::new(alpha, beta);
    let c_star_eval = fold_gadget
        .verify_evaluate(builder, columns_evals, chi_n_eval)?
        .0;
    let filtered_columns_evals =
        builder.try_consume_first_round_mle_evaluations(columns_evals.len())?;
    let (d_star_eval, d_fold_eval) =
        fold_gadget.verify_evaluate(builder, &filtered_columns_evals, chi_m_eval)?;

    // sum c_star * s - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval * s_eval - d_star_eval,
        2,
    )?;

    // d_fold * chi_m - d_fold = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_fold_eval * (chi_m_eval - S::ONE),
        2,
    )?;

    Ok(filtered_columns_evals)
}

/// Below are the mappings between the names of the parameters in the math and the code
/// `c = columns`
/// `d = filtered_columns`
/// `n = input_length`
/// `m = output_length`
#[expect(clippy::too_many_arguments)]
#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn final_round_evaluate_filter<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    columns: &[Column<S>],
    s: &'a [bool],
    filtered_columns: &[Column<S>],
    input_length: usize,
    output_length: usize,
) {
    let fold_gadget = FoldLogExpr::new(alpha, beta);
    let c_star = fold_gadget
        .final_round_evaluate(builder, alloc, columns, input_length)
        .0;
    let chi_m = alloc.alloc_slice_fill_copy(output_length, true);
    let (d_star, d_fold) = fold_gadget.final_round_evaluate_with_chi(
        builder,
        alloc,
        filtered_columns,
        output_length,
        chi_m,
    );

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star), Box::new(s)]),
            (-S::one(), vec![Box::new(d_star)]),
        ],
    );

    // d_fold * chi_m - d_fold = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_fold), Box::new(chi_m as &[_])]),
            (-S::one(), vec![Box::new(d_fold)]),
        ],
    );
}
