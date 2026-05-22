use super::{
    test_rng, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup, PublicParameters,
    VerifierSetup,
};

#[test]
fn prover_public_setup_returns_its_sigma_and_setup() {
    let public_parameters = PublicParameters::test_rand(3, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let public_setup = DoryProverPublicSetup::new(&prover_setup, 2);

    assert_eq!(public_setup.sigma(), 2);
    assert!(core::ptr::eq(public_setup.prover_setup(), &prover_setup));
}

#[test]
fn verifier_public_setup_returns_its_sigma_and_setup() {
    let public_parameters = PublicParameters::test_rand(3, &mut test_rng());
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let public_setup = DoryVerifierPublicSetup::new(&verifier_setup, 1);

    assert_eq!(public_setup.sigma(), 1);
    assert!(core::ptr::eq(
        public_setup.verifier_setup(),
        &verifier_setup
    ));
}
