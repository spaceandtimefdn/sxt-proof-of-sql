use super::{
    rand_G_vecs, scalar_product_prove, scalar_product_verify, test_rng, DoryMessages,
    ProverState, PublicParameters, VerifierSetup, VerifierState,
};
use merlin::Transcript;

fn scalar_product_fixture() -> (DoryMessages, VerifierState, VerifierSetup) {
    let mut rng = test_rng();
    let nu = 0;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut messages = DoryMessages::default();
    let mut transcript = Transcript::new(b"scalar_product_test");
    scalar_product_prove(&mut messages, &mut transcript, &prover_state);

    (messages, verifier_state, verifier_setup)
}

#[test]
fn scalar_product_prove_sends_one_group_message_each() {
    let (messages, _, _) = scalar_product_fixture();

    assert_eq!(messages.G1_messages.len(), 1);
    assert_eq!(messages.G2_messages.len(), 1);
    assert!(messages.F_messages.is_empty());
    assert!(messages.GT_messages.is_empty());
}

#[test]
fn scalar_product_verify_accepts_valid_messages() {
    let (mut messages, verifier_state, verifier_setup) = scalar_product_fixture();
    let mut transcript = Transcript::new(b"scalar_product_test");

    assert!(scalar_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn scalar_product_verify_rejects_unexpected_message_counts() {
    fn assert_rejected_by(mutate: impl FnOnce(&mut DoryMessages, &VerifierSetup)) {
        let (mut messages, verifier_state, verifier_setup) = scalar_product_fixture();
        mutate(&mut messages, &verifier_setup);
        let mut transcript = Transcript::new(b"scalar_product_test");

        assert!(!scalar_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ));
    }

    assert_rejected_by(|messages, _| {
        messages.G1_messages.clear();
    });
    assert_rejected_by(|messages, _| {
        let (extra_messages, _, _) = scalar_product_fixture();
        messages.G1_messages.extend(extra_messages.G1_messages);
    });
    assert_rejected_by(|messages, _| {
        messages.G2_messages.clear();
    });
    assert_rejected_by(|messages, _| {
        let (extra_messages, _, _) = scalar_product_fixture();
        messages.G2_messages.extend(extra_messages.G2_messages);
    });
    assert_rejected_by(|messages, _| {
        let (_, _, extra_setup) = scalar_product_fixture();
        messages.GT_messages.push(extra_setup.H_T);
    });
}
