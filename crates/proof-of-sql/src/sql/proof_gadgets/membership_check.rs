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
            database::table_utility::{borrowed_bigint, borrowed_boolean},
            polynomial::MultilinearExtension,
            proof::ProofError,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{
            mock_verification_builder::{run_verify_for_each_row, MockVerificationBuilder},
            FinalRoundBuilder, FirstRoundBuilder,
        },
    };
    use bumpalo::Bump;
    use core::convert::identity;
    use std::collections::VecDeque;

    #[test]
    fn we_can_evaluate_and_verify_membership_check_constraints() {
        let alloc = Bump::new();
        let source_id = borrowed_bigint::<TestScalar>("source_id", [1, 2, 3], &alloc).1;
        let source_flag =
            borrowed_boolean::<TestScalar>("source_flag", [true, false, true], &alloc).1;
        let candidate_id = borrowed_bigint::<TestScalar>("candidate_id", [2, 1, 2], &alloc).1;
        let candidate_flag =
            borrowed_boolean::<TestScalar>("candidate_flag", [false, true, false], &alloc).1;
        let source_columns = &[source_id.clone(), source_flag.clone()];
        let candidate_columns = &[candidate_id.clone(), candidate_flag.clone()];
        let chi = &[true, true, true];
        let alpha = TestScalar::TWO;
        let beta = TestScalar::TEN;

        let mut first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(3);
        let first_round_multiplicities = first_round_evaluate_membership_check(
            &mut first_round_builder,
            &alloc,
            source_columns,
            candidate_columns,
        );
        assert_eq!(first_round_multiplicities, [1, 2, 0]);

        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(3, VecDeque::new());
        let final_round_multiplicities = final_round_evaluate_membership_check(
            &mut final_round_builder,
            &alloc,
            alpha,
            beta,
            chi,
            chi,
            source_columns,
            candidate_columns,
        );
        assert_eq!(final_round_multiplicities, [1, 2, 0]);
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 3);

        let verification_builder = run_verify_for_each_row(
            3,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            3,
            |verification_builder, chi_eval, evaluation_point| {
                let source_evals = [
                    source_id.inner_product(evaluation_point),
                    source_flag.inner_product(evaluation_point),
                ];
                let candidate_evals = [
                    candidate_id.inner_product(evaluation_point),
                    candidate_flag.inner_product(evaluation_point),
                ];
                let multiplicity_eval = verify_membership_check(
                    verification_builder,
                    alpha,
                    beta,
                    chi_eval,
                    chi_eval,
                    &source_evals,
                    &candidate_evals,
                )
                .unwrap();
                assert_eq!(
                    multiplicity_eval,
                    first_round_multiplicities.inner_product(evaluation_point)
                );
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

    #[test]
    #[should_panic(expected = "The number of source and candidate columns should be equal")]
    fn first_round_panics_if_column_counts_differ() {
        let alloc = Bump::new();
        let source_id = borrowed_bigint::<TestScalar>("source_id", [1, 2], &alloc).1;
        let candidate_id = borrowed_bigint::<TestScalar>("candidate_id", [1, 2], &alloc).1;
        let candidate_flag =
            borrowed_boolean::<TestScalar>("candidate_flag", [true, false], &alloc).1;
        let mut builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(2);

        first_round_evaluate_membership_check(
            &mut builder,
            &alloc,
            &[source_id],
            &[candidate_id, candidate_flag],
        );
    }

    #[test]
    #[should_panic(expected = "The number of source columns should be greater than 0")]
    fn final_round_panics_if_there_are_no_source_columns() {
        let alloc = Bump::new();
        let mut builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(1, VecDeque::new());

        final_round_evaluate_membership_check(
            &mut builder,
            &alloc,
            TestScalar::ONE,
            TestScalar::TWO,
            &[true],
            &[true],
            &[],
            &[],
        );
    }

    #[test]
    fn verify_membership_check_rejects_column_count_mismatch() {
        let mut builder: MockVerificationBuilder<TestScalar> = MockVerificationBuilder::new(
            Vec::new(),
            2,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let result = verify_membership_check(
            &mut builder,
            TestScalar::ONE,
            TestScalar::TWO,
            TestScalar::ONE,
            TestScalar::ONE,
            &[TestScalar::ONE],
            &[],
        );

        assert!(matches!(
            result,
            Err(ProofError::VerificationError {
                error: "The number of source and candidate columns should be equal"
            })
        ));
    }
}
