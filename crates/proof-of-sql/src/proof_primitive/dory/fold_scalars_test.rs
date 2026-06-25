use super::{
    extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs, fold_scalars_0_prove,
    fold_scalars_0_verify, rand_F_tensors, rand_G_vecs, test_rng, DoryMessages,
    ExtendedProverState, G1Affine, G2Affine, PublicParameters,
};
use merlin::Transcript;

#[test]
fn we_can_fold_scalars() {
    let mut rng = test_rng();
    let nu = 0;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

    let mut transcript = Transcript::new(b"fold_scalars_test");
    let mut messages = DoryMessages::default();
    let prover_folded_state =
        fold_scalars_0_prove(&mut messages, &mut transcript, prover_state, &prover_setup);

    let mut transcript = Transcript::new(b"fold_scalars_test");
    let verifier_folded_state = fold_scalars_0_verify(
        &mut messages,
        &mut transcript,
        verifier_state,
        &verifier_setup,
        extended_dory_reduce_verify_fold_s_vecs,
    );
    assert_eq!(
        prover_folded_state.calculate_verifier_state(&prover_setup),
        verifier_folded_state
    );
}

#[test]
fn fold_scalars_prover_adds_challenge_scaled_hiding_terms() {
    let mut rng = test_rng();
    let nu = 0;
    let pp = PublicParameters::test_rand(nu, &mut rng);
    let prover_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);
    let (v1, v2) = rand_G_vecs(nu, &mut rng);
    let prover_state = ExtendedProverState::new_from_tensors(s1_tensor, s2_tensor, v1, v2, nu);
    let initial_v1 = prover_state.base_state.v1[0];
    let initial_v2 = prover_state.base_state.v2[0];
    let s1 = prover_state.s1[0];
    let s2 = prover_state.s2[0];

    let mut challenge_transcript = Transcript::new(b"fold_scalars_prover_update_test");
    let mut challenge_messages = DoryMessages::default();
    let (gamma, gamma_inv) = challenge_messages.verifier_F_message(&mut challenge_transcript);

    let mut transcript = Transcript::new(b"fold_scalars_prover_update_test");
    let mut messages = DoryMessages::default();
    let prover_folded_state =
        fold_scalars_0_prove(&mut messages, &mut transcript, prover_state, &prover_setup);
    let expected_v1: G1Affine = (initial_v1 + prover_setup.H_1 * s1 * gamma).into();
    let expected_v2: G2Affine = (initial_v2 + prover_setup.H_2 * s2 * gamma_inv).into();

    assert_eq!(prover_folded_state.nu, 0);
    assert_eq!(prover_folded_state.v1[0], expected_v1);
    assert_eq!(prover_folded_state.v2[0], expected_v2);
}
