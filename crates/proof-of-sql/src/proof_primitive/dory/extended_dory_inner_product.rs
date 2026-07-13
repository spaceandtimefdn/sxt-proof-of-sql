use super::{
    scalar_product_prove, scalar_product_verify, DoryMessages, ExtendedProverState,
    ExtendedVerifierState, ProverSetup, VerifierSetup, F,
};
use crate::{
    base::proof::Transcript,
    proof_primitive::dory::{
        extended_dory_reduce_prove, extended_dory_reduce_verify, fold_scalars_0_prove,
        fold_scalars_0_verify,
    },
    utils::log,
};

/// This is the prover side of the extended Dory-Innerproduct algorithm in section 4.3 of <https://eprint.iacr.org/2020/1274.pdf>.
/// This function builds/enqueues `messages`, appends to `transcript`, and consumes `state`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_inner_product_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedProverState,
    setup: &ProverSetup,
) {
    log::log_memory_usage("Start");

    let nu = state.base_state.nu;
    assert!(setup.max_nu >= nu);
    for _ in 0..nu {
        extended_dory_reduce_prove(messages, transcript, &mut state, setup);
    }
    let base_state = fold_scalars_0_prove(messages, transcript, state, setup);
    scalar_product_prove(messages, transcript, &base_state);

    log::log_memory_usage("End");
}

/// This is the verifier side of the extended Dory-Innerproduct algorithm in section 4.3 of <https://eprint.iacr.org/2020/1274.pdf>.
/// This function consumes/dequeues from `messages`, appends to `transcript`, and consumes `state`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_inner_product_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    mut state: ExtendedVerifierState,
    setup: &VerifierSetup,
    fold_s_tensors_verify: impl Fn(&ExtendedVerifierState) -> (F, F),
) -> bool {
    log::log_memory_usage("Start");

    let nu = state.base_state.nu;
    assert!(setup.max_nu >= nu);
    for _ in 0..nu {
        if !extended_dory_reduce_verify(messages, transcript, &mut state, setup) {
            return false;
        }
    }
    let base_state =
        fold_scalars_0_verify(messages, transcript, state, setup, fold_s_tensors_verify);
    let res = scalar_product_verify(messages, transcript, base_state, setup);

    log::log_memory_usage("End");

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitive::dory::{
        deferred_msm::DeferredMSM, test_rng, PublicParameters, VerifierState,
    };
    use merlin::Transcript;

    #[test]
    fn we_reject_extended_dory_inner_product_when_messages_are_empty() {
        let public_parameters = PublicParameters::test_rand(1, &mut test_rng());
        let verifier_setup = VerifierSetup::from(&public_parameters);
        let verifier_state = ExtendedVerifierState {
            base_state: VerifierState::new(
                DeferredMSM::new([], []),
                DeferredMSM::new([], []),
                DeferredMSM::new([], []),
                1,
            ),
            E_1: DeferredMSM::new([], []),
            E_2: DeferredMSM::new([], []),
            s1_tensor: vec![F::from(2)],
            s2_tensor: vec![F::from(3)],
            alphas: vec![F::from(0)],
            alpha_invs: vec![F::from(0)],
        };
        let mut messages = DoryMessages::default();
        let mut transcript = Transcript::new(b"extended_dory_inner_product_test");

        assert!(!extended_dory_inner_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup,
            |_| panic!("folding should not run when reduction messages are empty")
        ));
    }
}
