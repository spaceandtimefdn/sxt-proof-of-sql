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
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn new_stores_alpha_and_beta() {
        let expr: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(2u64),
            TestScalar::from(3u64),
        );
        assert_eq!(expr.alpha, TestScalar::from(2u64));
        assert_eq!(expr.beta, TestScalar::from(3u64));
    }

    #[test]
    fn clone_creates_equal_instance() {
        let expr: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(5u64),
            TestScalar::from(7u64),
        );
        assert_eq!(expr.clone(), expr);
    }

    #[test]
    fn two_fold_log_exprs_with_same_params_are_equal() {
        let a: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(1u64),
            TestScalar::from(2u64),
        );
        let b: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(1u64),
            TestScalar::from(2u64),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn two_fold_log_exprs_with_different_beta_are_not_equal() {
        let a: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(1u64),
            TestScalar::from(2u64),
        );
        let b: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(1u64),
            TestScalar::from(9u64),
        );
        assert_ne!(a, b);
    }

    #[test]
    fn debug_output_contains_struct_name() {
        let expr: FoldLogExpr<TestScalar> = FoldLogExpr::new(
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        );
        let debug = format!("{:?}", expr);
        assert!(debug.contains("FoldLogExpr"));
    }
}
