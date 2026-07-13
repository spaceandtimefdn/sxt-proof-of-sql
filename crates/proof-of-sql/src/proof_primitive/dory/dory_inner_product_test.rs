use super::{
    dory_inner_product_prove, dory_inner_product_verify, rand_G_vecs, test_rng, DoryMessages,
    G1Affine, ProverState, PublicParameters, VerifierSetup, VerifierState, GT,
};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn we_can_prove_and_verify_a_dory_inner_product() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}

#[test]
fn we_can_prove_and_verify_a_dory_inner_product_for_multiple_nu_values() {
    let mut rng = test_rng();
    let max_nu = 5;
    let pp = PublicParameters::test_rand(max_nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();

    for nu in 0..max_nu {
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let prover_state = ProverState::new(v1, v2, nu);
        let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let mut transcript = Transcript::new(b"dory_inner_product_test");
        let mut messages = DoryMessages::default();
        dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

        let mut transcript = Transcript::new(b"dory_inner_product_test");
        assert!(dory_inner_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ));
    }
}

fn assert_dory_inner_product_rejects(
    mut messages: DoryMessages,
    verifier_state: VerifierState,
    verifier_setup: &VerifierSetup,
) {
    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        verifier_setup
    ));
}
#[test]
fn we_reject_malformed_dory_inner_product_proofs_with_one_setup() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut base_messages = DoryMessages::default();
    dory_inner_product_prove(
        &mut base_messages,
        &mut transcript,
        prover_state,
        &prover_setup,
    );

    let mut messages = base_messages.clone();
    messages.GT_messages[0] = GT::rand(&mut rng);
    assert_dory_inner_product_rejects(messages, verifier_state.clone(), &verifier_setup);

    let mut messages = base_messages.clone();
    messages.GT_messages.pop();
    assert_dory_inner_product_rejects(messages, verifier_state.clone(), &verifier_setup);

    let mut messages = base_messages.clone();
    messages.GT_messages.push(GT::rand(&mut rng));
    assert_dory_inner_product_rejects(messages, verifier_state.clone(), &verifier_setup);

    let mut messages = base_messages.clone();
    messages.G1_messages.pop();
    assert_dory_inner_product_rejects(messages, verifier_state.clone(), &verifier_setup);

    let mut messages = base_messages.clone();
    messages.G1_messages.push(G1Affine::rand(&mut rng));
    assert_dory_inner_product_rejects(messages, verifier_state.clone(), &verifier_setup);

    let mut verifier_state = verifier_state;
    verifier_state.C = GT::rand(&mut rng).into();
    assert_dory_inner_product_rejects(base_messages, verifier_state, &verifier_setup);
}

#[test]
fn we_fail_to_verify_a_dory_inner_product_when_the_transcripts_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test_wrong");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}
#[test]
fn we_fail_to_verify_a_dory_inner_product_when_the_setups_differ() {
    let mut rng = test_rng();
    let nu = 3;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let pp_wrong = PublicParameters::test_rand(nu, &mut rng);
    let verifier_setup = (&pp_wrong).into();
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ProverState::new(v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    let mut messages = DoryMessages::default();
    dory_inner_product_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    messages.GT_messages[0] = GT::rand(&mut rng);

    let mut transcript = Transcript::new(b"dory_inner_product_test");
    assert!(!dory_inner_product_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup
    ));
}
