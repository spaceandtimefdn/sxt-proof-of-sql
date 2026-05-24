//! Tests for Dory prover and verifier public setup.

#[cfg(test)]
mod dory_public_setup_test {
    use crate::proof_primitive::dory::dory_public_setup::{
        DoryProverPublicSetup, DoryVerifierPublicSetup,
    };
    use crate::proof_primitive::dory::setup::{ProverSetup, VerifierSetup};
    use crate::proof_primitive::dory::PublicParameters;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_dory_prover_public_setup_new() {
        // Create minimal public parameters for testing
        let pp = PublicParameters::test_instance(1);
        let prover_setup = ProverSetup::from(&pp);

        let setup = DoryProverPublicSetup::new(&prover_setup, 2);
        assert_eq!(setup.sigma(), 2);
        let _ = setup.prover_setup();
    }

    #[test]
    fn test_dory_verifier_public_setup_new() {
        let pp = PublicParameters::test_instance(1);
        let verifier_setup = VerifierSetup::from(&pp);

        let setup = DoryVerifierPublicSetup::new(&verifier_setup, 3);
        assert_eq!(setup.sigma(), 3);
        let _ = setup.verifier_setup();
    }

    #[test]
    fn test_dory_prover_public_setup_copy() {
        let pp = PublicParameters::test_instance(1);
        let prover_setup = ProverSetup::from(&pp);

        let setup1 = DoryProverPublicSetup::new(&prover_setup, 2);
        let setup2 = setup1; // Copy (references)
        assert_eq!(setup2.sigma(), setup1.sigma());
    }

    #[test]
    fn test_dory_verifier_public_setup_copy() {
        let pp = PublicParameters::test_instance(1);
        let verifier_setup = VerifierSetup::from(&pp);

        let setup1 = DoryVerifierPublicSetup::new(&verifier_setup, 3);
        let setup2 = setup1; // Copy (references)
        assert_eq!(setup2.sigma(), setup1.sigma());
    }
}