use super::{
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    pairings, DeferredGT, DoryMessages, G1Projective, G2Projective, ProverSetup, ProverState,
    VerifierSetup, VerifierState, F,
};
use crate::{base::proof::Transcript, utils::log};

/// This is the prover side of the Fold-Scalars algorithm in section 4.1 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Note: this only works for nu = 0.
#[expect(clippy::missing_panics_doc)]
pub fn fold_scalars_0_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedProverState,
    setup: &ProverSetup,
) -> ProverState {
    assert_eq!(state.base_state.nu, 0);
    let (gamma, gamma_inv) = messages.verifier_F_message(transcript);
    state.base_state.v1[0] = (state.base_state.v1[0] + setup.H_1 * state.s1[0] * gamma).into();
    state.base_state.v2[0] = (state.base_state.v2[0] + setup.H_2 * state.s2[0] * gamma_inv).into();
    state.base_state
}

/// This is the verifier side of the Fold-Scalars algorithm in section 4.1 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Note: this only works for nu = 0.
///
/// See [extended_dory_reduce_verify_fold_s_vecs](super::extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs)
/// for an explaination of the `s_folded` values
#[tracing::instrument(level = "debug", skip_all)]
pub fn fold_scalars_0_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedVerifierState,
    setup: &VerifierSetup,
    fold_s_tensors_verify: impl Fn(&ExtendedVerifierState) -> (F, F),
) -> VerifierState {
    log::log_memory_usage("Start");

    assert_eq!(state.base_state.nu, 0);
    let (gamma, gamma_inv) = messages.verifier_F_message(transcript);
    let (s1_folded, s2_folded) = fold_s_tensors_verify(&state);
    state.base_state.C += DeferredGT::from(setup.H_T) * s1_folded * s2_folded
        + DeferredGT::from(pairings::pairing(
            setup.H_1,
            state.E_2.compute::<G2Projective>(),
        )) * gamma
        + DeferredGT::from(pairings::pairing(
            state.E_1.compute::<G1Projective>(),
            setup.H_2,
        )) * gamma_inv;
    state.base_state.D_1 += pairings::pairing(setup.H_1, setup.Gamma_2_0 * s1_folded * gamma);
    state.base_state.D_2 += pairings::pairing(setup.Gamma_1_0 * s2_folded * gamma_inv, setup.H_2);

    log::log_memory_usage("End");

    state.base_state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitive::dory::{deferred_msm::DeferredMSM, test_rng, PublicParameters};
    use merlin::Transcript;

    #[test]
    fn we_keep_verifier_state_unchanged_for_zero_folded_scalars() {
        let public_parameters = PublicParameters::test_rand(0, &mut test_rng());
        let verifier_setup = VerifierSetup::from(&public_parameters);
        let expected_state = VerifierState::new(
            DeferredMSM::new([], []),
            DeferredMSM::new([], []),
            DeferredMSM::new([], []),
            0,
        );
        let state = ExtendedVerifierState {
            base_state: VerifierState::new(
                DeferredMSM::new([], []),
                DeferredMSM::new([], []),
                DeferredMSM::new([], []),
                0,
            ),
            E_1: DeferredMSM::new([], []),
            E_2: DeferredMSM::new([], []),
            s1_tensor: vec![F::from(5)],
            s2_tensor: vec![F::from(7)],
            alphas: vec![F::from(0)],
            alpha_invs: vec![F::from(0)],
        };
        let mut messages = DoryMessages::default();
        let mut transcript = Transcript::new(b"fold_scalars_test");

        let folded_state = fold_scalars_0_verify(
            &mut messages,
            &mut transcript,
            state,
            &verifier_setup,
            |_| (F::from(0), F::from(0)),
        );

        assert_eq!(folded_state, expected_state);
    }
}
