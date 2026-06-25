use super::{
    extended_dory_reduce_helper::extended_dory_reduce_verify_fold_s_vecs, fold_scalars_0_prove,
    fold_scalars_0_verify, DoryMessages, ExtendedProverState, G1Affine, G2Affine, PublicParameters,
};
use ark_ec::AffineRepr;
use merlin::Transcript;

#[test]
fn we_can_fold_scalars() {
    let nu = 0;
    let pp = PublicParameters {
        Gamma_1: vec![G1Affine::generator()],
        Gamma_2: vec![G2Affine::generator()],
        H_1: G1Affine::generator(),
        H_2: G2Affine::generator(),
        Gamma_2_fin: G2Affine::generator(),
        max_nu: nu,
    };
    let prover_setup = (&pp).into();
    let verifier_setup = (&pp).into();
    let (s1_tensor, s2_tensor) = (vec![], vec![]);
    let (v1, v2) = (vec![G1Affine::generator()], vec![G2Affine::generator()]);
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
