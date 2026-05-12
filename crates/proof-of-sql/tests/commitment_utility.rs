//! Integration tests for the commitment utility CLI.

use curve25519_dalek::ristretto::RistrettoPoint;
use proof_of_sql::{
    base::commitment::{ColumnCommitments, TableCommitment},
    proof_primitive::dory::{DoryCommitment, DynamicDoryCommitment},
};
use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};
use tempfile::tempdir;

fn commitment_utility_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commitment-utility"))
}

fn empty_commitment_bytes<C>() -> Vec<u8>
where
    C: proof_of_sql::base::commitment::Commitment + serde::Serialize,
{
    let commitment = TableCommitment::<C>::try_new(ColumnCommitments::default(), 0..0).unwrap();
    postcard::to_allocvec(&commitment).unwrap()
}

#[test]
fn default_scheme_reads_dynamic_dory_commitment_from_stdin() {
    let mut child = commitment_utility_command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&empty_commitment_bytes::<DynamicDoryCommitment>())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("TableCommitment"));
}

#[test]
fn selected_scheme_reads_commitment_from_input_file_and_writes_output_file() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("commitment.bin");
    let output_path = temp_dir.path().join("commitment.txt");
    fs::write(&input_path, empty_commitment_bytes::<RistrettoPoint>()).unwrap();

    let output = commitment_utility_command()
        .args([
            "--scheme",
            "ipa",
            "--input",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert!(fs::read_to_string(output_path)
        .unwrap()
        .contains("TableCommitment"));
}

#[test]
fn dory_scheme_is_deserialized() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("commitment.bin");
    fs::write(&input_path, empty_commitment_bytes::<DoryCommitment>()).unwrap();

    let output = commitment_utility_command()
        .args(["--scheme", "dory", "--input", input_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("TableCommitment"));
}

#[test]
fn invalid_bytes_return_deserialization_error() {
    let mut child = commitment_utility_command()
        .args(["--scheme", "dynamic-dory"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&[0xff]).unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("DeserializationError"));
}

#[test]
fn missing_input_file_returns_open_error() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("missing.bin");

    let output = commitment_utility_command()
        .args(["--input", input_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("OpenInputFile"));
}

#[test]
fn missing_output_parent_returns_create_error() {
    let temp_dir = tempdir().unwrap();
    let input_path = temp_dir.path().join("commitment.bin");
    let output_path = temp_dir.path().join("missing-dir").join("commitment.txt");
    fs::write(
        &input_path,
        empty_commitment_bytes::<DynamicDoryCommitment>(),
    )
    .unwrap();

    let output = commitment_utility_command()
        .args([
            "--input",
            input_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("CreateOutputFile"));
}
