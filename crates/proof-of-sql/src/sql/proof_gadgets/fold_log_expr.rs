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
        base::{database::Column, scalar::test_scalar::TestScalar},
        sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
    };
    use alloc::{collections::VecDeque, vec, vec::Vec};
    use ark_ff::One;

    fn sample_columns<'a>(
        left: &'a [TestScalar],
        right: &'a [TestScalar],
    ) -> Vec<Column<'a, TestScalar>> {
        vec![Column::Scalar(left), Column::Scalar(right)]
    }

    fn expected_fold(left: &[TestScalar], right: &[TestScalar]) -> Vec<TestScalar> {
        left.iter()
            .zip(right)
            .map(|(&a, &b)| TestScalar::from(2) * (a * TestScalar::from(3) + b))
            .collect()
    }

    #[test]
    fn final_round_evaluate_produces_fold_star_and_identity_constraint() {
        let alloc = Bump::new();
        let left = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(4),
        ];
        let right = [
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
            TestScalar::from(40),
        ];
        let columns = sample_columns(&left, &right);
        let expr = FoldLogExpr::new(TestScalar::from(2), TestScalar::from(3));
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let (star, fold) = expr.final_round_evaluate(&mut builder, &alloc, &columns, left.len());

        assert_eq!(fold, expected_fold(&left, &right).as_slice());
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
        for (&star_value, &fold_value) in star.iter().zip(fold) {
            assert_eq!(
                (TestScalar::one() + fold_value) * star_value,
                TestScalar::one()
            );
        }
    }

    #[test]
    fn verify_evaluate_consumes_star_and_records_matching_identity_evaluations() {
        let alloc = Bump::new();
        let left = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(4),
        ];
        let right = [
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
            TestScalar::from(40),
        ];
        let columns = sample_columns(&left, &right);
        let expr = FoldLogExpr::new(TestScalar::from(2), TestScalar::from(3));
        let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
        let (star, fold) =
            expr.final_round_evaluate(&mut final_round_builder, &alloc, &columns, left.len());
        let final_round_mles = star.iter().map(|&value| vec![value]).collect();
        let mut verification_builder = MockVerificationBuilder::new(
            Vec::new(),
            3,
            Vec::new(),
            final_round_mles,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        for row in 0..left.len() {
            let (star_eval, fold_eval) = expr
                .verify_evaluate(
                    &mut verification_builder,
                    &[left[row], right[row]],
                    TestScalar::one(),
                )
                .unwrap();
            assert_eq!(star_eval, star[row]);
            assert_eq!(fold_eval, fold[row]);
            verification_builder.increment_row_index();
        }

        assert_eq!(
            verification_builder.get_identity_results(),
            vec![vec![true], vec![true], vec![true], vec![true]]
        );
    }
}
