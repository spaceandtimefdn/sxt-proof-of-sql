use crate::{
    base::{database::Column, proof::ProofError, scalar::Scalar, slice_ops},
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_plans::{fold_columns, fold_vals},
    },
};
use alloc::{boxed::Box, vec};
use ark_ff::{One, Zero};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct FoldLogExpr<S: Scalar> {
    alpha: S,
    beta: S,
}

impl<S: Scalar> FoldLogExpr<S> {
    pub fn new(alpha: S, beta: S) -> Self {
        Self { alpha, beta }
    }

    pub fn verify_evaluate(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        column_evals: &[S],
        chi_eval: S,
    ) -> Result<(S, S), ProofError> {
        let fold_eval = self.alpha * fold_vals(self.beta, column_evals);
        let star_eval = builder.try_consume_final_round_mle_evaluation()?;
        // star + fold * star - chi = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            star_eval + fold_eval * star_eval - chi_eval,
            2,
        )?;
        Ok((star_eval, fold_eval))
    }

    #[tracing::instrument(
        name = "FoldLogExpr::final_round_evaluate_with_chi",
        level = "debug",
        skip_all
    )]
    pub fn final_round_evaluate_with_chi<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        columns: &[Column<S>],
        length: usize,
        chi: &'a [bool],
    ) -> (&'a [S], &'a [S]) {
        let fold = alloc.alloc_slice_fill_copy(length, Zero::zero());
        fold_columns(fold, self.alpha, self.beta, columns);
        let star = alloc.alloc_slice_copy(fold);
        slice_ops::add_const::<S, S>(star, One::one());
        slice_ops::batch_inversion(star);
        builder.produce_intermediate_mle(star as &[_]);
        // star + fold * star - chi = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(star as &[_])]),
                (
                    S::one(),
                    vec![Box::new(star as &[_]), Box::new(fold as &[_])],
                ),
                (-S::one(), vec![Box::new(chi as &[_])]),
            ],
        );
        (star, fold)
    }

    pub fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        columns: &[Column<S>],
        length: usize,
    ) -> (&'a [S], &'a [S]) {
        let chi = alloc.alloc_slice_fill_copy(length, true);
        self.final_round_evaluate_with_chi(builder, alloc, columns, length, chi)
    }
}

#[cfg(test)]
mod tests {
    use super::FoldLogExpr;
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
            proof_plans::fold_vals,
        },
    };
    use bumpalo::Bump;
    use core::convert::identity;
    use std::collections::VecDeque;

    #[test]
    fn we_can_verify_fold_log_expr_constraints() {
        let alloc = Bump::new();
        let alpha = TestScalar::TWO;
        let beta = TestScalar::TEN;
        let expr = FoldLogExpr::new(alpha, beta);
        let lhs = borrowed_bigint::<TestScalar>("lhs", [1, 2, 3], &alloc).1;
        let rhs = borrowed_bigint::<TestScalar>("rhs", [4, 5, 6], &alloc).1;

        let first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(3);
        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(3, VecDeque::new());
        let (star, fold) = expr.final_round_evaluate(
            &mut final_round_builder,
            &alloc,
            &[lhs.clone(), rhs.clone()],
            3,
        );

        assert_eq!(star.len(), 3);
        assert_eq!(fold.len(), 3);
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 1);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 1);

        let verification_builder = run_verify_for_each_row(
            3,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            3,
            |verification_builder, chi_eval, evaluation_point| {
                let column_evals = [
                    lhs.inner_product(evaluation_point),
                    rhs.inner_product(evaluation_point),
                ];
                let (star_eval, fold_eval) = expr
                    .verify_evaluate(verification_builder, &column_evals, chi_eval)
                    .unwrap();
                assert_eq!(fold_eval, alpha * fold_vals(beta, &column_evals));
                assert_eq!(star_eval, star.inner_product(evaluation_point));
            },
        );

        assert!(verification_builder
            .get_identity_results()
            .iter()
            .all(|row| row.iter().copied().all(identity)));
    }

    #[test]
    fn we_can_verify_fold_log_expr_with_custom_chi() {
        let alloc = Bump::new();
        let alpha = TestScalar::from(3);
        let beta = TestScalar::TWO;
        let expr = FoldLogExpr::new(alpha, beta);
        let column = borrowed_bigint::<TestScalar>("column", [7, 8, 9], &alloc).1;
        let chi = &[true, true, true];

        let first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(3);
        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(3, VecDeque::new());
        expr.final_round_evaluate_with_chi(
            &mut final_round_builder,
            &alloc,
            &[column.clone()],
            3,
            chi,
        );

        let verification_builder = run_verify_for_each_row(
            3,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            3,
            |verification_builder, _chi_eval, evaluation_point| {
                expr.verify_evaluate(
                    verification_builder,
                    &[column.inner_product(evaluation_point)],
                    chi.inner_product(evaluation_point),
                )
                .unwrap();
            },
        );

        assert!(verification_builder
            .get_identity_results()
            .iter()
            .all(|row| row.iter().copied().all(identity)));
    }
}
