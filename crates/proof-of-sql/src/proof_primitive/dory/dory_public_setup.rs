use super::{ProverSetup, VerifierSetup};

/// The public setup required for the Dory PCS by the prover and the commitment computation.
#[derive(Clone, Copy)]
pub struct DoryProverPublicSetup<'a> {
    prover_setup: &'a ProverSetup<'a>,
    sigma: usize,
}
impl<'a> DoryProverPublicSetup<'a> {
    /// Create a new public setup for the Dory PCS.
    /// `public_parameters`: The public parameters for the Dory protocol.
    /// `sigma`: A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    #[must_use]
    pub fn new(prover_setup: &'a ProverSetup<'a>, sigma: usize) -> Self {
        Self {
            prover_setup,
            sigma,
        }
    }
    /// Returns sigma. A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    #[must_use]
    pub fn sigma(&self) -> usize {
        self.sigma
    }
    /// The public setup for the Dory protocol.
    #[must_use]
    pub fn prover_setup(&self) -> &ProverSetup<'_> {
        self.prover_setup
    }
}

/// The verifier's public setup for the Dory PCS.
#[derive(Clone, Copy)]
pub struct DoryVerifierPublicSetup<'a> {
    verifier_setup: &'a VerifierSetup,
    sigma: usize,
}
impl<'a> DoryVerifierPublicSetup<'a> {
    /// Create a new public setup for the Dory PCS.
    /// `verifier_setup`: The verifier's setup parameters for the Dory protocol.
    /// `sigma`: A commitment with this setup is a matrix commitment with `1 << sigma` columns.
    #[must_use]
    pub fn new(verifier_setup: &'a VerifierSetup, sigma: usize) -> Self {
        Self {
            verifier_setup,
            sigma,
        }
    }
    /// Returns sigma. A commitment with this setup is a matrix commitment with `1<<sigma` columns.
    #[must_use]
    pub fn sigma(&self) -> usize {
        self.sigma
    }
    /// The verifier's setup parameters for the Dory protocol.
    #[must_use]
    pub fn verifier_setup(&self) -> &VerifierSetup {
        self.verifier_setup
    }
}

#[cfg(test)]
mod tests {
    use super::{DoryProverPublicSetup, DoryVerifierPublicSetup};
    use crate::proof_primitive::dory::{test_rng, ProverSetup, PublicParameters, VerifierSetup};

    #[test]
    fn prover_public_setup_exposes_sigma_and_underlying_setup() {
        let mut rng = test_rng();
        let public_parameters = PublicParameters::test_rand(2, &mut rng);
        let prover_setup = ProverSetup::from(&public_parameters);
        let setup = DoryProverPublicSetup::new(&prover_setup, 3);

        assert_eq!(setup.sigma(), 3);
        assert!(core::ptr::eq(setup.prover_setup(), &prover_setup));

        let copied_setup = setup;
        assert_eq!(copied_setup.sigma(), 3);
        assert!(core::ptr::eq(copied_setup.prover_setup(), &prover_setup));

        let cloned_setup = setup.clone();
        assert_eq!(cloned_setup.sigma(), 3);
        assert!(core::ptr::eq(cloned_setup.prover_setup(), &prover_setup));
    }

    #[test]
    fn verifier_public_setup_exposes_sigma_and_underlying_setup() {
        let mut rng = test_rng();
        let public_parameters = PublicParameters::test_rand(2, &mut rng);
        let verifier_setup = VerifierSetup::from(&public_parameters);
        let setup = DoryVerifierPublicSetup::new(&verifier_setup, 4);

        assert_eq!(setup.sigma(), 4);
        assert!(core::ptr::eq(setup.verifier_setup(), &verifier_setup));

        let copied_setup = setup;
        assert_eq!(copied_setup.sigma(), 4);
        assert!(core::ptr::eq(
            copied_setup.verifier_setup(),
            &verifier_setup
        ));

        let cloned_setup = setup.clone();
        assert_eq!(cloned_setup.sigma(), 4);
        assert!(core::ptr::eq(
            cloned_setup.verifier_setup(),
            &verifier_setup
        ));
    }
}
