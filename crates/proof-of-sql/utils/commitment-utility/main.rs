//! Utility to deserialize and print a commitment from a file or stdin.
use clap::{Parser, ValueEnum};
use curve25519_dalek::ristretto::RistrettoPoint;
use proof_of_sql::{
    base::commitment::TableCommitment,
    proof_primitive::dory::{DoryCommitment, DynamicDoryCommitment},
};
use snafu::Snafu;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(ValueEnum, Clone, Debug)]
/// Supported commitment schemes.
enum CommitmentScheme {
    /// Inner Product Argument (IPA) commitment scheme.
    Ipa,
    /// Dory commitment scheme.
    Dory,
    /// Dynamic Dory commitment scheme.
    DynamicDory,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file (defaults to None which is stdin)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file (defaults to None which is stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Commitment scheme (e.g. `ipa`, `dynamic_dory`, `dory`)
    #[arg(long, value_enum, default_value = "CommitmentScheme::DynamicDory")]
    scheme: CommitmentScheme,
}

#[derive(Debug, Snafu)]
enum CommitUtilityError {
    #[snafu(display("Failed to open input file '{:?}'", filename))]
    OpenInputFile { filename: PathBuf },

    #[snafu(display("Failed to read from input file '{:?}'", filename))]
    ReadInputFile { filename: PathBuf },

    #[snafu(display("Failed to read from stdin"))]
    ReadStdin,

    #[snafu(display("Failed to create output file '{:?}'", filename))]
    CreateOutputFile { filename: PathBuf },

    #[snafu(display("Failed to write to output file '{:?}'", filename))]
    WriteOutputFile { filename: PathBuf },

    #[snafu(display("Failed to write to stdout"))]
    WriteStdout,

    #[snafu(display("Failed to deserialize commitment"))]
    DeserializationError,
}

type CommitUtilityResult<T, E = CommitUtilityError> = std::result::Result<T, E>;

fn main() -> CommitUtilityResult<()> {
    let cli = Cli::parse();
    run(&cli)
}

fn run(cli: &Cli) -> CommitUtilityResult<()> {
    // Read input data
    let input_data = read_input(cli.input.as_ref())?;

    // Deserialize commitment based on the scheme
    let human_readable = deserialize_commitment(&input_data, &cli.scheme)?;

    // Write output data
    write_output(cli.output.as_ref(), &human_readable)
}

fn read_input(input: Option<&PathBuf>) -> CommitUtilityResult<Vec<u8>> {
    if let Some(input_file) = input {
        let mut file = File::open(input_file).map_err(|_| CommitUtilityError::OpenInputFile {
            filename: input_file.clone(),
        })?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|_| CommitUtilityError::ReadInputFile {
                filename: input_file.clone(),
            })?;
        Ok(buffer)
    } else {
        let mut buffer = Vec::new();
        io::stdin()
            .read_to_end(&mut buffer)
            .map_err(|_| CommitUtilityError::ReadStdin)?;
        Ok(buffer)
    }
}

fn deserialize_commitment(
    input_data: &[u8],
    scheme: &CommitmentScheme,
) -> CommitUtilityResult<String> {
    match scheme {
        CommitmentScheme::DynamicDory => {
            let commitment: TableCommitment<DynamicDoryCommitment> =
                postcard::from_bytes(input_data)
                    .map_err(|_| CommitUtilityError::DeserializationError)?;
            Ok(format!("{commitment:#?}"))
        }
        CommitmentScheme::Dory => {
            let commitment: TableCommitment<DoryCommitment> = postcard::from_bytes(input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            Ok(format!("{commitment:#?}"))
        }
        CommitmentScheme::Ipa => {
            let commitment: TableCommitment<RistrettoPoint> = postcard::from_bytes(input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            Ok(format!("{commitment:#?}"))
        }
    }
}

fn write_output(output: Option<&PathBuf>, human_readable: &str) -> CommitUtilityResult<()> {
    match output {
        Some(output_file) => {
            let mut file =
                File::create(output_file).map_err(|_| CommitUtilityError::CreateOutputFile {
                    filename: output_file.clone(),
                })?;
            file.write_all(human_readable.as_bytes()).map_err(|_| {
                CommitUtilityError::WriteOutputFile {
                    filename: output_file.clone(),
                }
            })?;
        }
        None => {
            io::stdout()
                .write_all(human_readable.as_bytes())
                .map_err(|_| CommitUtilityError::WriteStdout)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn serialized_empty_commitment<C>() -> Vec<u8>
    where
        C: proof_of_sql::base::commitment::Commitment,
        TableCommitment<C>: serde::Serialize,
    {
        postcard::to_allocvec(&TableCommitment::<C>::default()).unwrap()
    }

    #[test]
    fn we_can_render_dynamic_dory_commitment() {
        let rendered = deserialize_commitment(
            &serialized_empty_commitment::<DynamicDoryCommitment>(),
            &CommitmentScheme::DynamicDory,
        )
        .unwrap();

        assert!(rendered.contains("TableCommitment"));
        assert!(rendered.contains("column_commitments"));
    }

    #[test]
    fn we_can_render_dory_commitment() {
        let rendered = deserialize_commitment(
            &serialized_empty_commitment::<DoryCommitment>(),
            &CommitmentScheme::Dory,
        )
        .unwrap();

        assert!(rendered.contains("TableCommitment"));
        assert!(rendered.contains("range"));
    }

    #[test]
    fn we_can_render_ipa_commitment() {
        let rendered = deserialize_commitment(
            &serialized_empty_commitment::<RistrettoPoint>(),
            &CommitmentScheme::Ipa,
        )
        .unwrap();

        assert!(rendered.contains("TableCommitment"));
        assert!(rendered.contains("range"));
    }

    #[test]
    fn we_reject_invalid_commitment_bytes() {
        let err = deserialize_commitment(&[0xff], &CommitmentScheme::DynamicDory).unwrap_err();

        assert!(matches!(err, CommitUtilityError::DeserializationError));
    }

    #[test]
    fn we_can_read_input_from_file() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.bin");
        std::fs::write(&input_path, [1_u8, 2, 3]).unwrap();

        assert_eq!(read_input(Some(&input_path)).unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn we_report_missing_input_file() {
        let missing = PathBuf::from("/no/such/commitment.bin");
        let err = read_input(Some(&missing)).unwrap_err();

        assert!(matches!(err, CommitUtilityError::OpenInputFile { .. }));
    }

    #[test]
    fn we_can_write_output_to_file() {
        let dir = tempdir().unwrap();
        let output_path = dir.path().join("output.txt");

        write_output(Some(&output_path), "commitment").unwrap();

        assert_eq!(std::fs::read_to_string(output_path).unwrap(), "commitment");
    }
}
