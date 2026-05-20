use crate::{
    base::{
        database::{join_util::get_multiplicities, Column},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_gadgets::fold_log_expr::FoldLogExpr,
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;

/// Perform first round evaluation of the membership check.
///
/// # Panics
/// Panics if the number of source and candidate columns are not equal
/// or if the number of columns is zero.
#[tracing::instrument(
    name = "MembershipChecks::first_round_evaluate_membership_check",
    level = "debug",
    skip_all
)]
pub(crate) fn first_round_evaluate_membership_check<'a, S: Scalar>(
    builder: &mut FirstRoundBuilder<'a, S>,
    alloc: &'a Bump,
    columns: &[Column<'a, S>],
    candidate_subset: &[Column<'a, S>],
) -> &'a [i128] {
    assert_eq!(
        columns.len(),
        candidate_subset.len(),
        "The number of source and candidate columns should be equal"
    );
    assert!(
        !columns.is_empty(),
        "The number of source columns should be greater than 0"
    );
    let multiplicities = get_multiplicities::<S>(candidate_subset, columns, alloc);
    builder.produce_intermediate_mle(multiplicities as &[_]);
    multiplicities
}

/// Perform final round evaluation of the membership check.
///
/// # Panics
/// Panics if the number of source and candidate columns are not equal
/// or if the number of columns is zero.
#[expect(clippy::too_many_arguments)]
#[tracing::instrument(
    name = "MembershipChecks::final_round_evaluate_membership_check",
    level = "debug",
    skip_all
)]
pub(crate) fn final_round_evaluate_membership_check<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    chi_n: &'a [bool],
    chi_m: &'a [bool],
    columns: &[Column<'a, S>],
    candidate_subset: &[Column<'a, S>],
) -> &'a [i128] {
    assert_eq!(
        columns.len(),
        candidate_subset.len(),
        "The number of source and candidate columns should be equal"
    );
    assert!(
        !columns.is_empty(),
        "The number of source columns should be greater than 0"
    );
    let multiplicities = get_multiplicities::<S>(candidate_subset, columns, alloc);

    let fold_gadget = FoldLogExpr::new(alpha, beta);
    let c_star = fold_gadget
        .final_round_evaluate_with_chi(builder, alloc, columns, chi_n.len(), chi_n)
        .0;
    let d_star = fold_gadget
        .final_round_evaluate_with_chi(builder, alloc, candidate_subset, chi_m.len(), chi_m)
        .0;

    // sum c_star * multiplicities - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(multiplicities as &[_])],
            ),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );
    multiplicities
}

#[expect(clippy::similar_names)]
pub(crate) fn verify_membership_check<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    chi_n_eval: S,
    chi_m_eval: S,
    column_evals: &[S],
    candidate_evals: &[S],
) -> Result<S, ProofError> {
    // Check that the source and candidate columns have the same amount of columns
    if column_evals.len() != candidate_evals.len() {
        return Err(ProofError::VerificationError {
            error: "The number of source and candidate columns should be equal",
        });
    }
    let multiplicity_eval = builder.try_consume_first_round_mle_evaluation()?;
    let fold_gadget = FoldLogExpr::new(alpha, beta);
    let c_star_eval = fold_gadget
        .verify_evaluate(builder, column_evals, chi_n_eval)?
        .0;
    let d_star_eval = fold_gadget
        .verify_evaluate(builder, candidate_evals, chi_m_eval)?
        .0;

    // sum c_star * multiplicities - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval * multiplicity_eval - d_star_eval,
        2,
    )?;

    Ok(multiplicity_eval)
}

#[cfg(test)]
mod tests {
    use super::{
        final_round_evaluate_membership_check, first_round_evaluate_membership_check,
        verify_membership_check,
    };
    use crate::{
        base::{
            database::{table_utility::*, Column},
            polynomial::MultilinearExtension,
            proof::ProofError,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{
            mock_verification_builder::{run_verify_for_each_row, MockVerificationBuilder},
            FinalRoundBuilder, FirstRoundBuilder,
        },
    };
    use alloc::{collections::VecDeque, vec, vec::Vec};
    use bumpalo::Bump;

    #[test]
    fn we_can_verify_membership_check_without_blitzar() {
        let alloc = Bump::new();
        let source_column = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
        let candidate_column =
            borrowed_bigint::<TestScalar>("candidate_a", [1, 2, 2, 1, 2], &alloc).1;
        let source_columns = [source_column];
        let candidate_columns = [candidate_column];
        let source_len = source_columns[0].len();
        let candidate_len = candidate_columns[0].len();
        let source_chi = alloc.alloc_slice_fill_copy(source_len, true) as &[bool];
        let candidate_chi = alloc.alloc_slice_fill_copy(candidate_len, true) as &[bool];

        let mut first_round_builder = FirstRoundBuilder::new(candidate_len);
        let multiplicities = first_round_evaluate_membership_check(
            &mut first_round_builder,
            &alloc,
            &source_columns,
            &candidate_columns,
        );
        assert_eq!(multiplicities, &[2, 3, 0]);
        assert_eq!(first_round_builder.pcs_proof_mles().len(), 1);

        let alpha = TestScalar::from(7);
        let beta = TestScalar::from(19);
        let mut final_round_builder = FinalRoundBuilder::new(candidate_len, VecDeque::new());
        let final_multiplicities = final_round_evaluate_membership_check(
            &mut final_round_builder,
            &alloc,
            alpha,
            beta,
            source_chi,
            candidate_chi,
            &source_columns,
            &candidate_columns,
        );
        assert_eq!(final_multiplicities, multiplicities);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 3);

        let verification_builder = run_verify_for_each_row(
            source_len,
            &first_round_builder,
            &final_round_builder,
            vec![],
            3,
            |builder, chi_n_eval, evaluation_point| {
                let chi_m_eval = candidate_chi.inner_product(evaluation_point);
                let column_evals = eval_columns(&source_columns, evaluation_point);
                let candidate_evals = eval_columns(&candidate_columns, evaluation_point);
                let multiplicity_eval = verify_membership_check(
                    builder,
                    alpha,
                    beta,
                    chi_n_eval,
                    chi_m_eval,
                    &column_evals,
                    &candidate_evals,
                )
                .unwrap();
                assert_eq!(
                    multiplicity_eval,
                    multiplicities.inner_product(evaluation_point)
                );
            },
        );

        assert!(verification_builder
            .get_zero_sum_results()
            .iter()
            .all(|result| *result));
    }

    #[test]
    fn we_can_reject_membership_check_with_mismatched_column_counts() {
        let mut builder: MockVerificationBuilder<TestScalar> = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            Vec::new(),
            vec![],
            vec![],
            vec![],
        );

        let err = verify_membership_check(
            &mut builder,
            TestScalar::from(7),
            TestScalar::from(19),
            TestScalar::ONE,
            TestScalar::ONE,
            &[TestScalar::from(1), TestScalar::from(2)],
            &[TestScalar::from(1)],
        )
        .unwrap_err();

        assert!(matches!(err, ProofError::VerificationError { .. }));
    }

    #[test]
    #[should_panic(expected = "The number of source and candidate columns should be equal")]
    fn we_cannot_first_round_membership_check_with_mismatched_column_counts() {
        let alloc = Bump::new();
        let source_columns = [
            borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1,
            borrowed_bigint::<TestScalar>("b", [4, 5, 6], &alloc).1,
        ];
        let candidate_columns = [borrowed_bigint::<TestScalar>("candidate_a", [1, 2, 3], &alloc).1];
        let mut builder = FirstRoundBuilder::new(3);

        let _ = first_round_evaluate_membership_check(
            &mut builder,
            &alloc,
            &source_columns,
            &candidate_columns,
        );
    }

    #[test]
    #[should_panic(expected = "The number of source columns should be greater than 0")]
    fn we_cannot_final_round_membership_check_without_columns() {
        let alloc = Bump::new();
        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
        let columns: [Column<'_, TestScalar>; 0] = [];
        let candidate_columns: [Column<'_, TestScalar>; 0] = [];

        let _ = final_round_evaluate_membership_check(
            &mut builder,
            &alloc,
            TestScalar::from(7),
            TestScalar::from(19),
            &[true],
            &[true],
            &columns,
            &candidate_columns,
        );
    }

    fn eval_columns(
        columns: &[Column<'_, TestScalar>],
        evaluation_point: &[TestScalar],
    ) -> Vec<TestScalar> {
        columns
            .iter()
            .map(|column| column.inner_product(evaluation_point))
            .collect()
    }
}
