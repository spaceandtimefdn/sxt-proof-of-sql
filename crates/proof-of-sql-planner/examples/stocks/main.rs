//! This is a non-interactive example of using Proof of SQL with a stocks dataset.
//! To run this, use cargo run --release --example stocks.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run cargo run --release --example stocks --no-default-features --features="arrow cpu-perf" instead. It will be slower for proof generation.
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
    base::database::{
        arrow_schema_utility::get_posql_compatible_schema, OwnedTable, OwnedTableTestAccessor,
        TableRef, TestAccessor,
    },
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
// The max_nu should be set such that the maximum table size is less than 2^(2*max_nu-1).
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"f9d2e8c1b7a654309cfe81d2b7a3c940";

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

    let filename = "crates/proof-of-sql-planner/examples/stocks/stocks.csv";
    let schema = get_posql_compatible_schema(&SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ));
    let stocks_batch = ReaderBuilder::new(schema)
        .with_header(true)
        .build(File::open(filename).unwrap())
        .unwrap()
        .next()
        .unwrap()
        .unwrap();

    // Load the table into an "Accessor" so that the prover and verifier can access the data/commitments.
    let mut accessor =
        OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table(
        TableRef::from_names(None, "stocks"),
        OwnedTable::try_from(stocks_batch).unwrap(),
        0,
    );

    // Query 1: Calculate total market cap and count of stocks
    prove_and_verify_query(
        "SELECT SUM(MarketCap) as total_market_cap, COUNT(*) as c FROM stocks",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 2: Find technology stocks with PE ratio under 30 and dividend yield > 0
    prove_and_verify_query(
        "SELECT Symbol, Company, PE_Ratio, DividendYield FROM stocks WHERE Sector = 'Technology' AND PE_Ratio < 30 AND DividendYield > 0",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 3: Total market cap by sector
    prove_and_verify_query(
        "SELECT Sector, SUM(MarketCap) as total_market_cap, COUNT(*) as num_stocks FROM stocks GROUP BY Sector",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    // Query 4: High value stocks with significant volume and dividend yield
    prove_and_verify_query(
        "SELECT Symbol, Company, Price, Volume, DividendYield FROM stocks WHERE Volume > 20000000 AND DividendYield > 0 AND Price > 100",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
