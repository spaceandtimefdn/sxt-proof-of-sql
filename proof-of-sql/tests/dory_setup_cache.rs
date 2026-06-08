//! Test helper for caching Dory setups to speed up tests.
use once_cell::sync::Lazy;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use proof_of_sql::proof_primitive::dory::{PublicParameters, ProverSetup, VerifierSetup};

static DORY_CACHE: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn cache_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push("sxt-proof-of-sql-dory-cache");
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn cached_public_parameters(size: usize) -> PublicParameters {
    let _lock = DORY_CACHE.lock().unwrap();
    let mut path = cache_dir();
    path.push(format!("public_parameters_{}.bin", size));
    if path.exists() {
        let mut file = fs::File::open(&path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        bincode::deserialize(&buf).unwrap()
    } else {
        let pp = PublicParameters::test_rand(size);
        let buf = bincode::serialize(&pp).unwrap();
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(&buf).unwrap();
        pp
    }
}

pub fn cached_prover_setup(pp: &PublicParameters, n: usize) -> ProverSetup {
    let _lock = DORY_CACHE.lock().unwrap();
    let mut path = cache_dir();
    path.push(format!("prover_setup_{}_{}.bin", pp.n, n));
    if path.exists() {
        let mut file = fs::File::open(&path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        bincode::deserialize(&buf).unwrap()
    } else {
        let ps = ProverSetup::from(pp, n);
        let buf = bincode::serialize(&ps).unwrap();
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(&buf).unwrap();
        ps
    }
}

pub fn cached_verifier_setup(pp: &PublicParameters, n: usize) -> VerifierSetup {
    let _lock = DORY_CACHE.lock().unwrap();
    let mut path = cache_dir();
    path.push(format!("verifier_setup_{}_{}.bin", pp.n, n));
    if path.exists() {
        let mut file = fs::File::open(&path).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        bincode::deserialize(&buf).unwrap()
    } else {
        let vs = VerifierSetup::from(pp, n);
        let buf = bincode::serialize(&vs).unwrap();
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(&buf).unwrap();
        vs
    }
}
