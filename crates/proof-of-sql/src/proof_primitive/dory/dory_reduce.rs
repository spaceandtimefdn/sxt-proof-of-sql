use super::{
    dory_reduce_helper::*, DoryMessages, ProverSetup, ProverState, VerifierSetup, VerifierState,
};
use crate::base::proof::Transcript;

/// This is the prover side of the Dory-Reduce algorithm in section 3.2 of <https://eprint.iacr.org/2020/1274.pdf>.
#[cfg(test)]
pub fn dory_reduce_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: &mut ProverState,
    setup: &ProverSetup,
) {
    assert!(state.nu > 0);
    let half_n = 1usize << (state.nu - 1);
    let (D_1L, D_1R, D_2L, D_2R) = dory_reduce_prove_compute_Ds(state, setup, half_n);
    messages.prover_send_GT_message(transcript, D_1L);
    messages.prover_send_GT_message(transcript, D_1R);
    messages.prover_send_GT_message(transcript, D_2L);
    messages.prover_send_GT_message(transcript, D_2R);
    let betas = messages.verifier_F_message(transcript);
    dory_reduce_prove_mutate_v_vecs(state, setup, betas);
    let (C_plus, C_minus) = dory_reduce_prove_compute_Cs(state, half_n);
    messages.prover_send_GT_message(transcript, C_plus);
    messages.prover_send_GT_message(transcript, C_minus);
    let alphas = messages.verifier_F_message(transcript);
    dory_reduce_prove_fold_v_vecs(state, alphas, half_n);
    state.nu -= 1;
}

/// This is the verifier side of the Dory-Reduce algorithm in section 3.2 of <https://eprint.iacr.org/2020/1274.pdf>.
#[cfg(test)]
pub fn dory_reduce_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: &mut VerifierState,
    setup: &VerifierSetup,
) -> bool {
    assert!(state.nu > 0);
    if messages.GT_messages.len() < 6 {
        return false;
    }
    let D_1L = messages.prover_receive_GT_message(transcript);
    let D_1R = messages.prover_receive_GT_message(transcript);
    let D_2L = messages.prover_receive_GT_message(transcript);
    let D_2R = messages.prover_receive_GT_message(transcript);
    let betas = messages.verifier_F_message(transcript);
    let C_plus = messages.prover_receive_GT_message(transcript);
    let C_minus = messages.prover_receive_GT_message(transcript);
    let alphas = messages.verifier_F_message(transcript);
    dory_reduce_verify_update_C(state, setup, (C_plus, C_minus), alphas, betas);
    dory_reduce_verify_update_Ds(state, setup, (D_1L, D_1R, D_2L, D_2R), alphas, betas);
    state.nu -= 1;
    true
}

#[cfg(test)]
mod tests {
    use super::super::{rand_G_vecs, test_rng, PublicParameters};
    use super::*;
    use merlin::Transcript as MerlinTranscript;

    #[test]
    fn we_can_prove_and_verify_one_dory_reduce_round_directly() {
        let mut rng = test_rng();
        let nu = 3;
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let prover_setup = (&pp).into();
        let verifier_setup = (&pp).into();
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let mut prover_state = ProverState::new(v1, v2, nu);
        let mut verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let mut transcript = MerlinTranscript::new(b"dory_reduce_test");
        let mut messages = DoryMessages::default();
        dory_reduce_prove(&mut messages, &mut transcript, &mut prover_state, &prover_setup);

        assert_eq!(prover_state.nu, nu - 1);

        let mut transcript = MerlinTranscript::new(b"dory_reduce_test");
        assert!(dory_reduce_verify(
            &mut messages,
            &mut transcript,
            &mut verifier_state,
            &verifier_setup
        ));
        assert_eq!(verifier_state.nu, nu - 1);
    }

    #[test]
    fn we_fail_to_verify_one_dory_reduce_round_when_gt_messages_are_missing() {
        let mut rng = test_rng();
        let nu = 3;
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let prover_setup = (&pp).into();
        let verifier_setup = (&pp).into();
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let mut prover_state = ProverState::new(v1, v2, nu);
        let mut verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let mut transcript = MerlinTranscript::new(b"dory_reduce_test");
        let mut messages = DoryMessages::default();
        dory_reduce_prove(&mut messages, &mut transcript, &mut prover_state, &prover_setup);
        messages.GT_messages.pop();

        let mut transcript = MerlinTranscript::new(b"dory_reduce_test");
        assert!(!dory_reduce_verify(
            &mut messages,
            &mut transcript,
            &mut verifier_state,
            &verifier_setup
        ));
        assert_eq!(verifier_state.nu, nu);
    }

    #[test]
    #[should_panic]
    fn dory_reduce_prove_requires_positive_nu() {
        let mut rng = test_rng();
        let nu = 0;
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let prover_setup = (&pp).into();
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let mut prover_state = ProverState::new(v1, v2, nu);

        let mut transcript = MerlinTranscript::new(b"dory_reduce_test");
        let mut messages = DoryMessages::default();
        dory_reduce_prove(&mut messages, &mut transcript, &mut prover_state, &prover_setup);
    }

    #[test]
    #[should_panic]
    fn dory_reduce_verify_requires_positive_nu() {
        let mut rng = test_rng();
        let nu = 0;
        let pp = PublicParameters::test_rand(nu, &mut rng);
        let prover_setup = (&pp).into();
        let verifier_setup = (&pp).into();
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let prover_state = ProverState::new(v1, v2, nu);
        let mut verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let mut transcript = MerlinTranscript::new(b"dory_reduce_test");
        let mut messages = DoryMessages::default();
        dory_reduce_verify(
            &mut messages,
            &mut transcript,
            &mut verifier_state,
            &verifier_setup,
        );
    }
}
