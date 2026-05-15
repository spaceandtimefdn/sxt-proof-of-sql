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
            database::Column,
            proof::{ProofError, ProofSizeMismatch},
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::{
            proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
            proof_plans::fold_vals,
        },
    };
    use bumpalo::Bump;
    use std::collections::VecDeque;

    #[test]
    fn we_can_verify_fold_log_constraint() {
        let expr = FoldLogExpr::new(TestScalar::from(2u64), TestScalar::from(10u64));
        let column_evals = [TestScalar::from(3u64), TestScalar::from(4u64)];
        let expected_fold_eval =
            TestScalar::from(2u64) * fold_vals(TestScalar::from(10u64), &column_evals);
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            vec![vec![TestScalar::ONE]],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let (star_eval, fold_eval) = expr
            .verify_evaluate(
                &mut builder,
                &column_evals,
                expected_fold_eval + TestScalar::ONE,
            )
            .unwrap();

        assert_eq!(star_eval, TestScalar::ONE);
        assert_eq!(fold_eval, expected_fold_eval);
        assert_eq!(builder.get_identity_results(), vec![vec![true]]);
    }

    #[test]
    fn verifier_errors_when_fold_log_has_too_few_final_round_mles() {
        let expr = FoldLogExpr::new(TestScalar::from(2u64), TestScalar::from(10u64));
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            vec![Vec::new()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let error = expr
            .verify_evaluate(&mut builder, &[TestScalar::ONE], TestScalar::ONE)
            .unwrap_err();

        assert!(matches!(
            error,
            ProofError::ProofSizeMismatch {
                source: ProofSizeMismatch::TooFewMLEEvaluations
            }
        ));
    }

    #[test]
    fn final_round_evaluate_with_chi_returns_fold_and_inverse_star_columns() {
        let alloc = Bump::new();
        let expr = FoldLogExpr::new(TestScalar::from(2u64), TestScalar::from(10u64));
        let columns = [
            Column::Int128::<TestScalar>(&[1, 2]),
            Column::Int128::<TestScalar>(&[3, 4]),
        ];
        let chi = &[true, true];
        let mut builder = FinalRoundBuilder::new(columns.len(), VecDeque::new());

        let (star, fold) =
            expr.final_round_evaluate_with_chi(&mut builder, &alloc, &columns, 2, chi);

        assert_eq!(fold, &[TestScalar::from(26u64), TestScalar::from(48u64)]);
        assert!(star
            .iter()
            .zip(fold.iter())
            .all(
                |(star_eval, fold_eval)| *star_eval * (*fold_eval + TestScalar::ONE)
                    == TestScalar::ONE
            ));
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }

    #[test]
    fn final_round_evaluate_uses_true_chi_column() {
        let alloc = Bump::new();
        let expr = FoldLogExpr::new(TestScalar::from(2u64), TestScalar::from(10u64));
        let columns = [Column::Int128::<TestScalar>(&[1, 2])];
        let mut builder = FinalRoundBuilder::new(columns.len(), VecDeque::new());

        let (star, fold) = expr.final_round_evaluate(&mut builder, &alloc, &columns, 2);

        assert_eq!(fold, &[TestScalar::from(2u64), TestScalar::from(4u64)]);
        assert_eq!(star.len(), 2);
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }
}
