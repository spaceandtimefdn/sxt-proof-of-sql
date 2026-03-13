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
    run(cli, &mut io::stdin(), &mut io::stdout())
}

fn run(cli: Cli, stdin: &mut impl Read, stdout: &mut impl Write) -> CommitUtilityResult<()> {
    let input_data = read_input(cli.input.as_ref(), stdin)?;
    let human_readable = render_commitment(&input_data, &cli.scheme)?;
    write_output(cli.output.as_ref(), &human_readable, stdout)
}

fn read_input(input: Option<&PathBuf>, stdin: &mut impl Read) -> CommitUtilityResult<Vec<u8>> {
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
        stdin
            .read_to_end(&mut buffer)
            .map_err(|_| CommitUtilityError::ReadStdin)?;
        Ok(buffer)
    }
}

fn render_commitment(
    input_data: &[u8],
    scheme: &CommitmentScheme,
) -> CommitUtilityResult<String> {
    match scheme {
        CommitmentScheme::DynamicDory => {
            let commitment: TableCommitment<DynamicDoryCommitment> =
                postcard::from_bytes(&input_data)
                    .map_err(|_| CommitUtilityError::DeserializationError)?;
            Ok(format!("{commitment:#?}"))
        }
        CommitmentScheme::Dory => {
            let commitment: TableCommitment<DoryCommitment> = postcard::from_bytes(&input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            Ok(format!("{commitment:#?}"))
        }
        CommitmentScheme::Ipa => {
            let commitment: TableCommitment<RistrettoPoint> = postcard::from_bytes(&input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            Ok(format!("{commitment:#?}"))
        }
    }
}

fn write_output(
    output: Option<&PathBuf>,
    human_readable: &str,
    stdout: &mut impl Write,
) -> CommitUtilityResult<()> {
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
            stdout
                .write_all(human_readable.as_bytes())
                .map_err(|_| CommitUtilityError::WriteStdout)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proof_of_sql::proof_primitive::dory::{DoryCommitment, DynamicDoryCommitment};
    use std::{
        fs,
        io::{self, Cursor, Read, Write},
    };
    use tempfile::tempdir;

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("boom"))
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("boom"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn we_can_read_input_from_a_file_or_stdin() {
        let temp_dir = tempdir().expect("temp dir");
        let input_path = temp_dir.path().join("commitment.bin");
        fs::write(&input_path, b"abc").expect("write input");

        let mut ignored_stdin = Cursor::new(b"stdin".to_vec());
        assert_eq!(
            read_input(Some(&input_path), &mut ignored_stdin).unwrap(),
            b"abc"
        );

        let mut stdin = Cursor::new(b"stdin".to_vec());
        assert_eq!(read_input(None, &mut stdin).unwrap(), b"stdin");
    }

    #[test]
    fn we_report_missing_input_and_deserialization_failures() {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let missing = PathBuf::from("definitely-missing.bin");
        assert!(matches!(
            read_input(Some(&missing), &mut stdin),
            Err(CommitUtilityError::OpenInputFile { .. })
        ));

        assert!(matches!(
            read_input(None, &mut FailingReader),
            Err(CommitUtilityError::ReadStdin)
        ));

        assert!(matches!(
            render_commitment(b"not-postcard", &CommitmentScheme::Ipa),
            Err(CommitUtilityError::DeserializationError)
        ));
    }

    #[test]
    fn we_can_render_all_supported_commitment_schemes() {
        let ipa = postcard::to_allocvec(&TableCommitment::<RistrettoPoint>::default())
            .expect("serialize ipa");
        let dory = postcard::to_allocvec(&TableCommitment::<DoryCommitment>::default())
            .expect("serialize dory");
        let dynamic = postcard::to_allocvec(&TableCommitment::<DynamicDoryCommitment>::default())
            .expect("serialize dynamic dory");

        assert!(render_commitment(&ipa, &CommitmentScheme::Ipa)
            .unwrap()
            .contains("TableCommitment"));
        assert!(render_commitment(&dory, &CommitmentScheme::Dory)
            .unwrap()
            .contains("TableCommitment"));
        assert!(render_commitment(&dynamic, &CommitmentScheme::DynamicDory)
            .unwrap()
            .contains("TableCommitment"));
    }

    #[test]
    fn we_can_write_output_to_a_file_or_stdout() {
        let temp_dir = tempdir().expect("temp dir");
        let output_path = temp_dir.path().join("commitment.txt");
        let mut stdout = Cursor::new(Vec::<u8>::new());

        write_output(Some(&output_path), "file body", &mut stdout).unwrap();
        assert_eq!(fs::read_to_string(&output_path).unwrap(), "file body");

        write_output(None, "stdout body", &mut stdout).unwrap();
        assert_eq!(String::from_utf8(stdout.into_inner()).unwrap(), "stdout body");
    }

    #[test]
    fn we_report_output_failures() {
        let temp_dir = tempdir().expect("temp dir");
        let missing_parent = temp_dir.path().join("missing").join("commitment.txt");
        let mut stdout = Cursor::new(Vec::<u8>::new());

        assert!(matches!(
            write_output(Some(&missing_parent), "body", &mut stdout),
            Err(CommitUtilityError::CreateOutputFile { .. })
        ));
        assert!(matches!(
            write_output(None, "body", &mut FailingWriter),
            Err(CommitUtilityError::WriteStdout)
        ));
    }

    #[test]
    fn we_can_run_the_full_commitment_utility_flow() {
        let temp_dir = tempdir().expect("temp dir");
        let input_path = temp_dir.path().join("commitment.bin");
        let output_path = temp_dir.path().join("commitment.txt");
        let encoded = postcard::to_allocvec(&TableCommitment::<RistrettoPoint>::default())
            .expect("serialize empty table commitment");
        fs::write(&input_path, encoded).expect("write encoded commitment");

        let cli = Cli {
            input: Some(input_path),
            output: Some(output_path.clone()),
            scheme: CommitmentScheme::Ipa,
        };
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Cursor::new(Vec::<u8>::new());

        run(cli, &mut stdin, &mut stdout).unwrap();
        assert!(fs::read_to_string(output_path)
            .unwrap()
            .contains("TableCommitment"));
    }
}
