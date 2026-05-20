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
    use crate::{
        base::{
            database::Column,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
    };
    use alloc::{collections::VecDeque, vec, vec::Vec};
    use num_traits::Inv;

    #[test]
    fn final_round_evaluate_builds_star_for_folded_columns() {
        let alloc = Bump::new();
        let alpha = TestScalar::from(2);
        let beta = TestScalar::from(3);
        let lhs = [1_i64, 2, 3];
        let rhs = [10_i64, 20, 30];
        let columns: [Column<'_, TestScalar>; 2] = [Column::BigInt(&lhs), Column::BigInt(&rhs)];
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let (star, fold) =
            FoldLogExpr::new(alpha, beta).final_round_evaluate(&mut builder, &alloc, &columns, 3);

        let expected_fold = [26_i64, 52, 78].map(TestScalar::from);
        let expected_star = expected_fold.map(|value| (TestScalar::ONE + value).inv().unwrap());
        assert_eq!(fold, expected_fold.as_slice());
        assert_eq!(star, expected_star.as_slice());
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
        assert_eq!(
            builder.evaluate_pcs_proof_mles(&[TestScalar::ONE, TestScalar::ZERO, TestScalar::ZERO]),
            vec![expected_star[0]]
        );
    }

    #[test]
    fn verify_evaluate_records_the_fold_identity_constraint() {
        let alpha = TestScalar::from(2);
        let beta = TestScalar::from(3);
        let column_evals = [TestScalar::from(4), TestScalar::from(5)];
        let fold_eval = TestScalar::from(34);
        let star_eval = (TestScalar::ONE + fold_eval).inv().unwrap();
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            vec![vec![star_eval]],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let (actual_star_eval, actual_fold_eval) = FoldLogExpr::new(alpha, beta)
            .verify_evaluate(&mut builder, &column_evals, TestScalar::ONE)
            .unwrap();

        assert_eq!(actual_star_eval, star_eval);
        assert_eq!(actual_fold_eval, fold_eval);
        assert_eq!(
            builder.identity_subpolynomial_evaluations,
            vec![vec![TestScalar::ZERO]]
        );
    }

    #[test]
    fn verify_evaluate_errors_when_the_star_evaluation_is_missing() {
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            Vec::new(),
            3,
            Vec::new(),
            vec![Vec::new()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let result = FoldLogExpr::new(TestScalar::from(2), TestScalar::from(3)).verify_evaluate(
            &mut builder,
            &[TestScalar::from(4), TestScalar::from(5)],
            TestScalar::ONE,
        );

        assert!(result.is_err());
        assert!(builder.identity_subpolynomial_evaluations.is_empty());
    }

    #[test]
    fn verify_evaluate_errors_when_the_identity_constraint_is_too_large() {
        let fold_eval = TestScalar::from(34);
        let star_eval = (TestScalar::ONE + fold_eval).inv().unwrap();
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            Vec::new(),
            2,
            Vec::new(),
            vec![vec![star_eval]],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let result = FoldLogExpr::new(TestScalar::from(2), TestScalar::from(3)).verify_evaluate(
            &mut builder,
            &[TestScalar::from(4), TestScalar::from(5)],
            TestScalar::ONE,
        );

        assert!(result.is_err());
        assert_eq!(builder.identity_subpolynomial_evaluations, vec![Vec::new()]);
    }
}
