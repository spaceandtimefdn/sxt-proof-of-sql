use super::{test_rng, ProverSetup, PublicParameters};
use std::cell::OnceCell;

thread_local! {
    static TEST_DORY_SETUP: OnceCell<(&'static PublicParameters, &'static ProverSetup<'static>)> =
        const { OnceCell::new() };
}

pub(crate) fn test_dory_setup() -> (&'static PublicParameters, &'static ProverSetup<'static>) {
    TEST_DORY_SETUP.with(|setup| {
        *setup.get_or_init(|| {
            let public_parameters =
                Box::leak(Box::new(PublicParameters::test_rand(5, &mut test_rng())));
            let prover_setup = Box::leak(Box::new(ProverSetup::from(&*public_parameters)));

            (public_parameters, prover_setup)
        })
    })
}
