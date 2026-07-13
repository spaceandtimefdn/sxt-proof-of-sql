use super::{SumcheckSubpolynomial, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommittableColumn, VecCommitmentExt},
        if_rayon,
        polynomial::MultilinearExtension,
        scalar::Scalar,
    },
    utils::log,
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Track components used to form a query's proof
pub struct FinalRoundBuilder<'a, S: Scalar> {
    num_sumcheck_variables: usize,
    bit_distributions: Vec<BitDistribution>,
    commitment_descriptor: Vec<CommittableColumn<'a>>,
    pcs_proof_mles: Vec<Box<dyn MultilinearExtension<S> + 'a>>,
    sumcheck_subpolynomials: Vec<SumcheckSubpolynomial<'a, S>>,
    /// The challenges used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    ///
    /// Note: this vector is treated as a stack and the first
    /// challenge is the last entry in the vector.
    post_result_challenges: VecDeque<S>,
}

impl<'a, S: Scalar> FinalRoundBuilder<'a, S> {
    pub fn new(num_sumcheck_variables: usize, post_result_challenges: VecDeque<S>) -> Self {
        Self {
            num_sumcheck_variables,
            bit_distributions: Vec::new(),
            commitment_descriptor: Vec::new(),
            pcs_proof_mles: Vec::new(),
            sumcheck_subpolynomials: Vec::new(),
            post_result_challenges,
        }
    }

    pub fn num_sumcheck_variables(&self) -> usize {
        self.num_sumcheck_variables
    }

    pub fn num_sumcheck_subpolynomials(&self) -> usize {
        self.sumcheck_subpolynomials.len()
    }

    pub fn pcs_proof_mles(&self) -> &[Box<dyn MultilinearExtension<S> + 'a>] {
        &self.pcs_proof_mles
    }

    /// Produce a bit distribution that describes which bits are constant
    /// and which bits varying in a column of data
    pub fn produce_bit_distribution(&mut self, dist: BitDistribution) {
        self.bit_distributions.push(dist);
    }

    /// Produce an anchored MLE that we can reference in sumcheck.
    ///
    /// An anchored MLE is an MLE where the verifier has access to the commitment.
    pub fn produce_anchored_mle(&mut self, data: impl MultilinearExtension<S> + 'a) {
        self.pcs_proof_mles.push(Box::new(data));
    }

    /// Produce an MLE for a intermediate computed column that we can reference in sumcheck.
    ///
    /// Because the verifier doesn't have access to the MLE's commitment, we will need to
    /// commit to the MLE before we form the sumcheck polynomial.
    pub fn produce_intermediate_mle(
        &mut self,
        data: impl MultilinearExtension<S> + Into<CommittableColumn<'a>> + Copy + 'a,
    ) {
        self.commitment_descriptor.push(data.into());
        self.produce_anchored_mle(data);
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    pub fn produce_sumcheck_subpolynomial(
        &mut self,
        subpolynomial_type: SumcheckSubpolynomialType,
        terms: Vec<SumcheckSubpolynomialTerm<'a, S>>,
    ) {
        self.sumcheck_subpolynomials
            .push(SumcheckSubpolynomial::new(subpolynomial_type, terms));
    }

    /// Compute commitments of all the interemdiate MLEs used in sumcheck
    #[tracing::instrument(
        name = "FinalRoundBuilder::commit_intermediate_mles",
        level = "debug",
        skip_all
    )]
    pub fn commit_intermediate_mles<C: Commitment>(
        &self,
        offset_generators: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Vec<C> {
        log::log_memory_usage("Start");

        let res = Vec::from_committable_columns_with_offset(
            &self.commitment_descriptor,
            offset_generators,
            setup,
        );

        log::log_memory_usage("End");

        res
    }

    /// Produce a subpolynomial to be aggegated into sumcheck where the sum across binary
    /// values of the variables is zero.
    pub fn sumcheck_subpolynomials(&self) -> &[SumcheckSubpolynomial<'a, S>] {
        &self.sumcheck_subpolynomials
    }

    /// Given the evaluation vector, compute evaluations of all the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    #[tracing::instrument(
        name = "FinalRoundBuilder::evaluate_pcs_proof_mles",
        level = "debug",
        skip_all
    )]
    pub fn evaluate_pcs_proof_mles(&self, evaluation_vec: &[S]) -> Vec<S> {
        log::log_memory_usage("Start");

        let res: Vec<S> = if_rayon!(self.pcs_proof_mles.par_iter(), self.pcs_proof_mles.iter())
            .map(|evaluator| evaluator.inner_product(evaluation_vec))
            .collect();

        log::log_memory_usage("End");

        res
    }

    pub fn bit_distributions(&self) -> &[BitDistribution] {
        &self.bit_distributions
    }

    /// Pops a challenge off the stack of post-result challenges.
    ///
    /// These challenges are used in creation of the constraints in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    /// # Panics
    ///
    /// Will panic if there are no post-result challenges available to pop from the stack.
    pub fn consume_post_result_challenge(&mut self) -> S {
        self.post_result_challenges.pop_front().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::{boxed::Box, collections::VecDeque, vec};

    #[test]
    fn we_can_track_final_round_builder_state() {
        let mle = [TestScalar::from(1), TestScalar::from(2)];
        let mut builder = FinalRoundBuilder::<TestScalar>::new(
            3,
            VecDeque::from([TestScalar::from(7), TestScalar::from(11)]),
        );

        assert_eq!(builder.num_sumcheck_variables(), 3);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 0);
        assert!(builder.bit_distributions().is_empty());
        assert!(builder.sumcheck_subpolynomials().is_empty());
        assert!(builder.pcs_proof_mles().is_empty());

        let bit_distribution = BitDistribution::new::<TestScalar, _>(&[0_i64, 1, 2]);
        builder.produce_bit_distribution(bit_distribution.clone());
        assert_eq!(builder.bit_distributions(), &[bit_distribution]);

        let terms: Vec<SumcheckSubpolynomialTerm<'_, TestScalar>> =
            vec![(TestScalar::from(5), vec![Box::new(&mle)])];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomialType::ZeroSum, terms);

        assert_eq!(builder.num_sumcheck_subpolynomials(), 1);
        {
            let mut subpolynomial_terms =
                builder.sumcheck_subpolynomials()[0].iter_mul_by(TestScalar::from(3));
            let (subpolynomial_type, coefficient, multiplicands) =
                subpolynomial_terms.next().unwrap();
            assert_eq!(subpolynomial_type, SumcheckSubpolynomialType::ZeroSum);
            assert_eq!(coefficient, TestScalar::from(15));
            assert_eq!(multiplicands.len(), 1);
            assert!(subpolynomial_terms.next().is_none());
        }

        assert_eq!(builder.consume_post_result_challenge(), TestScalar::from(7));
        assert_eq!(
            builder.consume_post_result_challenge(),
            TestScalar::from(11)
        );
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn we_panic_when_consuming_missing_post_result_challenge() {
        let mut builder = FinalRoundBuilder::<TestScalar>::new(0, VecDeque::new());

        builder.consume_post_result_challenge();
    }
}
