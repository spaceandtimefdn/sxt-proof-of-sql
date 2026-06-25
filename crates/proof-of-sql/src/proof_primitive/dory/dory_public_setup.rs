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
    use crate::proof_primitive::dory::{test_params_nu_4, test_verifier_setup_nu_4, ProverSetup};

    #[test]
    fn dory_verifier_setup_sigma_returns_configured_value() {
        let vs = test_verifier_setup_nu_4();
        let setup = DoryVerifierPublicSetup::new(vs, 3);
        assert_eq!(setup.sigma(), 3);
    }

    #[test]
    fn dory_verifier_setup_sigma_zero() {
        let vs = test_verifier_setup_nu_4();
        let setup = DoryVerifierPublicSetup::new(vs, 0);
        assert_eq!(setup.sigma(), 0);
    }

    #[test]
    fn dory_verifier_setup_returns_same_setup_reference() {
        let vs = test_verifier_setup_nu_4();
        let setup = DoryVerifierPublicSetup::new(vs, 2);
        assert!(core::ptr::eq(setup.verifier_setup(), vs));
    }

    #[test]
    fn dory_verifier_setup_is_copy() {
        let vs = test_verifier_setup_nu_4();
        let setup = DoryVerifierPublicSetup::new(vs, 5);
        let copied = setup;
        assert_eq!(copied.sigma(), 5);
    }

    #[test]
    fn dory_prover_setup_sigma_returns_configured_value() {
        let pp = test_params_nu_4();
        let prover_setup = ProverSetup::from(pp);
        let setup = DoryProverPublicSetup::new(&prover_setup, 4);
        assert_eq!(setup.sigma(), 4);
    }

    #[test]
    fn dory_prover_setup_sigma_zero() {
        let pp = test_params_nu_4();
        let prover_setup = ProverSetup::from(pp);
        let setup = DoryProverPublicSetup::new(&prover_setup, 0);
        assert_eq!(setup.sigma(), 0);
    }

    #[test]
    fn dory_prover_setup_returns_same_prover_setup_reference() {
        let pp = test_params_nu_4();
        let prover_setup = ProverSetup::from(pp);
        let setup = DoryProverPublicSetup::new(&prover_setup, 2);
        assert!(core::ptr::eq(setup.prover_setup(), &prover_setup));
    }

    #[test]
    fn dory_prover_setup_is_copy() {
        let pp = test_params_nu_4();
        let prover_setup = ProverSetup::from(pp);
        let setup = DoryProverPublicSetup::new(&prover_setup, 7);
        let copied = setup;
        assert_eq!(copied.sigma(), 7);
    }
}
