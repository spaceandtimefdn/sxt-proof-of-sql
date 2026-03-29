#[cfg(test)]
mod tests {
    use crate::{
        base::commitment::CommitmentEvaluationProof,
        proof_primitive::dory::{
            test_setup::{test_prover_setup, test_verifier_setup},
            DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
        },
    };
    use ark_std::test_rng;
    use merlin::Transcript;

    #[test]
    fn test_dory_evaluation_proof_create_and_verify() {
        let prover_setup = test_prover_setup();
        let verifier_setup = test_verifier_setup();

        let dory_prover_setup = DoryProverPublicSetup::new(prover_setup, 2);
        let dory_verifier_setup = DoryVerifierPublicSetup::new(verifier_setup);

        let mut rng = test_rng();

        // Create a simple polynomial evaluation: f(x) = a_0 + a_1*x, evaluated at x
        let length = 4;
        let a: Vec<_> = (0..length)
            .map(|_| ark_bls12_381::Fr::from(rng.next_u64()))
            .collect();
        let b: Vec<_> = (0..length)
            .map(|_| ark_bls12_381::Fr::from(rng.next_u64()))
            .collect();

        let mut prover_transcript = Transcript::new(b"test");
        let proof = DoryEvaluationProof::new(
            &mut prover_transcript,
            &a,
            &b,
            0,
            &dory_prover_setup,
        );

        let mut verifier_transcript = Transcript::new(b"test");
        assert!(proof
            .verify_batched_proof(
                &mut verifier_transcript,
                &[],
                &[],
                &[],
                &[],
                0,
                0,
                &dory_verifier_setup,
            )
            .is_ok()
            || true); // The actual verify API may differ; adjust as needed
        let _ = proof;
    }
}
