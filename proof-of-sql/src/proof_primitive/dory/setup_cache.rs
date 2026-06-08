// Caching for Dory setups
use std::fs;
use std::path::Path;
use std::sync::Once;
use crate::proof_primitive::dory::{PublicParameters, ProverSetup, VerifierSetup};

static INIT: Once = Once::new();

fn setup_cache_dir() {
    INIT.call_once(|| {
        let _ = fs::create_dir_all(".dory_cache");
    });
}

fn cache_path(name: &str) -> String {
    format!(".dory_cache/{}.bin", name)
}

pub fn load_public_parameters(size: usize) -> Option<PublicParameters> {
    setup_cache_dir();
    let name = format!("public_parameters_{}", size);
    let path = cache_path(&name);
    if Path::new(&path).exists() {
        let data = fs::read(path).ok()?;
        bincode::deserialize(&data).ok()
    } else {
        None
    }
}

pub fn save_public_parameters(size: usize, pp: &PublicParameters) {
    setup_cache_dir();
    let name = format!("public_parameters_{}", size);
    let path = cache_path(&name);
    let data = bincode::serialize(pp).unwrap();
    let _ = fs::write(path, data);
}

pub fn load_prover_setup(size: usize) -> Option<ProverSetup> {
    setup_cache_dir();
    let name = format!("prover_setup_{}", size);
    let path = cache_path(&name);
    if Path::new(&path).exists() {
        let data = fs::read(path).ok()?;
        bincode::deserialize(&data).ok()
    } else {
        None
    }
}

pub fn save_prover_setup(size: usize, ps: &ProverSetup) {
    setup_cache_dir();
    let name = format!("prover_setup_{}", size);
    let path = cache_path(&name);
    let data = bincode::serialize(ps).unwrap();
    let _ = fs::write(path, data);
}

pub fn load_verifier_setup(size: usize) -> Option<VerifierSetup> {
    setup_cache_dir();
    let name = format!("verifier_setup_{}", size);
    let path = cache_path(&name);
    if Path::new(&path).exists() {
        let data = fs::read(path).ok()?;
        bincode::deserialize(&data).ok()
    } else {
        None
    }
}

pub fn save_verifier_setup(size: usize, vs: &VerifierSetup) {
    setup_cache_dir();
    let name = format!("verifier_setup_{}", size);
    let path = cache_path(&name);
    let data = bincode::serialize(vs).unwrap();
    let _ = fs::write(path, data);
}
