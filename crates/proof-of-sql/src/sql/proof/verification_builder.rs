use super::{SumcheckMleEvaluations, SumcheckSubpolynomialType};
use crate::base::{bit::BitDistribution, proof::ProofSizeMismatch, scalar::Scalar};
use alloc::{collections::VecDeque, vec::Vec};
use core::iter;

pub trait VerificationBuilder<S: Scalar> {
    /// Consume the evaluation of a chi evaluation
    fn try_consume_chi_evaluation(&mut self) -> Result<(S, usize), ProofSizeMismatch>;

    /// Consume the evaluation of a rho evaluation
    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume the evaluation of a first round MLE used in sumcheck and provide the commitment of the MLE
    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume multiple first round MLE evaluations
    fn try_consume_first_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch>;

    /// Consume the evaluation of a final round MLE used in sumcheck and provide the commitment of the MLE
    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Consume multiple final round MLE evaluations
    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch>;

    /// Consume a bit distribution that describes which bits are constant
    /// and which bits varying in a column of data
    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch>;

    /// Produce the evaluation of a subpolynomial used in sumcheck
    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
        degree: usize,
    ) -> Result<(), ProofSizeMismatch>;

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch>;

    /// Retrieves the `singleton_chi_evaluation` from the `mle_evaluations`
    fn singleton_chi_evaluation(&self) -> S;

    /// Retrieves the `rho_256_evaluation` from the `mle_evaluations`
    fn rho_256_evaluation(&self) -> Option<S>;
}

/// Track components used to verify a query's proof
pub struct VerificationBuilderImpl<'a, S: Scalar> {
    mle_evaluations: SumcheckMleEvaluations<'a, S>,
    subpolynomial_multipliers: &'a [S],
    sumcheck_evaluation: S,
    bit_distributions: &'a [BitDistribution],
    consumed_chi_evaluations: usize,
    consumed_rho_evaluations: usize,
    consumed_first_round_pcs_proof_mles: usize,
    consumed_final_round_pcs_proof_mles: usize,
    produced_subpolynomials: usize,
    /// The challenges used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this vector is treated as a stack and the first
    /// challenge is the last entry in the vector.
    post_result_challenges: VecDeque<S>,
    chi_evaluation_length_queue: Vec<usize>,
    rho_evaluation_length_queue: Vec<usize>,
    subpolynomial_max_multiplicands: usize,
}

impl<'a, S: Scalar> VerificationBuilderImpl<'a, S> {
    pub fn new(
        mle_evaluations: SumcheckMleEvaluations<'a, S>,
        bit_distributions: &'a [BitDistribution],
        subpolynomial_multipliers: &'a [S],
        post_result_challenges: VecDeque<S>,
        chi_evaluation_length_queue: Vec<usize>,
        rho_evaluation_length_queue: Vec<usize>,
        subpolynomial_max_multiplicands: usize,
    ) -> Self {
        Self {
            mle_evaluations,
            bit_distributions,
            subpolynomial_multipliers,
            sumcheck_evaluation: S::zero(),
            consumed_chi_evaluations: 0,
            consumed_rho_evaluations: 0,
            consumed_first_round_pcs_proof_mles: 0,
            consumed_final_round_pcs_proof_mles: 0,
            produced_subpolynomials: 0,
            post_result_challenges,
            chi_evaluation_length_queue,
            rho_evaluation_length_queue,
            subpolynomial_max_multiplicands,
        }
    }

    #[expect(
        clippy::missing_panics_doc,
        reason = "The panic condition is clear due to the assertion that checks if the computation is completed."
    )]
    /// Get the evaluation of the sumcheck polynomial at its randomly selected point
    pub fn sumcheck_evaluation(&self) -> S {
        assert!(self.completed());
        self.sumcheck_evaluation
    }

    /// Check that the verification builder is completely built up
    fn completed(&self) -> bool {
        self.bit_distributions.is_empty()
            && self.produced_subpolynomials == self.subpolynomial_multipliers.len()
            && self.consumed_first_round_pcs_proof_mles
                == self.mle_evaluations.first_round_pcs_proof_evaluations.len()
            && self.consumed_final_round_pcs_proof_mles
                == self.mle_evaluations.final_round_pcs_proof_evaluations.len()
            && self.post_result_challenges.is_empty()
    }
}

impl<S: Scalar> VerificationBuilder<S> for VerificationBuilderImpl<'_, S> {
    fn try_consume_chi_evaluation(&mut self) -> Result<(S, usize), ProofSizeMismatch> {
        let index = self.consumed_chi_evaluations;
        let length = self
            .chi_evaluation_length_queue
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewChiLengths)?;
        self.consumed_chi_evaluations += 1;
        Ok((
            *self
                .mle_evaluations
                .chi_evaluations
                .get(&length)
                .ok_or(ProofSizeMismatch::ChiLengthNotFound)?,
            length,
        ))
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_rho_evaluations;
        let length = self
            .rho_evaluation_length_queue
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewRhoLengths)?;
        self.consumed_rho_evaluations += 1;
        Ok(*self
            .mle_evaluations
            .rho_evaluations
            .get(&length)
            .ok_or(ProofSizeMismatch::RhoLengthNotFound)?)
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_first_round_pcs_proof_mles;
        self.consumed_first_round_pcs_proof_mles += 1;
        self.mle_evaluations
            .first_round_pcs_proof_evaluations
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_first_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        iter::repeat_with(|| self.try_consume_first_round_mle_evaluation())
            .take(count)
            .collect()
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        let index = self.consumed_final_round_pcs_proof_mles;
        self.consumed_final_round_pcs_proof_mles += 1;
        self.mle_evaluations
            .final_round_pcs_proof_evaluations
            .get(index)
            .copied()
            .ok_or(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        iter::repeat_with(|| self.try_consume_final_round_mle_evaluation())
            .take(count)
            .collect()
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        let res = self
            .bit_distributions
            .first()
            .cloned()
            .ok_or(ProofSizeMismatch::TooFewBitDistributions)?;
        self.bit_distributions = &self.bit_distributions[1..];
        Ok(res)
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        eval: S,
        degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        self.sumcheck_evaluation += self
            .subpolynomial_multipliers
            .get(self.produced_subpolynomials)
            .copied()
            .ok_or(ProofSizeMismatch::ConstraintCountMismatch)?
            * match subpolynomial_type {
                SumcheckSubpolynomialType::Identity => {
                    if degree + 1 > self.subpolynomial_max_multiplicands {
                        Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                    }
                    eval * self.mle_evaluations.random_evaluation
                }
                SumcheckSubpolynomialType::ZeroSum => {
                    if degree > self.subpolynomial_max_multiplicands {
                        Err(ProofSizeMismatch::SumcheckProofTooSmall)?;
                    }
                    eval
                }
            };
        self.produced_subpolynomials += 1;
        Ok(())
    }

    /// # Panics
    /// This function will panic if there are no post-result challenges available to pop from the stack.
    ///
    /// # Panics
    /// This function will panic if `post_result_challenges` is empty,
    /// as it attempts to pop an element from the vector and unwraps the result.
    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        self.post_result_challenges
            .pop_front()
            .ok_or(ProofSizeMismatch::PostResultCountMismatch)
    }

    fn singleton_chi_evaluation(&self) -> S {
        self.mle_evaluations.singleton_chi_evaluation
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        self.mle_evaluations.rho_256_evaluation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{map::indexmap, proof::ProofSizeMismatch, scalar::test_scalar::TestScalar};
    use alloc::{collections::VecDeque, vec};

    fn scalar(value: u64) -> TestScalar {
        TestScalar::from(value)
    }

    #[test]
    fn we_can_consume_chi_and_rho_evaluations_by_requested_length() {
        let first_round_evals = [scalar(11), scalar(12)];
        let final_round_evals = [scalar(21), scalar(22), scalar(23)];
        let bit_distributions = [BitDistribution::new::<TestScalar, _>(&[1_i64, -2, 3])];
        let subpolynomial_multipliers = [scalar(3), scalar(5)];
        let mle_evaluations = SumcheckMleEvaluations {
            chi_evaluations: indexmap! {
                2 => scalar(102),
                4 => scalar(104),
            },
            rho_evaluations: indexmap! {
                3 => scalar(203),
                5 => scalar(205),
            },
            singleton_chi_evaluation: scalar(1),
            random_evaluation: scalar(7),
            first_round_pcs_proof_evaluations: &first_round_evals,
            final_round_pcs_proof_evaluations: &final_round_evals,
            rho_256_evaluation: Some(scalar(256)),
        };
        let mut builder = VerificationBuilderImpl::new(
            mle_evaluations,
            &bit_distributions,
            &subpolynomial_multipliers,
            VecDeque::from([scalar(31), scalar(32)]),
            vec![2, 4],
            vec![3, 5],
            3,
        );

        assert_eq!(
            builder.try_consume_chi_evaluation().unwrap(),
            (scalar(102), 2)
        );
        assert_eq!(
            builder.try_consume_chi_evaluation().unwrap(),
            (scalar(104), 4)
        );
        assert!(matches!(
            builder.try_consume_chi_evaluation(),
            Err(ProofSizeMismatch::TooFewChiLengths)
        ));

        assert_eq!(builder.try_consume_rho_evaluation().unwrap(), scalar(203));
        assert_eq!(builder.try_consume_rho_evaluation().unwrap(), scalar(205));
        assert!(matches!(
            builder.try_consume_rho_evaluation(),
            Err(ProofSizeMismatch::TooFewRhoLengths)
        ));

        assert_eq!(
            builder.try_consume_first_round_mle_evaluations(2).unwrap(),
            first_round_evals
        );

        assert_eq!(
            builder.try_consume_final_round_mle_evaluations(2).unwrap(),
            &final_round_evals[..2]
        );
        assert_eq!(
            builder.try_consume_final_round_mle_evaluation().unwrap(),
            scalar(23)
        );

        assert_eq!(
            builder.try_consume_bit_distribution().unwrap(),
            bit_distributions[0]
        );
        assert!(matches!(
            builder.try_consume_bit_distribution(),
            Err(ProofSizeMismatch::TooFewBitDistributions)
        ));

        builder
            .try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                scalar(2),
                2,
            )
            .unwrap();
        builder
            .try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::ZeroSum,
                scalar(4),
                3,
            )
            .unwrap();

        assert_eq!(
            builder.try_consume_post_result_challenge().unwrap(),
            scalar(31)
        );
        assert_eq!(
            builder.try_consume_post_result_challenge().unwrap(),
            scalar(32)
        );
        assert!(matches!(
            builder.try_consume_post_result_challenge(),
            Err(ProofSizeMismatch::PostResultCountMismatch)
        ));
        assert_eq!(builder.singleton_chi_evaluation(), scalar(1));
        assert_eq!(builder.rho_256_evaluation(), Some(scalar(256)));

        let expected = scalar(3) * scalar(2) * scalar(7) + scalar(5) * scalar(4);
        assert_eq!(builder.sumcheck_evaluation(), expected);
    }

    #[test]
    fn we_get_errors_for_missing_evaluation_lengths_and_subpolynomial_capacity() {
        let first_round_evals = [scalar(11)];
        let final_round_evals = [scalar(21)];
        let mle_evaluations = SumcheckMleEvaluations {
            first_round_pcs_proof_evaluations: &first_round_evals,
            final_round_pcs_proof_evaluations: &final_round_evals,
            ..Default::default()
        };
        let mut builder = VerificationBuilderImpl::new(
            mle_evaluations,
            &[],
            &[],
            VecDeque::new(),
            Vec::new(),
            Vec::new(),
            0,
        );
        assert_eq!(
            builder.try_consume_first_round_mle_evaluation().unwrap(),
            scalar(11)
        );
        assert!(matches!(
            builder.try_consume_first_round_mle_evaluation(),
            Err(ProofSizeMismatch::TooFewMLEEvaluations)
        ));
        assert_eq!(
            builder.try_consume_final_round_mle_evaluation().unwrap(),
            scalar(21)
        );
        assert!(matches!(
            builder.try_consume_final_round_mle_evaluation(),
            Err(ProofSizeMismatch::TooFewMLEEvaluations)
        ));
        assert!(matches!(
            builder.try_consume_bit_distribution(),
            Err(ProofSizeMismatch::TooFewBitDistributions)
        ));

        let mle_evaluations = SumcheckMleEvaluations {
            chi_evaluations: indexmap! {
                4 => scalar(44),
            },
            rho_evaluations: indexmap! {
                8 => scalar(88),
            },
            random_evaluation: scalar(9),
            ..Default::default()
        };
        let missing_length_multipliers = [scalar(2)];
        let mut builder = VerificationBuilderImpl::new(
            mle_evaluations,
            &[],
            &missing_length_multipliers,
            VecDeque::new(),
            vec![2],
            vec![3],
            1,
        );

        assert!(matches!(
            builder.try_consume_chi_evaluation(),
            Err(ProofSizeMismatch::ChiLengthNotFound)
        ));
        assert!(matches!(
            builder.try_consume_rho_evaluation(),
            Err(ProofSizeMismatch::RhoLengthNotFound)
        ));
        assert!(matches!(
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::Identity,
                scalar(1),
                1,
            ),
            Err(ProofSizeMismatch::SumcheckProofTooSmall)
        ));

        let constraint_multipliers = [scalar(2)];
        let mut builder = VerificationBuilderImpl::new(
            SumcheckMleEvaluations::default(),
            &[],
            &constraint_multipliers,
            VecDeque::new(),
            Vec::new(),
            Vec::new(),
            1,
        );
        builder
            .try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::ZeroSum,
                scalar(3),
                1,
            )
            .unwrap();
        assert!(matches!(
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::ZeroSum,
                scalar(5),
                1,
            ),
            Err(ProofSizeMismatch::ConstraintCountMismatch)
        ));

        let zero_sum_degree_multipliers = [scalar(2)];
        let mut builder = VerificationBuilderImpl::new(
            SumcheckMleEvaluations::default(),
            &[],
            &zero_sum_degree_multipliers,
            VecDeque::new(),
            Vec::new(),
            Vec::new(),
            0,
        );
        assert!(matches!(
            builder.try_produce_sumcheck_subpolynomial_evaluation(
                SumcheckSubpolynomialType::ZeroSum,
                scalar(1),
                1,
            ),
            Err(ProofSizeMismatch::SumcheckProofTooSmall)
        ));
    }
}
