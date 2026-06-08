//! Shared Dory setup cache for tests
use once_cell::sync::Lazy;
use std::sync::Mutex;
use proof_of_sql::proof_primitive::dory::{PublicParameters, ProverSetup, VerifierSetup};

/// Cached Dory public parameters for tests
pub static DORY_PP: Lazy<Mutex<Option<PublicParameters>>> = Lazy::new(|| Mutex::new(None));
/// Cached Dory prover setup for tests
pub static DORY_PROVER: Lazy<Mutex<Option<ProverSetup>>> = Lazy::new(|| Mutex::new(None));
/// Cached Dory verifier setup for tests
pub static DORY_VERIFIER: Lazy<Mutex<Option<VerifierSetup>>> = Lazy::new(|| Mutex::new(None));

/// Get or generate cached Dory public parameters
pub fn get_dory_pp() -> PublicParameters {
    let mut cache = DORY_PP.lock().unwrap();
    if let Some(pp) = &*cache {
        pp.clone()
    } else {
        let pp = PublicParameters::test_rand();
        *cache = Some(pp.clone());
        pp
    }
}

/// Get or generate cached Dory prover setup
pub fn get_dory_prover() -> ProverSetup {
    let mut cache = DORY_PROVER.lock().unwrap();
    if let Some(ps) = &*cache {
        ps.clone()
    } else {
        let pp = get_dory_pp();
        let ps = ProverSetup::from(&pp);
        *cache = Some(ps.clone());
        ps
    }
}

/// Get or generate cached Dory verifier setup
pub fn get_dory_verifier() -> VerifierSetup {
    let mut cache = DORY_VERIFIER.lock().unwrap();
    if let Some(vs) = &*cache {
        vs.clone()
    } else {
        let pp = get_dory_pp();
        let vs = VerifierSetup::from(&pp);
        *cache = Some(vs.clone());
        vs
    }
}
