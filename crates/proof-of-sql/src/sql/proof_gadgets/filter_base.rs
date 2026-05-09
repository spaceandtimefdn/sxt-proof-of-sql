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
    use super::*;
    use crate::base::{database::Column, scalar::test_scalar::TestScalar};
    use alloc::collections::VecDeque;
    use num_traits::{One, Zero};

    #[test]
    fn we_build_filter_final_round_intermediate_mles_and_constraints() {
        let alloc = Bump::new();
        let input_column = Column::BigInt(&[1_i64, 2]);
        let output_column = Column::BigInt(&[1_i64]);
        let selection = [true, false];
        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());

        final_round_evaluate_filter(
            &mut builder,
            &alloc,
            TestScalar::from(3u64),
            TestScalar::from(10u64),
            &[input_column],
            &selection,
            &[output_column],
            2,
            1,
        );

        assert_eq!(builder.pcs_proof_mles().len(), 2);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 4);

        let evaluations = builder.evaluate_pcs_proof_mles(&[TestScalar::one(), TestScalar::zero()]);
        assert_eq!(evaluations.len(), 2);
        assert_eq!(evaluations[0] * TestScalar::from(4u64), TestScalar::one());
        assert_eq!(evaluations[1] * TestScalar::from(4u64), TestScalar::one());
    }

    #[test]
    fn we_can_record_filter_constraints_without_intermediate_mles() {
        let c_star = [TestScalar::one()];
        let d_star = [TestScalar::one()];
        let selection = [true];
        let c_fold = [TestScalar::zero()];
        let d_fold = [TestScalar::zero()];
        let input_chi = [true];
        let output_chi = [true];
        let mut builder = FinalRoundBuilder::<TestScalar>::new(1, VecDeque::new());

        final_round_filter_constraints(
            &mut builder,
            &c_star,
            &d_star,
            &selection,
            &c_fold,
            &d_fold,
            &input_chi,
            &output_chi,
        );

        assert_eq!(builder.pcs_proof_mles().len(), 0);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 4);
    }
}
