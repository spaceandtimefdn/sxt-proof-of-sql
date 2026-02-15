use crate::{
    base::{
        database::{apply_column_to_indexes, Column},
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
use itertools::Itertools;

/// Perform first round evaluation of the permutation check.
///
/// # Panics
/// Panics if the number of columns is zero.
/// Panics if the relevant columns do not all have the same length.
pub(crate) fn first_round_evaluate_permutation_check<'a, S: Scalar>(
    builder: &mut FirstRoundBuilder<'a, S>,
    alloc: &'a Bump,
    chi: &'a [bool],
    columns: &[Column<'a, S>],
    permutation: &[usize],
) -> Vec<Column<'a, S>> {
    assert!(
        !columns.is_empty(),
        "The number of columns should be greater than 0"
    );
    let table_length = chi.len();
    builder.produce_rho_evaluation_length(table_length);
    assert!(
        core::iter::once(table_length)
            .chain(columns.iter().map(Column::len))
            .chain(core::iter::once(permutation.len()))
            .all_equal(),
        "The length of all relevant columns should be the same"
    );
    let rho = Column::<S>::rho(table_length, alloc);
    let columns_with_rho = columns.iter().copied().chain(core::iter::once(rho));
    let mut permuted_columns_with_rho = columns_with_rho.clone().map(|column| {
        apply_column_to_indexes(&column, alloc, permutation)
            .expect("Permutation confirmed to be valid at this point")
    }).collect::<Vec<_>>();
    for column in permuted_columns_with_rho.clone() {
        builder.produce_intermediate_mle(column);
    }

    permuted_columns_with_rho.pop().expect("permuted_column_evals should have at least one element");

    permuted_columns_with_rho
}

/// Perform final round evaluation of the permutation check.
///
/// # Panics
/// Panics if the number of columns is zero.
/// Panics if the relevant columns do not all have the same length.
pub(crate) fn final_round_evaluate_permutation_check<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    chi: &'a [bool],
    columns: &[Column<'a, S>],
    permutation: &[usize],
) -> Vec<Column<'a, S>> {
    assert!(
        !columns.is_empty(),
        "The number of columns should be greater than 0"
    );
    let table_length = chi.len();
    assert!(
        core::iter::once(table_length)
            .chain(columns.iter().map(Column::len))
            .chain(core::iter::once(permutation.len()))
            .all_equal(),
        "The length of all relevant columns should be the same"
    );
    let rho = Column::<S>::rho(table_length, alloc);
    let columns_with_rho = columns.iter().copied().chain(core::iter::once(rho));
    let mut permuted_columns_with_rho = columns_with_rho
        .clone()
        .map(|column| {
            apply_column_to_indexes(&column, alloc, permutation)
                .expect("Permutation confirmed to be valid at this point")
        })
        .collect::<Vec<_>>();

    let fold_gadget = FoldLogExpr::new(alpha, beta);
    let (c_star, _) = fold_gadget.final_round_evaluate_with_chi(
        builder,
        alloc,
        &columns_with_rho.collect::<Vec<_>>(),
        table_length,
        chi,
    );
    let (d_star, _) = fold_gadget.final_round_evaluate_with_chi(
        builder,
        alloc,
        &permuted_columns_with_rho,
        table_length,
        chi,
    );

    // sum c_star - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );

    permuted_columns_with_rho
        .pop()
        .expect("permuted_column_evals should have at least one element");

    permuted_columns_with_rho
}

#[expect(clippy::missing_panics_doc)]
pub(crate) fn verify_permutation_check<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    chi_eval: S,
    column_evals: &[S],
) -> Result<Vec<S>, ProofError> {
    let column_evals: Vec<_> = column_evals
        .iter()
        .copied()
        .chain(core::iter::once(builder.try_consume_rho_evaluation()?))
        .collect();
    let mut permuted_column_evals =
        builder.try_consume_first_round_mle_evaluations(column_evals.len())?;

    let fold_gadget = FoldLogExpr::new(alpha, beta);
    let c_star_eval = fold_gadget
        .verify_evaluate(builder, &column_evals, chi_eval)?
        .0;
    let d_star_eval = fold_gadget
        .verify_evaluate(builder, &permuted_column_evals, chi_eval)?
        .0;

    // sum c_star - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval - d_star_eval,
        1,
    )?;

    permuted_column_evals
        .pop()
        .expect("permuted_column_evals confirmed to have at least one element");

    Ok(permuted_column_evals)
}

#[cfg(test)]
mod tests {
    use super::{final_round_evaluate_permutation_check, verify_permutation_check};
    use crate::{
        base::{
            database::table_utility::borrowed_bigint,
            polynomial::MultilinearExtension,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::{
            proof::{
                mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder,
                FirstRoundBuilder,
            },
            proof_gadgets::permutation_check::first_round_evaluate_permutation_check,
        },
    };
    use bumpalo::Bump;
    use std::collections::VecDeque;

    #[test]
    fn we_can_do_permutation_check() {
        let alloc = Bump::new();
        let column = borrowed_bigint::<TestScalar>("a", [1, 2, 3], &alloc).1;
        let permutation = vec![1usize, 2, 0];
        let mut first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(3);
        let mut final_round_builder: FinalRoundBuilder<TestScalar> =
            FinalRoundBuilder::new(3, VecDeque::new());
        first_round_evaluate_permutation_check(
            &mut first_round_builder,
            &alloc,
            &[true, true, true],
            &[column],
            &permutation,
        );
        final_round_evaluate_permutation_check(
            &mut final_round_builder,
            &alloc,
            TestScalar::TWO,
            TestScalar::TEN,
            &[true, true, true],
            &[column],
            &permutation,
        );
        let verification_builder = run_verify_for_each_row(
            3,
            &first_round_builder,
            &final_round_builder,
            vec![TestScalar::TWO, TestScalar::TEN],
            3,
            |verification_builder, chi_eval, evaluation_point| {
                verify_permutation_check(
                    verification_builder,
                    TestScalar::TWO,
                    TestScalar::TEN,
                    chi_eval,
                    &[column.inner_product(evaluation_point)],
                )
                .unwrap();
            },
        );
        assert!(verification_builder
            .get_identity_results()
            .iter()
            .all(|v| v.iter().all(|val| *val)));
        assert!(verification_builder
            .get_zero_sum_results()
            .iter()
            .all(|v| *v));
    }
}
