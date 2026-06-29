#[cfg(test)]
mod dory_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::proof_primitive::dory::{
            DoryCommitment, DoryEvaluationProof, DoryMessages, DoryProverPublicSetup,
            DoryScalar, DoryVerifierPublicSetup, DynamicDoryCommitment, DynamicDoryEvaluationProof,
            ProverSetup, PublicParameters, VerifierSetup,
        };
        // Just verify types exist
        assert!(true);
    }
}
