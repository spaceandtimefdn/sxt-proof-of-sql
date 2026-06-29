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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use crate::sql::proof::mock_verification_builder::MockVerificationBuilder;
    use alloc::collections::VecDeque;
    use num_traits::Inv;

    #[test]
    fn verifier_consumes_star_and_records_identity_constraint() {
        let fold = FoldLogExpr::new(TestScalar::from(2), TestScalar::from(3));
        let mut builder = MockVerificationBuilder::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(5)]],
            vec![],
            vec![],
            vec![],
        );

        let (star_eval, fold_eval) = fold
            .verify_evaluate(
                &mut builder,
                &[TestScalar::from(7), TestScalar::from(11)],
                TestScalar::from(97),
            )
            .unwrap();

        assert_eq!(star_eval, TestScalar::from(5));
        assert_eq!(fold_eval, TestScalar::from(64));
        assert_eq!(
            builder.identity_subpolynomial_evaluations,
            vec![vec![TestScalar::from(5 + 64 * 5 - 97)]]
        );
    }

    #[test]
    fn final_round_evaluate_produces_star_and_fold_mles() {
        let alloc = Bump::new();
        let fold = FoldLogExpr::new(TestScalar::from(2), TestScalar::from(3));
        let columns = [
            Column::Scalar(&[TestScalar::from(7), TestScalar::from(13)]),
            Column::Scalar(&[TestScalar::from(11), TestScalar::from(17)]),
        ];
        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());

        let (star, folded) = fold.final_round_evaluate(&mut builder, &alloc, &columns, 2);

        assert_eq!(folded, &[TestScalar::from(64), TestScalar::from(112)]);
        assert_eq!(
            star,
            &[
                TestScalar::from(65).inv().unwrap(),
                TestScalar::from(113).inv().unwrap()
            ]
        );
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }
}
