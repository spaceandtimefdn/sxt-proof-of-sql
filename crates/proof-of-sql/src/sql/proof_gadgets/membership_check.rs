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
    use super::verify_membership_check;
    use crate::{
        base::{
            proof::ProofError,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };

    #[test]
    fn verifier_accepts_matching_folded_membership_evaluations() {
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            vec![vec![TestScalar::ONE]],
            vec![vec![TestScalar::ONE, TestScalar::ONE]],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let multiplicity_eval = verify_membership_check(
            &mut builder,
            TestScalar::ONE,
            TestScalar::TEN,
            TestScalar::ONE,
            TestScalar::ONE,
            &[TestScalar::ZERO],
            &[TestScalar::ZERO],
        )
        .unwrap();

        assert_eq!(multiplicity_eval, TestScalar::ONE);
        assert_eq!(builder.get_identity_results(), vec![vec![true, true]]);
        assert_eq!(builder.get_zero_sum_results(), vec![true]);
    }

    #[test]
    fn verifier_rejects_mismatched_source_and_candidate_column_counts() {
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            vec![Vec::new()],
            vec![Vec::new()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let error = verify_membership_check(
            &mut builder,
            TestScalar::ONE,
            TestScalar::TEN,
            TestScalar::ONE,
            TestScalar::ONE,
            &[TestScalar::ZERO],
            &[TestScalar::ZERO, TestScalar::ONE],
        )
        .unwrap_err();

        assert!(matches!(
            error,
            ProofError::VerificationError {
                error: "The number of source and candidate columns should be equal"
            }
        ));
    }
}
