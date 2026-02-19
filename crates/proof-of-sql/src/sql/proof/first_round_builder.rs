use crate::{
    base::{
        commitment::{Commitment, CommittableColumn, VecCommitmentExt},
        polynomial::MultilinearExtension,
        proof::{Keccak256Transcript, Transcript},
        scalar::Scalar,
    },
    utils::log,
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct FirstRoundMessage<C> {
    /// Length of the range of generators we use
    pub range_length: usize,
    pub post_result_challenge_count: usize,
    /// Chi evaluation lengths
    pub chi_evaluation_lengths: Vec<usize>,
    /// Rho evaluation lengths
    pub rho_evaluation_lengths: Vec<usize>,
    /// First Round Commitments
    pub round_commitments: Vec<C>,
}

pub struct PreliminaryRound<'a, S> {
    commitment_descriptor: Vec<CommittableColumn<'a>>,
    pcs_proof_mles: Vec<Box<dyn MultilinearExtension<S> + 'a>>,
    /// The number of challenges used in the proof.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    num_post_result_challenges: usize,
    /// The extra chi evaluation lengths used in the proof.
    chi_evaluation_lengths: Vec<usize>,
    /// The rho evaluation lengths used in the proof.
    rho_evaluation_lengths: Vec<usize>,
    // The range_length used in sumcheck which is max of all possible ones.
    range_length: usize,
}

/// Track the result created by a query
pub struct FirstRoundBuilder<'a, S> {
    rounds: Vec<PreliminaryRound<'a, S>>,
}

impl<'a, S: Scalar> FirstRoundBuilder<'a, S> {
    pub fn new(initial_range_length: usize) -> Self {
        Self {
            rounds: vec![PreliminaryRound {
                commitment_descriptor: Vec::new(),
                pcs_proof_mles: Vec::new(),
                num_post_result_challenges: 0,
                chi_evaluation_lengths: Vec::new(),
                rho_evaluation_lengths: Vec::new(),
                range_length: initial_range_length,
            }],
        }
    }

    fn current_round_mut(&mut self) -> &mut PreliminaryRound<'a, S> {
        self.rounds.last_mut().expect("at least one round exists")
    }

    fn current_round(&self) -> &PreliminaryRound<'a, S> {
        self.rounds.last().expect("at least one round exists")
    }

    /// Get the range length used in the proof.
    pub(crate) fn range_length(&self) -> usize {
        self.current_round().range_length
    }

    /// Update the range length used in the proof only if the new range is larger than the existing range.
    pub(crate) fn update_range_length(&mut self, new_range_length: usize) {
        let current_round = self.current_round_mut();
        if new_range_length > current_round.range_length {
            current_round.range_length = new_range_length;
        }
    }

    pub fn pcs_proof_mles(&self) -> Vec<&Vec<Box<dyn MultilinearExtension<S> + 'a>>> {
        self.rounds
            .iter()
            .map(|round| &round.pcs_proof_mles)
            .collect::<Vec<_>>()
    }

    /// Get the chi evaluation lengths used in the proof.
    pub(crate) fn chi_evaluation_lengths(&self) -> &[usize] {
        &self.current_round().chi_evaluation_lengths
    }

    /// Append the length to the list of chi evaluation lengths.
    pub(crate) fn produce_chi_evaluation_length(&mut self, length: usize) {
        self.update_range_length(length);
        self.current_round_mut().chi_evaluation_lengths.push(length);
    }

    /// Get the rho evaluation lengths used in the proof.
    pub(crate) fn rho_evaluation_lengths(&self) -> Vec<usize> {
        self.rounds
            .iter()
            .flat_map(|round| round.rho_evaluation_lengths.clone())
            .collect::<Vec<_>>()
    }

    /// Append the length to the list of rho evaluation lengths.
    pub(crate) fn produce_rho_evaluation_length(&mut self, length: usize) {
        self.current_round_mut().rho_evaluation_lengths.push(length);
    }

    /// Produce an MLE for a intermediate computed column that we can reference in sumcheck.
    ///
    /// Because the verifier doesn't have access to the MLE's commitment, we will need to
    /// commit to the MLE before we form the sumcheck polynomial.
    pub fn produce_intermediate_mle(
        &mut self,
        data: impl MultilinearExtension<S> + Into<CommittableColumn<'a>> + Copy + 'a,
    ) {
        let current_round = self.current_round_mut();
        current_round.commitment_descriptor.push(data.into());
        current_round.pcs_proof_mles.push(Box::new(data));
    }

    /// Given the evaluation vector, compute evaluations of all the MLEs used in sumcheck except
    /// for those that correspond to result columns sent to the verifier.
    #[tracing::instrument(
        name = "FirstRoundBuilder::evaluate_pcs_proof_mles",
        level = "debug",
        skip_all
    )]
    pub fn evaluate_pcs_proof_mles(&self, evaluation_vec: &[S]) -> Vec<Vec<S>> {
        log::log_memory_usage("Start");

        let res = self
            .pcs_proof_mles()
            .iter()
            .map(|pcs_proof_mles| {
                let mut res = Vec::with_capacity(pcs_proof_mles.len());
                for evaluator in pcs_proof_mles.iter() {
                    res.push(evaluator.inner_product(evaluation_vec));
                }
                res
            })
            .collect::<Vec<_>>();

        log::log_memory_usage("End");

        res
    }

    /// Request `cnt` more post result challenges.
    /// Specifically, these are the challenges that the verifier sends to
    /// the prover after the prover sends the result, but before the prover
    /// send commitments to the intermediate witness columns.
    pub fn request_post_result_challenges(&mut self, cnt: usize) {
        self.current_round_mut().num_post_result_challenges += cnt;
    }

    pub fn get_first_round_messages<C: Commitment + Serialize>(
        &self,
        offset_generators: usize,
        setup: &C::PublicSetup<'_>,
    ) -> (
        Vec<FirstRoundMessage<C>>,
        VecDeque<C::Scalar>,
        Keccak256Transcript,
    ) {
        let mut transcript: Keccak256Transcript = Transcript::new();
        let messages: Vec<FirstRoundMessage<C>> = self
            .rounds
            .iter()
            .map(|round| FirstRoundMessage {
                range_length: round.range_length,
                post_result_challenge_count: round.num_post_result_challenges,
                chi_evaluation_lengths: round.chi_evaluation_lengths.clone(),
                rho_evaluation_lengths: round.rho_evaluation_lengths.clone(),
                round_commitments: Vec::from_committable_columns_with_offset(
                    &round.commitment_descriptor,
                    offset_generators,
                    setup,
                ),
            })
            .collect();
        let challenges: Vec<_> = messages
            .iter()
            .map(|message| {
                transcript.extend_serialize_as_le(message);
                core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                    .take(message.post_result_challenge_count)
                    .collect::<Vec<C::Scalar>>()
            })
            .collect();
        let challenges = challenges.into_iter().flatten().collect::<VecDeque<_>>();
        (messages, challenges, transcript)
    }
}
