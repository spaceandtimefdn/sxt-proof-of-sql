use curve25519_dalek::ristretto::RistrettoPoint;
use proof_of_sql::base::commitment::TableCommitment;
use std::{
    fs,
    process::{Command, Stdio},
};
use tempfile::tempdir;

#[test]
fn generate_parameters_can_create_verifier_artifacts_and_refresh_digests() {
    let temp_dir = tempdir().expect("temp dir");
    let target = temp_dir.path();
    let binary = env!("CARGO_BIN_EXE_generate-parameters");

    let first_run = Command::new(binary)
        .args(["--nu", "1", "--mode", "verifier", "--target"])
        .arg(target)
        .output()
        .expect("run generate-parameters");
    assert!(
        first_run.status.success(),
        "first run failed: {first_run:?}"
    );

    let verifier_setup = target.join("verifier_setup_nu_1.bin");
    let digests = target.join("digests_nu_1.txt");
    assert!(verifier_setup.exists(), "missing verifier setup");
    assert!(digests.exists(), "missing digests file");

    let first_digest_contents = fs::read_to_string(&digests).expect("read digests");
    assert!(first_digest_contents.contains("verifier_setup_nu_1.bin"));

    let second_run = Command::new(binary)
        .args(["--nu", "1", "--mode", "verifier", "--target"])
        .arg(target)
        .output()
        .expect("rerun generate-parameters");
    assert!(
        second_run.status.success(),
        "second run failed: {second_run:?}"
    );

    let refreshed_digest_contents = fs::read_to_string(&digests).expect("read refreshed digests");
    assert_eq!(refreshed_digest_contents.lines().count(), 1);
}

#[test]
fn generate_parameters_fails_when_target_is_a_file() {
    let temp_dir = tempdir().expect("temp dir");
    let target_file = temp_dir.path().join("not-a-directory");
    fs::write(&target_file, b"occupied").expect("write target file");

    let output = Command::new(env!("CARGO_BIN_EXE_generate-parameters"))
        .args(["--nu", "1", "--mode", "verifier", "--target"])
        .arg(&target_file)
        .output()
        .expect("run generate-parameters with invalid target");

    assert!(!output.status.success());
}

#[test]
fn commitment_utility_can_deserialize_ipa_commitments_to_a_file() {
    let temp_dir = tempdir().expect("temp dir");
    let input_path = temp_dir.path().join("commitment.bin");
    let output_path = temp_dir.path().join("commitment.txt");
    let encoded = postcard::to_allocvec(&TableCommitment::<RistrettoPoint>::default())
        .expect("serialize empty table commitment");
    fs::write(&input_path, encoded).expect("write encoded commitment");

    let output = Command::new(env!("CARGO_BIN_EXE_commitment-utility"))
        .args(["--scheme", "ipa", "--input"])
        .arg(&input_path)
        .args(["--output"])
        .arg(&output_path)
        .output()
        .expect("run commitment-utility");

    assert!(output.status.success(), "utility failed: {output:?}");
    let rendered = fs::read_to_string(&output_path).expect("read output");
    assert!(rendered.contains("TableCommitment"));
}

#[test]
fn commitment_utility_rejects_invalid_input_from_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_commitment-utility"))
        .args(["--scheme", "ipa"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn commitment-utility");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        use std::io::Write;
        stdin.write_all(b"definitely-not-postcard")
            .expect("write invalid bytes");
    }

    let output = child.wait_with_output().expect("wait on child");
    assert!(!output.status.success());
}

#[test]
fn commitment_utility_reports_missing_input_files() {
    let missing = tempdir()
        .expect("temp dir")
        .path()
        .join("missing-commitment.bin");
    let output = Command::new(env!("CARGO_BIN_EXE_commitment-utility"))
        .args(["--scheme", "ipa", "--input"])
        .arg(&missing)
        .output()
        .expect("run commitment-utility with missing input");

    assert!(!output.status.success());
}
