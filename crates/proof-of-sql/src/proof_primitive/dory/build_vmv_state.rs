use super::{
    compute_L_R_vec, compute_l_r_tensors, compute_v_vec, DeferredGT, G1Affine, VMVProverState,
    VMVVerifierState, F,
};
use alloc::vec::Vec;

/// Builds a [`VMVProverState`] from the given parameters.
pub(super) fn build_vmv_prover_state(
    a: &[F],
    b_point: &[F],
    T_vec_prime: Vec<G1Affine>,
    sigma: usize,
    nu: usize,
) -> VMVProverState {
    let (L_vec, R_vec) = compute_L_R_vec(b_point, sigma, nu);
    #[cfg(test)]
    let (l_tensor, r_tensor) = compute_l_r_tensors(b_point, sigma, nu);
    let v_vec = compute_v_vec(a, &L_vec, sigma, nu);
    VMVProverState {
        v_vec,
        T_vec_prime,
        #[cfg(test)]
        l_tensor,
        #[cfg(test)]
        r_tensor,
        L_vec,
        R_vec,
        nu,
    }
}

/// Builds a [`VMVVerifierState`] from the given parameters.
pub(super) fn build_vmv_verifier_state(
    y: F,
    b_point: &[F],
    T: DeferredGT,
    sigma: usize,
    nu: usize,
) -> VMVVerifierState {
    let (l_tensor, r_tensor) = compute_l_r_tensors(b_point, sigma, nu);
    VMVVerifierState {
        y,
        T,
        l_tensor,
        r_tensor,
        nu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::AffineRepr;

    #[test]
    fn we_can_build_vmv_prover_state_from_matrix_inputs() {
        let a = vec![
            F::from(1),
            F::from(2),
            F::from(3),
            F::from(4),
            F::from(5),
            F::from(6),
        ];
        let b_point = vec![F::from(3), F::from(5), F::from(7)];
        let T_vec_prime = vec![G1Affine::zero(); 4];
        let sigma = 1;
        let nu = 2;

        let state = build_vmv_prover_state(&a, &b_point, T_vec_prime.clone(), sigma, nu);

        let (expected_L_vec, expected_R_vec) = compute_L_R_vec(&b_point, sigma, nu);
        let (expected_l_tensor, expected_r_tensor) = compute_l_r_tensors(&b_point, sigma, nu);
        let expected_v_vec = compute_v_vec(&a, &expected_L_vec, sigma, nu);

        assert_eq!(state.v_vec, expected_v_vec);
        assert_eq!(state.T_vec_prime, T_vec_prime);
        assert_eq!(state.l_tensor, expected_l_tensor);
        assert_eq!(state.r_tensor, expected_r_tensor);
        assert_eq!(state.L_vec, expected_L_vec);
        assert_eq!(state.R_vec, expected_R_vec);
        assert_eq!(state.nu, nu);
    }
}
