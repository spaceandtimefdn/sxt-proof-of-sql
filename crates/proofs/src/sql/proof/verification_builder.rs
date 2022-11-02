use super::SumcheckMleEvaluations;

use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar, traits::Identity};

/// Track components used to verify a query's proof
pub struct VerificationBuilder<'a> {
    pub mle_evaluations: SumcheckMleEvaluations<'a>,
    intermediate_commitments: &'a [RistrettoPoint],
    subpolynomial_multipliers: &'a [Scalar],
    inner_product_multipliers: &'a [Scalar],
    sumcheck_evaluation: Scalar,
    folded_pre_result_commitment: RistrettoPoint,
    consumed_result_mles: usize,
    consumed_pre_result_mles: usize,
    consumed_intermediate_mles: usize,
    produced_subpolynomials: usize,
}

impl<'a> VerificationBuilder<'a> {
    pub fn new(
        mle_evaluations: SumcheckMleEvaluations<'a>,
        intermediate_commitments: &'a [RistrettoPoint],
        subpolynomial_multipliers: &'a [Scalar],
        inner_product_multipliers: &'a [Scalar],
    ) -> Self {
        assert_eq!(
            inner_product_multipliers.len(),
            mle_evaluations.pre_result_evaluations.len()
        );
        Self {
            mle_evaluations,
            intermediate_commitments,
            subpolynomial_multipliers,
            inner_product_multipliers,
            sumcheck_evaluation: Scalar::zero(),
            folded_pre_result_commitment: RistrettoPoint::identity(),
            consumed_result_mles: 0,
            consumed_pre_result_mles: 0,
            consumed_intermediate_mles: 0,
            produced_subpolynomials: 0,
        }
    }

    /// Consume the evaluation of an anchored MLE used in sumcheck and provide the commitment of the MLE
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment
    pub fn consume_anchored_mle(&mut self, commitment: &RistrettoPoint) -> Scalar {
        let index = self.consumed_pre_result_mles;
        self.folded_pre_result_commitment += self.inner_product_multipliers[index] * commitment;
        self.consumed_pre_result_mles += 1;
        self.mle_evaluations.pre_result_evaluations[index]
    }

    /// Consume the evaluation of an intermediate MLE used in sumcheck
    ///
    /// An interemdiate MLE is one where the verifier doesn't have access to its commitment
    pub fn consume_intermediate_mle(&mut self) -> Scalar {
        let commitment = &self.intermediate_commitments[self.consumed_intermediate_mles];
        self.consumed_intermediate_mles += 1;
        self.consume_anchored_mle(commitment)
    }

    /// Consume the evaluation of the MLE for a result column used in sumcheck
    pub fn consume_result_mle(&mut self) -> Scalar {
        let index = self.consumed_result_mles;
        self.consumed_result_mles += 1;
        self.mle_evaluations.result_evaluations[index]
    }

    /// Produce the evaluation of a subpolynomial used in sumcheck
    pub fn produce_sumcheck_subpolynomial_evaluation(&mut self, eval: &Scalar) {
        self.sumcheck_evaluation +=
            self.subpolynomial_multipliers[self.produced_subpolynomials] * eval;
        self.produced_subpolynomials += 1;
    }

    /// Get the evaluation of the sumcheck polynomial at its randomly selected point
    pub fn sumcheck_evaluation(&self) -> Scalar {
        assert!(self.completed());
        self.sumcheck_evaluation
    }

    /// Get the commitment of the folded pre-result MLE vectors used in a verifiable query's
    /// bulletproof
    pub fn folded_pre_result_commitment(&self) -> RistrettoPoint {
        assert!(self.completed());
        self.folded_pre_result_commitment
    }

    /// Check that the verification builder is completely built up
    fn completed(&self) -> bool {
        self.produced_subpolynomials == self.subpolynomial_multipliers.len()
            && self.consumed_intermediate_mles == self.intermediate_commitments.len()
            && self.consumed_pre_result_mles == self.mle_evaluations.pre_result_evaluations.len()
            && self.consumed_result_mles == self.mle_evaluations.result_evaluations.len()
    }
}
