use super::{
    compute_sha256 as utility_compute_sha256, rng_from_seed, save_digests, Args, Mode, SEED,
};
use ark_std::rand::RngCore;
use clap::Parser;
use proof_of_sql::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
    process::Command,
};
use tempfile::tempdir;

#[test]
fn we_can_parse_default_args() {
    let args = Args::try_parse_from(["generate-parameters"]).unwrap();

    assert_eq!(args.nu, 8);
    assert!(matches!(args.mode, Mode::All));
    assert_eq!(args.seed, SEED);
    assert_eq!(args.target, "./output");
}

#[test]
fn we_can_parse_custom_args() {
    let args = Args::try_parse_from([
        "generate-parameters",
        "--nu",
        "5",
        "--mode",
        "verifier",
        "--seed",
        "custom",
        "--target",
        "./custom-output",
    ])
    .unwrap();

    assert_eq!(args.nu, 5);
    assert!(matches!(args.mode, Mode::Verifier));
    assert_eq!(args.seed, "custom");
    assert_eq!(args.target, "./custom-output");
}

#[test]
fn we_can_seed_rng_deterministically() {
    let args = Args::try_parse_from(["generate-parameters", "--seed", "abc"]).unwrap();
    let mut first_rng = rng_from_seed(&args);
    let mut second_rng = rng_from_seed(&args);

    assert_eq!(first_rng.next_u64(), second_rng.next_u64());
    assert_eq!(first_rng.next_u64(), second_rng.next_u64());
}

#[test]
fn we_can_compute_sha256_for_existing_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("data.txt");
    std::fs::write(&file_path, b"proof-of-sql").unwrap();

    assert_eq!(
        utility_compute_sha256(file_path.to_str().unwrap()).unwrap(),
        "58b1a9cc252e31aada59534f24bbd4b9b1b0cbd95dd1fbe5c38bb38577faeaac"
    );
}

#[test]
fn compute_sha256_returns_none_for_missing_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("missing.txt");

    assert!(utility_compute_sha256(file_path.to_str().unwrap()).is_none());
}

#[test]
fn we_can_save_and_read_digest_entries() {
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_str().unwrap();
    let first_path = temp_dir.path().join("first.bin");
    let second_path = temp_dir.path().join("second.bin");

    let digests = vec![
        (
            first_path.to_string_lossy().into_owned(),
            "abc123".to_string(),
        ),
        (
            second_path.to_string_lossy().into_owned(),
            "def456".to_string(),
        ),
    ];

    save_digests(&digests, target, 3);

    let digest_path = temp_dir.path().join("digests_nu_3.txt");
    let entries = read_digests_from_file(digest_path.to_str().unwrap());

    assert_eq!(entries.get(first_path.to_str().unwrap()).unwrap(), "abc123");
    assert_eq!(
        entries.get(second_path.to_str().unwrap()).unwrap(),
        "def456"
    );
}

#[test]
fn read_digests_ignores_malformed_entries() {
    let temp_dir = tempdir().unwrap();
    let digest_path = temp_dir.path().join("digests.txt");
    let mut file = File::create(&digest_path).unwrap();
    writeln!(file, "abc123  /tmp/valid.bin").unwrap();
    writeln!(file, "malformed only has too many fields").unwrap();
    writeln!(file, "missing_path").unwrap();

    let entries = read_digests_from_file(digest_path.to_str().unwrap());

    assert_eq!(entries.len(), 1);
    assert_eq!(entries.get("/tmp/valid.bin").unwrap(), "abc123");
}

/// # Panics
/// This test will panic in a number of non-consequential, expected cases.
#[test]
#[ignore = "Requires running external binary"]
fn we_can_generate_save_and_load_public_setups() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create a temporary directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run the binary with nu = 4, mode = "pv", and target as the temp directory
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("generate-parameters")
        .arg("--release")
        .arg("--")
        .arg("--nu")
        .arg("4")
        .arg("--mode")
        .arg("all")
        .arg("--target")
        .arg(temp_path)
        .output()
        .expect("Failed to execute command");

    // Check the output to make sure the process ran successfully
    assert!(output.status.success(), "Process failed to run: {output:?}");

    // Check that both Prover and Verifier files exist in the temp directory
    let blitzar_handle_path = format!("{temp_path}/blitzar_handle_nu_4.bin");
    let verifier_setup_path = format!("{temp_path}/verifier_setup_nu_4.bin");
    let public_parameters_path = format!("{temp_path}/public_parameters_nu_4.bin");
    let digests_path = format!("{temp_path}/digests_nu_4.txt");

    assert!(
        Path::new(&blitzar_handle_path).exists(),
        "Prover setup file is missing"
    );
    assert!(
        Path::new(&verifier_setup_path).exists(),
        "Verifier setup file is missing"
    );
    assert!(
        Path::new(&public_parameters_path).exists(),
        "Public parameters file is missing"
    );
    assert!(Path::new(&digests_path).exists(), "Digests file is missing");

    // Load the ProverSetup and VerifierSetup from their files
    let handle = blitzar::compute::MsmHandle::new_from_file(&blitzar_handle_path);
    let params = PublicParameters::load_from_file(Path::new(&public_parameters_path)).unwrap();

    let _prover_setup = ProverSetup::from_public_parameters_and_blitzar_handle(&params, handle);
    let _verifier_setup = VerifierSetup::load_from_file(Path::new(&verifier_setup_path))
        .expect("Failed to load VerifierSetup");

    // Verify that the digests.txt file contains the correct hash values
    let mut expected_digests = Vec::new();

    // Compute SHA-256 digests for each file
    if let Some(digest) = compute_sha256(&public_parameters_path) {
        expected_digests.push((public_parameters_path.clone(), digest));
    }
    if let Some(digest) = compute_sha256(&blitzar_handle_path) {
        expected_digests.push((blitzar_handle_path.clone(), digest));
    }
    if let Some(digest) = compute_sha256(&verifier_setup_path) {
        expected_digests.push((verifier_setup_path.clone(), digest));
    }

    // Read and parse digests from the file
    let actual_digests = read_digests_from_file(&digests_path);

    // Compare expected digests to those read from digests.txt
    for (file_path, expected_digest) in &expected_digests {
        let actual_digest = actual_digests
            .get(file_path)
            .unwrap_or_else(|| panic!("Digest for {file_path} not found in digests.txt"));
        assert_eq!(
            actual_digest, expected_digest,
            "Digest mismatch for {file_path}"
        );
    }
}

/// Compute SHA-256 hash of a file and return it as a hex string.
fn compute_sha256(file_path: &str) -> Option<String> {
    let mut file = File::open(file_path).ok()?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).ok()?;
    Some(format!("{:x}", hasher.finalize()))
}

/// Read digests from the digests file and return them as a `HashMap`.
/// # Panics
/// because it is a test and is allowed to panic
fn read_digests_from_file(digests_path: &str) -> std::collections::HashMap<String, String> {
    let file = File::open(digests_path).expect("Failed to open digests file");
    let reader = io::BufReader::new(file);
    let mut digests = std::collections::HashMap::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line from digests file");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let digest = parts[0].to_string();
            let file_path = parts[1].to_string();
            digests.insert(file_path, digest);
        }
    }
    digests
}
