use super::{rand_G_vecs, test_rng, ProverState, PublicParameters};
use ark_ec::pairing::Pairing;

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn prover_state_rejects_v1_with_wrong_length() {
    let mut rng = test_rng();
    let nu = 2;
    let (mut v1, v2) = rand_G_vecs(nu, &mut rng);
    v1.pop();

    ProverState::new(v1, v2, nu);
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn prover_state_rejects_v2_with_wrong_length() {
    let mut rng = test_rng();
    let nu = 2;
    let (v1, mut v2) = rand_G_vecs(nu, &mut rng);
    v2.pop();

    ProverState::new(v1, v2, nu);
}

#[test]
pub fn we_can_create_a_verifier_state_from_a_prover_state() {
    let mut rng = test_rng();
    let max_nu = 5;
    let pp = PublicParameters::test_rand(max_nu, &mut rng);
    let prover_setup = (&pp).into();
    for nu in 0..max_nu {
        let (v1, v2) = rand_G_vecs(nu, &mut rng);
        let prover_state = ProverState::new(v1.clone(), v2.clone(), nu);
        let verifier_state = prover_state.calculate_verifier_state(&prover_setup);

        let C = Pairing::multi_pairing(&v1, &v2);
        let D_1 = Pairing::multi_pairing(&v1, prover_setup.Gamma_2[nu]);
        let D_2 = Pairing::multi_pairing(prover_setup.Gamma_1[nu], &v2);

        assert_eq!(verifier_state.C, C);
        assert_eq!(verifier_state.D_1, D_1);
        assert_eq!(verifier_state.D_2, D_2);
        assert_eq!(verifier_state.nu, nu);
    }
}
