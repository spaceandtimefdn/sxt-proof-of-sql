use super::{test_rng, ProverSetup, PublicParameters, VerifierSetup};
use std::sync::OnceLock;

pub(super) fn cached_public_parameters(max_nu: usize) -> &'static PublicParameters {
    fn get_or_init(
        cache: &'static OnceLock<PublicParameters>,
        max_nu: usize,
    ) -> &'static PublicParameters {
        cache.get_or_init(|| PublicParameters::test_rand(max_nu, &mut test_rng()))
    }

    static NU_2: OnceLock<PublicParameters> = OnceLock::new();
    static NU_4: OnceLock<PublicParameters> = OnceLock::new();
    static NU_6: OnceLock<PublicParameters> = OnceLock::new();

    match max_nu {
        2 => get_or_init(&NU_2, 2),
        4 => get_or_init(&NU_4, 4),
        6 => get_or_init(&NU_6, 6),
        _ => panic!("no cached Dory test setup for max_nu {max_nu}"),
    }
}

pub(super) fn cached_prover_setup(max_nu: usize) -> &'static ProverSetup<'static> {
    fn get_or_init(
        cache: &'static OnceLock<ProverSetup<'static>>,
        max_nu: usize,
    ) -> &'static ProverSetup<'static> {
        cache.get_or_init(|| ProverSetup::from(cached_public_parameters(max_nu)))
    }

    static NU_4: OnceLock<ProverSetup<'static>> = OnceLock::new();
    static NU_6: OnceLock<ProverSetup<'static>> = OnceLock::new();

    match max_nu {
        4 => get_or_init(&NU_4, 4),
        6 => get_or_init(&NU_6, 6),
        _ => panic!("no cached Dory prover setup for max_nu {max_nu}"),
    }
}

pub(super) fn cached_verifier_setup(max_nu: usize) -> &'static VerifierSetup {
    fn get_or_init(
        cache: &'static OnceLock<VerifierSetup>,
        max_nu: usize,
    ) -> &'static VerifierSetup {
        cache.get_or_init(|| VerifierSetup::from(cached_public_parameters(max_nu)))
    }

    static NU_2: OnceLock<VerifierSetup> = OnceLock::new();
    static NU_4: OnceLock<VerifierSetup> = OnceLock::new();
    static NU_6: OnceLock<VerifierSetup> = OnceLock::new();

    match max_nu {
        2 => get_or_init(&NU_2, 2),
        4 => get_or_init(&NU_4, 4),
        6 => get_or_init(&NU_6, 6),
        _ => panic!("no cached Dory verifier setup for max_nu {max_nu}"),
    }
}
