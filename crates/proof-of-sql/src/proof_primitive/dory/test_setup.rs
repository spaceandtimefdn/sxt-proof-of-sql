use super::{rand_util::test_seed_rng, ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

pub(crate) struct TestDorySetup {
    pub(crate) prover_setup: ProverSetup<'static>,
    pub(crate) verifier_setup: VerifierSetup,
}

pub(crate) fn test_dory_setup(max_nu: usize) -> &'static TestDorySetup {
    match max_nu {
        2 => setup_for::<2>(),
        4 => setup_for::<4>(),
        5 => setup_for::<5>(),
        6 => setup_for::<6>(),
        _ => panic!("missing cached Dory test setup for nu {max_nu}"),
    }
}

fn setup_for<const MAX_NU: usize>() -> &'static TestDorySetup {
    static SETUP_2: OnceLock<TestDorySetup> = OnceLock::new();
    static SETUP_4: OnceLock<TestDorySetup> = OnceLock::new();
    static SETUP_5: OnceLock<TestDorySetup> = OnceLock::new();
    static SETUP_6: OnceLock<TestDorySetup> = OnceLock::new();

    match MAX_NU {
        2 => SETUP_2.get_or_init(|| setup_with_max_nu(MAX_NU)),
        4 => SETUP_4.get_or_init(|| setup_with_max_nu(MAX_NU)),
        5 => SETUP_5.get_or_init(|| setup_with_max_nu(MAX_NU)),
        6 => SETUP_6.get_or_init(|| setup_with_max_nu(MAX_NU)),
        _ => unreachable!("unsupported cached Dory test setup"),
    }
}

fn setup_with_max_nu(max_nu: usize) -> TestDorySetup {
    let mut rng = test_seed_rng([max_nu as u8; 32]);
    let public_parameters =
        Box::leak(Box::new(PublicParameters::test_rand(max_nu, &mut rng)));
    let prover_setup = ProverSetup::from(&*public_parameters);
    let verifier_setup = VerifierSetup::from(&*public_parameters);

    TestDorySetup {
        prover_setup,
        verifier_setup,
    }
}
