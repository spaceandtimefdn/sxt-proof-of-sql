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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::collections::VecDeque;

    #[test]
    fn we_compute_membership_multiplicities_in_the_first_round() {
        let alloc = Bump::new();
        let source_values = [1_i64, 2];
        let candidate_values = [1_i64, 1, 2];
        let columns = [Column::<TestScalar>::BigInt(&source_values)];
        let candidate_subset = [Column::<TestScalar>::BigInt(&candidate_values)];
        let mut builder = FirstRoundBuilder::new(0);

        let multiplicities = first_round_evaluate_membership_check(
            &mut builder,
            &alloc,
            &columns,
            &candidate_subset,
        );

        assert_eq!(multiplicities, &[2, 1]);
        assert_eq!(builder.pcs_proof_mles().len(), 1);
    }

    #[test]
    fn we_build_membership_final_round_constraints() {
        let alloc = Bump::new();
        let source_values = [1_i64, 2];
        let candidate_values = [1_i64, 1, 2];
        let columns = [Column::<TestScalar>::BigInt(&source_values)];
        let candidate_subset = [Column::<TestScalar>::BigInt(&candidate_values)];
        let chi_n = [true, true];
        let chi_m = [true, true, true];
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let multiplicities = final_round_evaluate_membership_check(
            &mut builder,
            &alloc,
            TestScalar::from(3u64),
            TestScalar::from(10u64),
            &chi_n,
            &chi_m,
            &columns,
            &candidate_subset,
        );

        assert_eq!(multiplicities, &[2, 1]);
        assert_eq!(builder.pcs_proof_mles().len(), 2);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 3);
    }
}
