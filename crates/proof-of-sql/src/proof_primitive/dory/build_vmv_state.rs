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
    use ark_ec::{pairing::PairingOutput, AffineRepr};
    use num_traits::One;

    fn g1(scale: u64) -> G1Affine {
        (G1Affine::generator() * F::from(scale)).into()
    }

    #[test]
    fn build_vmv_prover_state_populates_vectors_and_tensors() {
        let a = [F::from(5), F::from(7), F::from(11), F::from(13)];
        let b_point = [F::from(2), F::from(3)];
        let T_vec_prime = vec![g1(17), g1(19)];

        let state = build_vmv_prover_state(&a, &b_point, T_vec_prime.clone(), 1, 1);

        assert_eq!(state.v_vec, vec![F::from(23), F::from(25)]);
        assert_eq!(state.T_vec_prime, T_vec_prime);
        assert_eq!(state.l_tensor, vec![F::from(3)]);
        assert_eq!(state.r_tensor, vec![F::from(2)]);
        assert_eq!(state.L_vec, vec![F::from(-2), F::from(3)]);
        assert_eq!(state.R_vec, vec![F::from(-1), F::from(2)]);
        assert_eq!(state.nu, 1);
    }

    #[test]
    fn build_vmv_verifier_state_populates_tensors() {
        let b_point = [F::from(2), F::from(3)];
        let T = DeferredGT::from(PairingOutput::<ark_bls12_381::Bls12_381>(One::one()));

        let state = build_vmv_verifier_state(F::from(29), &b_point, T, 1, 1);

        assert_eq!(state.y, F::from(29));
        assert_eq!(state.l_tensor, vec![F::from(3)]);
        assert_eq!(state.r_tensor, vec![F::from(2)]);
        assert_eq!(state.nu, 1);
    }
}
