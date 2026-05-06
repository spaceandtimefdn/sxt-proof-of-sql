use super::{test_rng, ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

// Dory tests repeatedly rebuild the same deterministic setup values. Cache them
// once per test binary to avoid repeating public-parameter generation, verifier
// pairings, and prover setup initialization.
macro_rules! cached_setups {
    ($max_nu:literal, $public_parameters:ident, $prover_setup:ident, $verifier_setup:ident) => {
        static $public_parameters: OnceLock<PublicParameters> = OnceLock::new();
        static $prover_setup: OnceLock<ProverSetup<'static>> = OnceLock::new();
        static $verifier_setup: OnceLock<VerifierSetup> = OnceLock::new();

        impl CachedSetup<$max_nu> {
            fn public_parameters() -> &'static PublicParameters {
                $public_parameters
                    .get_or_init(|| PublicParameters::test_rand($max_nu, &mut test_rng()))
            }

            fn prover_setup() -> &'static ProverSetup<'static> {
                $prover_setup.get_or_init(|| ProverSetup::from(Self::public_parameters()))
            }

            fn verifier_setup() -> &'static VerifierSetup {
                $verifier_setup.get_or_init(|| VerifierSetup::from(Self::public_parameters()))
            }
        }
    };
}

struct CachedSetup<const MAX_NU: usize>;

cached_setups!(4, PUBLIC_PARAMETERS_4, PROVER_SETUP_4, VERIFIER_SETUP_4);
cached_setups!(5, PUBLIC_PARAMETERS_5, PROVER_SETUP_5, VERIFIER_SETUP_5);
cached_setups!(6, PUBLIC_PARAMETERS_6, PROVER_SETUP_6, VERIFIER_SETUP_6);
cached_setups!(2, PUBLIC_PARAMETERS_2, PROVER_SETUP_2, VERIFIER_SETUP_2);
cached_setups!(3, PUBLIC_PARAMETERS_3, PROVER_SETUP_3, VERIFIER_SETUP_3);

#[must_use]
pub(crate) fn cached_public_parameters(max_nu: usize) -> &'static PublicParameters {
    match max_nu {
        2 => CachedSetup::<2>::public_parameters(),
        3 => CachedSetup::<3>::public_parameters(),
        4 => CachedSetup::<4>::public_parameters(),
        5 => CachedSetup::<5>::public_parameters(),
        6 => CachedSetup::<6>::public_parameters(),
        _ => panic!("unsupported cached Dory test setup size: {max_nu}"),
    }
}

#[must_use]
pub(crate) fn cached_prover_setup(max_nu: usize) -> &'static ProverSetup<'static> {
    match max_nu {
        2 => CachedSetup::<2>::prover_setup(),
        3 => CachedSetup::<3>::prover_setup(),
        4 => CachedSetup::<4>::prover_setup(),
        5 => CachedSetup::<5>::prover_setup(),
        6 => CachedSetup::<6>::prover_setup(),
        _ => panic!("unsupported cached Dory test setup size: {max_nu}"),
    }
}

#[must_use]
pub(crate) fn cached_verifier_setup(max_nu: usize) -> &'static VerifierSetup {
    match max_nu {
        2 => CachedSetup::<2>::verifier_setup(),
        3 => CachedSetup::<3>::verifier_setup(),
        4 => CachedSetup::<4>::verifier_setup(),
        5 => CachedSetup::<5>::verifier_setup(),
        6 => CachedSetup::<6>::verifier_setup(),
        _ => panic!("unsupported cached Dory test setup size: {max_nu}"),
    }
}
