use crate::{
    base::{database::Column, proof::ProofError, scalar::Scalar, slice_ops},
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_plans::fold_columns,
    },
};
use alloc::{boxed::Box, vec};
use ark_ff::{One, Zero};
use bumpalo::Bump;

#[expect(clippy::similar_names)]
pub(crate) fn verify_evaluate_filter<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    c_fold_eval: S,
    d_fold_eval: S,
    chi_n_eval: S,
    chi_m_eval: S,
    s_eval: S,
) -> Result<(), ProofError> {
    let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    // c_star + c_fold * c_star - chi_n = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        c_star_eval + c_fold_eval * c_star_eval - chi_n_eval,
        2,
    )?;

    // d_star + d_fold * d_star - chi_m = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_star_eval + d_fold_eval * d_star_eval - chi_m_eval,
        2,
    )?;

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

    Ok(())
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
    let chi_n = alloc.alloc_slice_fill_copy(input_length, true);
    let chi_m = alloc.alloc_slice_fill_copy(output_length, true);

    let c_fold = alloc.alloc_slice_fill_copy(input_length, Zero::zero());
    fold_columns(c_fold, alpha, beta, columns);
    let d_fold = alloc.alloc_slice_fill_copy(output_length, Zero::zero());
    fold_columns(d_fold, alpha, beta, filtered_columns);

    let c_star = alloc.alloc_slice_copy(c_fold);
    slice_ops::add_const::<S, S>(c_star, One::one());
    slice_ops::batch_inversion(c_star);

    let d_star = alloc.alloc_slice_copy(d_fold);
    slice_ops::add_const::<S, S>(d_star, One::one());
    slice_ops::batch_inversion(d_star);

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(d_star as &[_]);

    final_round_filter_constraints(
        builder,
        c_star as &[_],
        d_star as &[_],
        s,
        c_fold as &[_],
        d_fold as &[_],
        chi_n,
        chi_m,
    );
}

#[expect(clippy::too_many_arguments)]
pub(crate) fn final_round_filter_constraints<'a, S: Scalar + 'a>(
    builder: &mut FinalRoundBuilder<'a, S>,
    c_star: &'a [S],
    d_star: &'a [S],
    selection: &'a [bool],
    c_fold: &'a [S],
    d_fold: &'a [S],
    input_chi: &'a [bool],
    output_chi: &'a [bool],
) {
    // c_star + c_fold * c_star - chi_n = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star)]),
            (S::one(), vec![Box::new(c_star), Box::new(c_fold)]),
            (-S::one(), vec![Box::new(input_chi)]),
        ],
    );

    // d_star + d_fold * d_star - chi_m = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star)]),
            (S::one(), vec![Box::new(d_star), Box::new(d_fold)]),
            (-S::one(), vec![Box::new(output_chi)]),
        ],
    );

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star), Box::new(selection)]),
            (-S::one(), vec![Box::new(d_star)]),
        ],
    );

    // d_fold * chi_m - d_fold = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_fold), Box::new(output_chi)]),
            (-S::one(), vec![Box::new(d_fold)]),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::{final_round_evaluate_filter, verify_evaluate_filter};
    use crate::{
        base::{
            database::table_utility::borrowed_bigint,
            polynomial::MultilinearExtension,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{
            mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder,
            FirstRoundBuilder,
        },
        sql::proof_plans::fold_vals,
    };
    use bumpalo::Bump;
    use core::convert::identity;
    use std::collections::VecDeque;

    #[test]
    fn we_can_verify_filter_constraints() {
        let alloc = Bump::new();
        let input_column = borrowed_bigint::<TestScalar>("input", [1, 2, 3], &alloc).1;
        let filtered_column = borrowed_bigint::<TestScalar>("filtered", [1, 3], &alloc).1;
        let selection = &[true, false, true];
        let output_chi = &[true, true];
        let alpha = TestScalar::TWO;
        let beta = TestScalar::TEN;

        let first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(3);
        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(3, VecDeque::new());
        final_round_evaluate_filter(
            &mut final_round_builder,
            &alloc,
            alpha,
            beta,
            &[input_column.clone()],
            selection,
            &[filtered_column.clone()],
            3,
            2,
        );

        assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 4);

        let verification_builder = run_verify_for_each_row(
            3,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            3,
            |verification_builder, input_chi_eval, evaluation_point| {
                let c_fold_eval =
                    alpha * fold_vals(beta, &[input_column.inner_product(evaluation_point)]);
                let d_fold_eval =
                    alpha * fold_vals(beta, &[filtered_column.inner_product(evaluation_point)]);
                verify_evaluate_filter(
                    verification_builder,
                    c_fold_eval,
                    d_fold_eval,
                    input_chi_eval,
                    output_chi.inner_product(evaluation_point),
                    selection.inner_product(evaluation_point),
                )
                .unwrap();
            },
        );

        assert!(verification_builder
            .get_identity_results()
            .iter()
            .all(|row| row.iter().copied().all(identity)));
        assert!(verification_builder
            .get_zero_sum_results()
            .iter()
            .copied()
            .all(identity));
    }
}
