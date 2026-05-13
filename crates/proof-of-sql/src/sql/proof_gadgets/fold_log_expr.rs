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
        base::{database::Column, scalar::test_scalar::TestScalar},
        sql::proof::{
            mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder,
            SumcheckSubpolynomialType,
        },
    };
    use alloc::{collections::VecDeque, vec};
    use bumpalo::Bump;
    use num_traits::{Inv, One};

    fn scalar(value: u64) -> TestScalar {
        TestScalar::from(value)
    }

    #[test]
    fn we_can_verify_fold_log_expr_evaluation() {
        let expr = FoldLogExpr::new(scalar(2), scalar(3));
        let column_evals = [scalar(4), scalar(5)];
        let star_eval = scalar(7);
        let fold_eval = scalar(34);
        let chi_eval = star_eval + fold_eval * star_eval;
        let mut builder = MockVerificationBuilder::new(
            vec![],
            3,
            vec![],
            vec![vec![star_eval]],
            vec![],
            vec![],
            vec![],
        );

        assert_eq!(
            expr.verify_evaluate(&mut builder, &column_evals, chi_eval)
                .unwrap(),
            (star_eval, fold_eval)
        );
        assert_eq!(builder.get_identity_results(), vec![vec![true]]);
        assert!(builder.get_zero_sum_results().is_empty());
    }

    #[test]
    fn we_error_if_fold_log_expr_is_missing_star_evaluation() {
        let expr = FoldLogExpr::new(scalar(2), scalar(3));
        let mut builder =
            MockVerificationBuilder::new(vec![], 3, vec![], vec![vec![]], vec![], vec![], vec![]);

        assert!(expr
            .verify_evaluate(&mut builder, &[scalar(4), scalar(5)], scalar(1))
            .is_err());
    }

    #[test]
    fn we_can_build_final_round_fold_log_expr_constraints_with_custom_chi() {
        let alloc = Bump::new();
        let expr = FoldLogExpr::new(scalar(2), scalar(3));
        let first_column = [1_i64, 2, 3];
        let second_column = [10_i64, 20, 30];
        let columns = [
            Column::<TestScalar>::BigInt(&first_column),
            Column::<TestScalar>::BigInt(&second_column),
        ];
        let chi = alloc.alloc_slice_fill_copy(3, true);
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let (star, fold) =
            expr.final_round_evaluate_with_chi(&mut builder, &alloc, &columns, 3, chi);
        let expected_fold = [scalar(26), scalar(52), scalar(78)];
        let expected_star = expected_fold.map(|value| (value + TestScalar::one()).inv().unwrap());

        assert_eq!(fold, expected_fold);
        assert_eq!(star, expected_star);
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
        assert_eq!(
            builder.sumcheck_subpolynomials()[0].subpolynomial_type(),
            SumcheckSubpolynomialType::Identity
        );
    }

    #[test]
    fn we_can_build_final_round_fold_log_expr_constraints_with_default_chi() {
        let alloc = Bump::new();
        let expr = FoldLogExpr::new(scalar(5), scalar(7));
        let column = [11_i64, 13];
        let columns = [Column::<TestScalar>::BigInt(&column)];
        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());

        let (star, fold) = expr.final_round_evaluate(&mut builder, &alloc, &columns, 2);
        let expected_fold = [scalar(55), scalar(65)];
        let expected_star = expected_fold.map(|value| (value + TestScalar::one()).inv().unwrap());

        assert_eq!(fold, expected_fold);
        assert_eq!(star, expected_star);
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }
}
