use super::{
    DeferredG1, DeferredG2, DeferredGT, G2Affine, ProverState, VMVProverState, VerifierState, F,
};
#[cfg(test)]
use super::{G1Affine, G1Projective, G2Projective, ProverSetup};
use alloc::{vec, vec::Vec};
#[cfg(test)]
use ark_ec::VariableBaseMSM;
use ark_ff::Fp;

/// The state of the prover during the Dory proof generation with the extended algorithm.
/// `base_state` is the state of the prover during the Dory proof generation with the original algorithm.
/// See the beginning of section 4 of <https://eprint.iacr.org/2020/1274.pdf> for details.
pub struct ExtendedProverState {
    /// The state of the prover during the Dory proof generation with the original algorithm.
    pub(super) base_state: ProverState,
    /// The first tensor of F elements in the witness. This will be mutated during the proof generation.
    #[cfg(test)]
    pub(super) s1_tensor: Vec<F>,
    /// The second tensor of F elements in the witness. This will be mutated during the proof generation.
    #[cfg(test)]
    pub(super) s2_tensor: Vec<F>,
    /// The first vector of F elements in the witness. This will be mutated during the proof generation.
    pub(super) s1: Vec<F>,
    /// The second vector of F elements in the witness. This will be mutated during the proof generation.
    pub(super) s2: Vec<F>,
}

impl ExtendedProverState {
    pub fn from_vmv_prover_state(state: VMVProverState, v2: Vec<G2Affine>) -> Self {
        let s1 = state.R_vec;
        let s2 = state.L_vec;
        let v1 = state.T_vec_prime;
        let nu = state.nu;
        ExtendedProverState {
            base_state: ProverState::new(v1, v2, nu),
            s1,
            s2,
            #[cfg(test)]
            s1_tensor: state.r_tensor,
            #[cfg(test)]
            s2_tensor: state.l_tensor,
        }
    }
    /// Create a new `ExtendedProverState` from the witness using the tensor representation.
    #[cfg(test)]
    pub fn new_from_tensors(
        s1_tensor: Vec<F>,
        s2_tensor: Vec<F>,
        v1: Vec<G1Affine>,
        v2: Vec<G2Affine>,
        nu: usize,
    ) -> Self {
        use crate::base::polynomial::compute_evaluation_vector;
        assert_eq!(s1_tensor.len(), nu);
        assert_eq!(s2_tensor.len(), nu);
        let mut s1 = vec![Fp::default(); 1 << nu];
        let mut s2 = vec![Fp::default(); 1 << nu];
        compute_evaluation_vector(&mut s1, &s1_tensor);
        compute_evaluation_vector(&mut s2, &s2_tensor);
        ExtendedProverState {
            base_state: ProverState::new(v1, v2, nu),
            s1,
            s2,
            #[cfg(test)]
            s1_tensor,
            #[cfg(test)]
            s2_tensor,
        }
    }
    /// Calculate the verifier state from the prover state and setup information.
    /// This is basically the commitment computation of the witness.
    /// See the beginning of section 4 of <https://eprint.iacr.org/2020/1274.pdf> for details.
    #[cfg(test)]
    pub fn calculate_verifier_state(&self, setup: &ProverSetup) -> ExtendedVerifierState {
        let E_1: G1Affine = G1Projective::msm_unchecked(&self.base_state.v1, &self.s2).into();
        let E_2: G2Affine = G2Projective::msm_unchecked(&self.base_state.v2, &self.s1).into();
        ExtendedVerifierState {
            base_state: self.base_state.calculate_verifier_state(setup),
            E_1: E_1.into(),
            E_2: E_2.into(),
            s1_tensor: self.s1_tensor.clone(),
            s2_tensor: self.s2_tensor.clone(),
            alphas: vec![Fp::default(); self.base_state.nu],
            alpha_invs: vec![Fp::default(); self.base_state.nu],
        }
    }
}

/// The state of the verifier during the Dory proof verification with the extended algorithm.
/// `base_state` is the state of the verifier during the Dory proof verification with the original algorithm.
/// See the beginning of section 4 of <https://eprint.iacr.org/2020/1274.pdf> for details.
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct ExtendedVerifierState {
    /// The state of the verifier during the Dory proof verification with the original algorithm.
    pub(super) base_state: super::VerifierState,
    /// The "commitment" to s1. This should be <v1,s2>. This will be mutated during the proof verification.
    pub(super) E_1: DeferredG1,
    /// The "commitment" to s2. This should be <s1,v2>. This will be mutated during the proof verification.
    pub(super) E_2: DeferredG2,
    /// The first tensor of F elements in the witness. This will NOT be mutated during the proof verification.
    pub(super) s1_tensor: Vec<F>,
    /// The second tensor of F elements in the witness. This will NOT be mutated during the proof verification.
    pub(super) s2_tensor: Vec<F>,
    /// The folding factors for the `s1_tensors`. This will be populated during the proof verification.
    pub(super) alphas: Vec<F>,
    /// The folding factors for the `s2_tensors`. This will be populated during the proof verification.
    pub(super) alpha_invs: Vec<F>,
}

impl ExtendedVerifierState {
    /// Create a new `ExtendedVerifierState` from the commitment to the witness.
    #[expect(clippy::too_many_arguments)]
    pub fn new_tensor(
        E_1: DeferredG1,
        E_2: DeferredG2,
        s1_tensor: Vec<F>,
        s2_tensor: Vec<F>,
        C: DeferredGT,
        D_1: DeferredGT,
        D_2: DeferredGT,
        nu: usize,
    ) -> Self {
        ExtendedVerifierState {
            base_state: VerifierState::new(C, D_1, D_2, nu),
            E_1,
            E_2,
            s1_tensor,
            s2_tensor,
            alphas: vec![Fp::default(); nu],
            alpha_invs: vec![Fp::default(); nu],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{rand_F_tensors, rand_G_vecs, test_rng, ProverSetup, PublicParameters};
    use super::*;
    use ark_ec::pairing::Pairing;

    #[test]
    fn we_can_build_extended_verifier_state_from_tensor_commitments() {
        let mut rng = test_rng();
        let max_nu = 4;
        let nu = 3;
        let pp = PublicParameters::test_rand(max_nu, &mut rng);
        let prover_setup: ProverSetup = (&pp).into();
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let (s1_tensor, s2_tensor) = rand_F_tensors(nu, &mut rng);

        let C: DeferredGT = Pairing::multi_pairing(&v1, &v2).into();
        let D_1: DeferredGT = Pairing::multi_pairing(&v1, prover_setup.Gamma_2[nu]).into();
        let D_2: DeferredGT = Pairing::multi_pairing(prover_setup.Gamma_1[nu], &v2).into();
        let E_1 = DeferredG1::from(v1[0]);
        let E_2 = DeferredG2::from(v2[0]);

        let verifier_state = ExtendedVerifierState::new_tensor(
            E_1.clone(),
            E_2.clone(),
            s1_tensor.clone(),
            s2_tensor.clone(),
            C.clone(),
            D_1.clone(),
            D_2.clone(),
            nu,
        );

        assert_eq!(verifier_state.E_1, E_1);
        assert_eq!(verifier_state.E_2, E_2);
        assert_eq!(verifier_state.s1_tensor, s1_tensor);
        assert_eq!(verifier_state.s2_tensor, s2_tensor);
        assert_eq!(verifier_state.base_state.C, C);
        assert_eq!(verifier_state.base_state.D_1, D_1);
        assert_eq!(verifier_state.base_state.D_2, D_2);
        assert_eq!(verifier_state.base_state.nu, nu);
        assert_eq!(verifier_state.alphas, vec![F::default(); nu]);
        assert_eq!(verifier_state.alpha_invs, vec![F::default(); nu]);
    }
}
