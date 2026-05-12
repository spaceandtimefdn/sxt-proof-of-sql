use super::{
    extended_state::{ExtendedProverState, ExtendedVerifierState},
    DeferredG1, DeferredG2, G1Affine, G1Projective, G2Affine, G2Projective, ProverSetup, F,
};
use crate::utils::log;
use ark_ec::VariableBaseMSM;
use ark_ff::Field;

/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Computes
/// * `E_1beta` = <`Gamma_1`, `s_2`>
/// * `E_2beta` = <`s_1`, `Gamma_2`>
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_reduce_prove_compute_E_betas(
    state: &ExtendedProverState,
    setup: &ProverSetup,
) -> (G1Affine, G2Affine) {
    log::log_memory_usage("Start");

    let E_1beta: G1Affine =
        G1Projective::msm_unchecked(setup.Gamma_1[state.base_state.nu], &state.s2).into();
    let E_2beta: G2Affine =
        G2Projective::msm_unchecked(setup.Gamma_2[state.base_state.nu], &state.s1).into();

    log::log_memory_usage("End");

    (E_1beta, E_2beta)
}
/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Computes
/// * `E_1plus` = <`v_1L`, `s_2R`>
/// * `E_1minus` = <`v_1R`, `s_2L`>
/// * `E_2plus` = <`s_1L`, `v_2R`>
/// * `E_2minus` = <`s_1R`, `v_2L`>
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_reduce_prove_compute_signed_Es(
    state: &ExtendedProverState,
    half_n: usize,
) -> (G1Affine, G1Affine, G2Affine, G2Affine) {
    log::log_memory_usage("Start");

    let (v_1L, v_1R) = state.base_state.v1.split_at(half_n);
    let (v_2L, v_2R) = state.base_state.v2.split_at(half_n);
    let (s_1L, s_1R) = state.s1.split_at(half_n);
    let (s_2L, s_2R) = state.s2.split_at(half_n);
    let E_1plus = G1Projective::msm_unchecked(v_1L, s_2R).into();
    let E_1minus = G1Projective::msm_unchecked(v_1R, s_2L).into();
    let E_2plus = G2Projective::msm_unchecked(v_2R, s_1L).into();
    let E_2minus = G2Projective::msm_unchecked(v_2L, s_1R).into();

    log::log_memory_usage("End");

    (E_1plus, E_1minus, E_2plus, E_2minus)
}
/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Folds `s1` and `s2`.
/// * `s_1`' <- alpha * `s_1L` + `s_1R`
/// * `s_2`' <- `alpha_inv` * `s_2L` + `s_2R`
#[tracing::instrument(level = "debug", skip_all)]
pub fn extended_dory_reduce_prove_fold_s_vecs(
    state: &mut ExtendedProverState,
    (alpha, alpha_inv): (F, F),
    half_n: usize,
) {
    log::log_memory_usage("Start");

    let (s_1L, s_1R) = state.s1.split_at_mut(half_n);
    let (s_2L, s_2R) = state.s2.split_at_mut(half_n);
    s_1L.iter_mut()
        .zip(s_1R)
        .for_each(|(s_L, s_R)| *s_L = *s_L * alpha + s_R);
    s_2L.iter_mut()
        .zip(s_2R)
        .for_each(|(s_L, s_R)| *s_L = *s_L * alpha_inv + s_R);
    state.s1.truncate(half_n);
    state.s2.truncate(half_n);

    log::log_memory_usage("End");
}
/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Updates `E_1` and `E_2`
/// * `E_1' <- E_1 + beta * E_1beta + alpha * E_1plus + alpha_inv * E_1minus`
/// * `E_2' <- E_2 + beta_inv * E_2beta + alpha * E_2plus + alpha_inv * E_2minus`
pub fn extended_dory_reduce_verify_update_Es(
    state: &mut ExtendedVerifierState,
    (E_1beta, E_2beta): (G1Affine, G2Affine),
    (E_1plus, E_1minus, E_2plus, E_2minus): (G1Affine, G1Affine, G2Affine, G2Affine),
    (alpha, alpha_inv): (F, F),
    (beta, beta_inv): (F, F),
) {
    state.E_1 += DeferredG1::from(E_1beta) * beta
        + DeferredG1::from(E_1plus) * alpha
        + DeferredG1::from(E_1minus) * alpha_inv;
    state.E_2 += DeferredG2::from(E_2beta) * beta_inv
        + DeferredG2::from(E_2plus) * alpha
        + DeferredG2::from(E_2minus) * alpha_inv;
}

/// From the extended Dory-Reduce algorithm in section 4.2 of <https://eprint.iacr.org/2020/1274.pdf>.
///
/// Folds s1 and s2.
/// * `s_1' <- alpha * s_1L + s_1R`
/// * `s_2' <- alpha_inv * s_2L + s_2R`
///
/// NOTE: this logically is identical to `extended_dory_reduce_prove_fold_s_vecs`. However, the actual values
/// of the s vectors not needed.
///
/// Instead, only the final, completely folded value is used, in [`fold_scalars_0_verify`](super::fold_scalars_0_verify).
/// This implementation works because the final value of the s vectors is:
///
/// `product (1-s1_tensor[i]) * alpha[i] + s1_tensor[i] over all i`
///
/// So, instead of folding the s vectors, we can directly compute the final value by mutating
///
/// `s1_tensor[nu-1] <- s1_tensor[nu-1] * (1- alpha) + alpha`
///
/// and taking the product in [`fold_scalars_0_verify`](super::fold_scalars_0_verify).
pub fn extended_dory_reduce_verify_fold_s_vecs(state: &ExtendedVerifierState) -> (F, F) {
    (
        state
            .s1_tensor
            .iter()
            .zip(state.alphas.iter())
            .map(|(s, a)| (F::ONE - s) * a + s)
            .product(),
        state
            .s2_tensor
            .iter()
            .zip(state.alpha_invs.iter())
            .map(|(s, a)| (F::ONE - s) * a + s)
            .product(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitive::dory::{
        deferred_msm::DeferredMSM, G1Affine, G1Projective, G2Affine, G2Projective, ProverSetup,
        PublicParameters, GT,
    };
    use ark_ec::{CurveGroup, PrimeGroup, VariableBaseMSM};

    fn test_extended_prover_state(nu: usize, setup: &ProverSetup<'_>) -> ExtendedProverState {
        ExtendedProverState::new_from_tensors(
            vec![F::from(2), F::from(3), F::from(5)][..nu].to_vec(),
            vec![F::from(7), F::from(11), F::from(13)][..nu].to_vec(),
            setup.Gamma_1[nu][..1 << nu].to_vec(),
            setup.Gamma_2[nu][..1 << nu].to_vec(),
            nu,
        )
    }

    #[test]
    fn we_can_compute_extended_dory_e_beta_messages() {
        let nu = 2;
        let public_parameters = PublicParameters::test_rand(nu, &mut ark_std::test_rng());
        let setup = ProverSetup::from(&public_parameters);
        let state = test_extended_prover_state(nu, &setup);

        let (E_1beta, E_2beta) = extended_dory_reduce_prove_compute_E_betas(&state, &setup);
        let expected_E_1beta: G1Affine =
            G1Projective::msm_unchecked(setup.Gamma_1[nu], &state.s2).into();
        let expected_E_2beta: G2Affine =
            G2Projective::msm_unchecked(setup.Gamma_2[nu], &state.s1).into();

        assert_eq!(E_1beta, expected_E_1beta);
        assert_eq!(E_2beta, expected_E_2beta);
    }

    #[test]
    fn we_can_compute_extended_dory_signed_e_messages() {
        let public_parameters = PublicParameters::test_rand(2, &mut ark_std::test_rng());
        let setup = ProverSetup::from(&public_parameters);
        let state = test_extended_prover_state(2, &setup);

        let (E_1plus, E_1minus, E_2plus, E_2minus) =
            extended_dory_reduce_prove_compute_signed_Es(&state, 2);
        let expected_E_1plus: G1Affine =
            G1Projective::msm_unchecked(&state.base_state.v1[..2], &state.s2[2..]).into();
        let expected_E_1minus: G1Affine =
            G1Projective::msm_unchecked(&state.base_state.v1[2..], &state.s2[..2]).into();
        let expected_E_2plus: G2Affine =
            G2Projective::msm_unchecked(&state.base_state.v2[2..], &state.s1[..2]).into();
        let expected_E_2minus: G2Affine =
            G2Projective::msm_unchecked(&state.base_state.v2[..2], &state.s1[2..]).into();

        assert_eq!(E_1plus, expected_E_1plus);
        assert_eq!(E_1minus, expected_E_1minus);
        assert_eq!(E_2plus, expected_E_2plus);
        assert_eq!(E_2minus, expected_E_2minus);
    }

    #[test]
    fn we_can_fold_extended_prover_s_vectors() {
        let nu = 2;
        let mut state = ExtendedProverState::new_from_tensors(
            vec![F::from(2), F::from(3)],
            vec![F::from(5), F::from(7)],
            vec![G1Affine::default(); 1 << nu],
            vec![G2Affine::default(); 1 << nu],
            nu,
        );
        let original_s1 = state.s1.clone();
        let original_s2 = state.s2.clone();
        let alpha = F::from(11);
        let alpha_inv = F::from(13);

        extended_dory_reduce_prove_fold_s_vecs(&mut state, (alpha, alpha_inv), 2);

        assert_eq!(
            state.s1,
            vec![
                original_s1[0] * alpha + original_s1[2],
                original_s1[1] * alpha + original_s1[3]
            ]
        );
        assert_eq!(
            state.s2,
            vec![
                original_s2[0] * alpha_inv + original_s2[2],
                original_s2[1] * alpha_inv + original_s2[3],
            ]
        );
    }

    #[test]
    fn we_can_fold_extended_verifier_s_tensors() {
        let s1_tensor = vec![F::from(2), F::from(3), F::from(5)];
        let s2_tensor = vec![F::from(7), F::from(11), F::from(13)];
        let alphas = vec![F::from(17), F::from(19), F::from(23)];
        let alpha_invs = vec![F::from(29), F::from(31), F::from(37)];
        let state = ExtendedVerifierState {
            base_state: super::super::VerifierState::new(
                DeferredMSM::<GT, F>::new([], []),
                DeferredMSM::<GT, F>::new([], []),
                DeferredMSM::<GT, F>::new([], []),
                3,
            ),
            E_1: DeferredMSM::<G1Affine, F>::new([], []),
            E_2: DeferredMSM::<G2Affine, F>::new([], []),
            s1_tensor: s1_tensor.clone(),
            s2_tensor,
            alphas: alphas.clone(),
            alpha_invs: alpha_invs.clone(),
        };

        let (s1_fold, s2_fold) = extended_dory_reduce_verify_fold_s_vecs(&state);

        let expected_s1 = s1_tensor
            .iter()
            .zip(alphas.iter())
            .map(|(s, a)| (F::ONE - s) * a + s)
            .product();
        let expected_s2 = state
            .s2_tensor
            .iter()
            .zip(alpha_invs.iter())
            .map(|(s, a)| (F::ONE - s) * a + s)
            .product();
        assert_eq!(s1_fold, expected_s1);
        assert_eq!(s2_fold, expected_s2);
    }

    #[test]
    fn we_can_update_extended_verifier_e_commitments() {
        let E_1beta = G1Projective::generator().into_affine();
        let E_1plus = (E_1beta * F::from(2)).into();
        let E_1minus = (E_1beta * F::from(3)).into();
        let E_2beta = G2Projective::generator().into_affine();
        let E_2plus = (E_2beta * F::from(5)).into();
        let E_2minus = (E_2beta * F::from(7)).into();
        let alpha = F::from(11);
        let alpha_inv = F::from(13);
        let beta = F::from(17);
        let beta_inv = F::from(19);
        let mut state = ExtendedVerifierState {
            base_state: super::super::VerifierState::new(
                DeferredMSM::<GT, F>::new([], []),
                DeferredMSM::<GT, F>::new([], []),
                DeferredMSM::<GT, F>::new([], []),
                2,
            ),
            E_1: DeferredMSM::<G1Affine, F>::new([], []),
            E_2: DeferredMSM::<G2Affine, F>::new([], []),
            s1_tensor: Vec::new(),
            s2_tensor: Vec::new(),
            alphas: vec![F::from(0); 2],
            alpha_invs: vec![F::from(0); 2],
        };

        extended_dory_reduce_verify_update_Es(
            &mut state,
            (E_1beta, E_2beta),
            (E_1plus, E_1minus, E_2plus, E_2minus),
            (alpha, alpha_inv),
            (beta, beta_inv),
        );

        assert_eq!(
            state.E_1,
            E_1beta * beta + E_1plus * alpha + E_1minus * alpha_inv
        );
        assert_eq!(
            state.E_2,
            E_2beta * beta_inv + E_2plus * alpha + E_2minus * alpha_inv
        );
    }
}
