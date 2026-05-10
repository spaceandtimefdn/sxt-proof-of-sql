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
    use alloc::collections::VecDeque;
    use num_traits::{One, Zero};

    #[test]
    fn we_compute_fold_log_star_and_fold_columns() {
        let alloc = Bump::new();
        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
        let gadget = FoldLogExpr::new(TestScalar::from(3u64), TestScalar::from(10u64));
        let column = Column::BigInt(&[1_i64, 2]);

        let (star, fold) = gadget.final_round_evaluate(&mut builder, &alloc, &[column], 2);

        assert_eq!(fold, &[TestScalar::from(3u64), TestScalar::from(6u64)]);
        assert!(star
            .iter()
            .zip(fold)
            .all(
                |(&star_eval, &fold_eval)| star_eval * (fold_eval + TestScalar::one())
                    == TestScalar::one()
            ));
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }

    #[test]
    fn we_can_build_fold_log_constraints_with_a_supplied_chi_column() {
        let alloc = Bump::new();
        let mut builder = FinalRoundBuilder::new(1, VecDeque::new());
        let gadget = FoldLogExpr::new(TestScalar::from(3u64), TestScalar::from(10u64));
        let chi = alloc.alloc_slice_copy(&[true, false]);

        let (star, fold) = gadget.final_round_evaluate_with_chi(&mut builder, &alloc, &[], 2, chi);

        assert_eq!(fold, &[TestScalar::zero(), TestScalar::zero()]);
        assert_eq!(star, &[TestScalar::one(), TestScalar::one()]);
        assert_eq!(builder.pcs_proof_mles().len(), 1);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
    }
}
