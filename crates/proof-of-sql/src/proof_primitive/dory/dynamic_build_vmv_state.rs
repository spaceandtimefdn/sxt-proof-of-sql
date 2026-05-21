use super::{
    dynamic_dory_helper::compute_dynamic_v_vec, DeferredGT, DoryScalar, G1Affine, VMVProverState,
    VMVVerifierState, F,
};
use crate::proof_primitive::dynamic_matrix_utils::standard_basis_helper::compute_dynamic_vecs;
use alloc::vec::Vec;

/// Builds a [`VMVProverState`] from the given parameters.
pub(super) fn build_dynamic_vmv_prover_state(
    a: &[F],
    b_point: &[F],
    T_vec_prime: Vec<G1Affine>,
    nu: usize,
) -> VMVProverState {
    let (lo_vec, hi_vec) =
        compute_dynamic_vecs(bytemuck::TransparentWrapper::wrap_slice(b_point) as &[DoryScalar]);
    let (lo_vec, hi_vec) = (
        bytemuck::TransparentWrapper::peel_slice(&lo_vec) as &[F],
        bytemuck::TransparentWrapper::peel_slice(&hi_vec) as &[F],
    );
    let v_vec = compute_dynamic_v_vec(a, hi_vec, nu);
    VMVProverState {
        v_vec,
        T_vec_prime,
        L_vec: hi_vec.to_vec(),
        R_vec: lo_vec.to_vec(),
        #[cfg(test)]
        l_tensor: Vec::with_capacity(0),
        #[cfg(test)]
        r_tensor: b_point.to_vec(),
        nu,
    }
}

/// Builds a [`VMVVerifierState`] from the given parameters.
pub(super) fn build_dynamic_vmv_verifier_state(
    y: F,
    b_point: &[F],
    T: DeferredGT,
    nu: usize,
) -> VMVVerifierState {
    VMVVerifierState {
        y,
        T,
        l_tensor: Vec::with_capacity(0),
        r_tensor: b_point.to_vec(),
        nu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute_dynamic_vecs_as_fields(b_point: &[F]) -> (Vec<F>, Vec<F>) {
        let (lo_vec, hi_vec) = compute_dynamic_vecs(bytemuck::TransparentWrapper::wrap_slice(
            b_point,
        ) as &[DoryScalar]);
        (
            bytemuck::TransparentWrapper::peel_slice(&lo_vec).to_vec(),
            bytemuck::TransparentWrapper::peel_slice(&hi_vec).to_vec(),
        )
    }

    #[test]
    fn we_can_build_dynamic_vmv_prover_state() {
        let a: Vec<F> = (100..109).map(Into::into).collect();
        let b_point: Vec<F> = [2, 3, 5, 7, 11].into_iter().map(Into::into).collect();
        let nu = 3;
        let t_vec_prime = vec![G1Affine::identity(); 1 << nu];

        let state = build_dynamic_vmv_prover_state(&a, &b_point, t_vec_prime.clone(), nu);

        let (expected_r_vec, expected_l_vec) = compute_dynamic_vecs_as_fields(&b_point);
        let expected_v_vec = compute_dynamic_v_vec(&a, &expected_l_vec, nu);

        assert_eq!(state.v_vec, expected_v_vec);
        assert_eq!(state.T_vec_prime, t_vec_prime);
        assert_eq!(state.L_vec, expected_l_vec);
        assert_eq!(state.R_vec, expected_r_vec);
        assert!(state.l_tensor.is_empty());
        assert_eq!(state.r_tensor, b_point);
        assert_eq!(state.nu, nu);
    }

    #[test]
    fn we_can_build_dynamic_vmv_verifier_state() {
        let y = F::from(42);
        let b_point: Vec<F> = [2, 3, 5, 7, 11].into_iter().map(Into::into).collect();
        let t = DeferredGT::new([], []);
        let nu = 3;

        let state = build_dynamic_vmv_verifier_state(y, &b_point, t.clone(), nu);

        assert_eq!(state.y, y);
        assert_eq!(state.T, t);
        assert!(state.l_tensor.is_empty());
        assert_eq!(state.r_tensor, b_point);
        assert_eq!(state.nu, nu);
    }
}
