use super::{test_rng, ProverSetup, PublicParameters, VerifierSetup};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

pub(crate) struct CachedDoryTestSetup {
    pub public_parameters: &'static PublicParameters,
    pub prover_setup: ProverSetup<'static>,
    pub verifier_setup: VerifierSetup,
}

pub(crate) fn cached_dory_test_setup(max_nu: usize) -> &'static CachedDoryTestSetup {
    static CACHE: OnceLock<Mutex<BTreeMap<usize, &'static CachedDoryTestSetup>>> = OnceLock::new();

    let cache = CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut cache = cache.lock().expect("cached dory test setup mutex poisoned");
    if let Some(setup) = cache.get(&max_nu).copied() {
        return setup;
    }

    // Dory tests only need one valid setup per `max_nu`; recomputing the same random setup
    // dominates runtime without increasing coverage.
    let public_parameters: &'static PublicParameters = Box::leak(Box::new(
        PublicParameters::test_rand(max_nu, &mut test_rng()),
    ));
    let setup = Box::leak(Box::new(CachedDoryTestSetup {
        public_parameters,
        prover_setup: ProverSetup::from(public_parameters),
        verifier_setup: VerifierSetup::from(public_parameters),
    }));
    cache.insert(max_nu, setup);
    setup
}
