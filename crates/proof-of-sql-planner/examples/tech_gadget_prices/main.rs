//! This is a non-interactive example of using Proof of SQL with a tech gadget prices dataset.
//! To run this, use `cargo run --release --example tech_gadget_prices`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example tech_gadget_prices --no-default-features --features="cpu-perf"` instead. It will be slower for proof generation.
use datafusion::{
    arrow::{
        csv::{infer_schema_from_files, ReaderBuilder},
        datatypes::SchemaRef,
        record_batch::RecordBatch,
        util::pretty::pretty_format_batches,
    },
    config::ConfigOptions,
};
use proof_of_sql::{
    base::database::{OwnedTable, OwnedTableTestAccessor, TableRef},
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::proof::VerifiableQueryResult,
};
use proof_of_sql_planner::sql_to_proof_plans;
use rand::{rngs::StdRng, SeedableRng};
use sqlparser::{dialect::GenericDialect, parser::Parser};
use std::{fs::File, time::Instant};

// We generate the public parameters and the setups used by the prover and verifier for the Dory PCS.
// The `max_nu` should be set such that the maximum table size is less than `2^(2*max_nu-1)`.
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"tech-gadget-prices-dataset-seed!";

/// # Panics
/// Will panic if the query does not parse or the proof fails to verify.
fn prove_and_verify_query(
    sql: &str,
    accessor: &OwnedTableTestAccessor<DynamicDoryEvaluationProof>,
    prover_setup: &ProverSetup,
    verifier_setup: &VerifierSetup,
) {
    // Parse the query:
    println!("Parsing the query: {sql}...");
    let now = Instant::now();
    let config = ConfigOptions::default();
    let statements = Parser::parse_sql(&GenericDialect {}, sql).unwrap();
    let query_plan = &sql_to_proof_plans(&statements, accessor, &config).unwrap()[0];
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Generate the proof and result:
    print!("Generating proof...");
    let now = Instant::now();
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query_plan,
        accessor,
        &prover_setup,
        &[],
    )
    .unwrap();
    println!("Done in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Verify the result with the proof:
    print!("Verifying proof...");
    let now = Instant::now();
    let result: RecordBatch = verifiable_result
        .verify(query_plan, accessor, &verifier_setup, &[])
        .unwrap()
        .table
        .try_into()
        .unwrap();
    println!("Verified in {} ms.", now.elapsed().as_secs_f64() * 1000.);

    // Display the result
    println!("Query Result:");
    println!("{}", pretty_format_batches(&[result]).unwrap());
}

fn main() {
    let mut rng = StdRng::from_seed(DORY_SEED);
    let public_parameters = PublicParameters::rand(DORY_SETUP_MAX_NU, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    let filename = "crates/proof-of-sql-planner/examples/tech_gadget_prices/tech_gadget_prices.csv";
    let schema = infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap();
    let data_batch = ReaderBuilder::new(SchemaRef::new(schema))
        .with_header(true)
        .build(File::open(filename).unwrap())
        .unwrap()
        .next()
        .ok_or("No data found in CSV file")
        .unwrap()
        .unwrap();

    let accessor = OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_from_table(
        TableRef::from_names(None, "prices"),
        OwnedTable::try_from(data_batch).unwrap(),
        0,
        &prover_setup,
    );

    prove_and_verify_query(
        "SELECT COUNT(*) AS total FROM prices",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Brand, COUNT(*) AS total FROM prices GROUP BY Brand",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Name, Price FROM prices WHERE Category = 'Smartphone'",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
    prove_and_verify_query(
        "SELECT Name, ReleaseYear FROM prices WHERE Price > 500",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
