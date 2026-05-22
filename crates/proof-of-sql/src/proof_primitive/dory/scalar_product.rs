use super::{pairings, DoryMessages, ProverState, VerifierSetup, VerifierState};
use crate::{base::proof::Transcript, utils::log};

/// This is the prover side of the Scalar-Product algorithm in section 3.1 of <https://eprint.iacr.org/2020/1274.pdf>.
#[expect(clippy::missing_panics_doc)]
pub fn scalar_product_prove(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: &ProverState,
) {
    // See section 3.1 of https://eprint.iacr.org/2020/1274.pdf.
    //
    // Note:
    // We use nu = m and k = m-i or m-j.
    // This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
    //
    // So,
    // * `Gamma_1[k]` = Γ_1,(m-k) in the paper.
    // * `Gamma_2[k]` = Γ_2,(m-k) in the paper.
    // * `Delta_1L[k]` = Δ_1L,(m-k) in the paper, so `Delta_1L[0]` is unused.
    // * `Delta_1R[k]` = Δ_1R,(m-k) in the paper, so `Delta_1R[0]` is unused.
    // * `Delta_2L[k]` = Δ_2L,(m-k) in the paper, so `Delta_2L[0]` is unused.
    // * `Delta_2R[k]` = Δ_2R,(m-k) in the paper, so `Delta_2R[0]` is unused.
    // * `chi[k]` = χ,(m-k) in the paper.
    // * `Gamma_1_0` is the Γ_1 used in Scalar-Product algorithm.
    // * `Gamma_2_0` is the Γ_2 used in Scalar-Product algorithm.

    assert_eq!(state.nu, 0);
    // v1 is a single element.
    let E_1 = state.v1[0];
    // v2 is a single element.
    let E_2 = state.v2[0];
    messages.prover_send_G1_message(transcript, E_1);
    messages.prover_send_G2_message(transcript, E_2);
    let (_d, _d_inv) = messages.verifier_F_message(transcript);
}

/// This is the verifier side of the Scalar-Product algorithm in section 3.1 of <https://eprint.iacr.org/2020/1274.pdf>.
#[tracing::instrument(level = "debug", skip_all)]
pub fn scalar_product_verify(
    messages: &mut DoryMessages,
    transcript: &mut impl Transcript,
    state: VerifierState,
    setup: &VerifierSetup,
) -> bool {
    // See section 3.1 of https://eprint.iacr.org/2020/1274.pdf.
    //
    // Note:
    // We use nu = m and k = m-i or m-j.
    // This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
    //
    // So,
    // * `Gamma_1[k]` = Γ_1,(m-k) in the paper.
    // * `Gamma_2[k]` = Γ_2,(m-k) in the paper.
    // * `Delta_1L[k]` = Δ_1L,(m-k) in the paper, so `Delta_1L[0]` is unused.
    // * `Delta_1R[k]` = Δ_1R,(m-k) in the paper, so `Delta_1R[0]` is unused.
    // * `Delta_2L[k]` = Δ_2L,(m-k) in the paper, so `Delta_2L[0]` is unused.
    // * `Delta_2R[k]` = Δ_2R,(m-k) in the paper, so `Delta_2R[0]` is unused.
    // * `chi[k]` = χ,(m-k) in the paper.
    // * `Gamma_1_0` is the Γ_1 used in Scalar-Product algorithm.
    // * `Gamma_2_0` is the Γ_2 used in Scalar-Product algorithm.

    log::log_memory_usage("Start");

    assert_eq!(state.nu, 0);
    if messages.G1_messages.len() != 1
        || messages.G2_messages.len() != 1
        || !messages.GT_messages.is_empty()
    {
        return false;
    }
    let E_1 = messages.prover_receive_G1_message(transcript);
    let E_2 = messages.prover_receive_G2_message(transcript);
    let (d, d_inv) = messages.verifier_F_message(transcript);
    let res = pairings::pairing(E_1 + setup.Gamma_1_0 * d, E_2 + setup.Gamma_2_0 * d_inv)
        == (state.C + setup.chi[0] + state.D_2 * d + state.D_1 * d_inv).compute();

    log::log_memory_usage("End");

    res
}

#[cfg(test)]
mod tests {
    use super::super::{
        rand_G_vecs, test_rng, DoryMessages, G1Affine, ProverSetup, ProverState, PublicParameters,
        VerifierSetup, VerifierState,
    };
    use super::{scalar_product_prove, scalar_product_verify};
    use ark_std::UniformRand;
    use merlin::Transcript;

    fn prove_scalar_product() -> (DoryMessages, VerifierState, VerifierSetup) {
        let mut rng = test_rng();
        let public_parameters = PublicParameters::test_rand(0, &mut rng);
        let prover_setup = ProverSetup::from(&public_parameters);
        let verifier_setup = VerifierSetup::from(&public_parameters);
        let (v1, v2) = rand_G_vecs(0, &mut rng);
        let prover_state = ProverState::new(v1, v2, 0);
        let verifier_state = prover_state.calculate_verifier_state(&prover_setup);
        let mut messages = DoryMessages::default();
        let mut transcript = Transcript::new(b"scalar_product_test");

        scalar_product_prove(&mut messages, &mut transcript, &prover_state);

        (messages, verifier_state, verifier_setup)
    }

    #[test]
    fn scalar_product_verify_accepts_valid_nu_zero_messages() {
        let (mut messages, verifier_state, verifier_setup) = prove_scalar_product();
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
        let (_, verifier_state, verifier_setup) = prove_scalar_product();
        let mut messages = DoryMessages::default();
        let mut transcript = Transcript::new(b"scalar_product_test");

        assert!(!scalar_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ));

        let (mut messages, verifier_state, verifier_setup) = prove_scalar_product();
        messages.G1_messages.push(messages.G1_messages[0]);
        let mut transcript = Transcript::new(b"scalar_product_test");

        assert!(!scalar_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ));
    }

    #[test]
    fn scalar_product_verify_rejects_tampered_g1_message() {
        let mut rng = test_rng();
        let (mut messages, verifier_state, verifier_setup) = prove_scalar_product();
        let original_message = messages.G1_messages[0];
        let mut tampered_message = G1Affine::rand(&mut rng);
        while tampered_message == original_message {
            tampered_message = G1Affine::rand(&mut rng);
        }
        messages.G1_messages[0] = tampered_message;
        let mut transcript = Transcript::new(b"scalar_product_test");

        assert!(!scalar_product_verify(
            &mut messages,
            &mut transcript,
            verifier_state,
            &verifier_setup
        ));
    }
}
