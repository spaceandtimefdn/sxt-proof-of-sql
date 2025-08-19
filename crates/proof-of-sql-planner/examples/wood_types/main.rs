//! This is a non-interactive example of using Proof of SQL with a wood types dataset.
//! To run this, use `cargo run --release --example wood_types`.
//!
//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example wood_types --no-default-features --features="cpu-perf"` instead. It will be slower for proof generation.
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
// The `max_nu` should be set such that the maximum table size is less than `2^(2*max_nu-1)`.
const DORY_SETUP_MAX_NU: usize = 8;
// This should be a "nothing-up-my-sleeve" phrase or number.
const DORY_SEED: [u8; 32] = *b"f3a8d12e6b7c4590a1f2e3d4b5c6a7b8";

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

    let filename = "crates/proof-of-sql-planner/examples/wood_types/wood_types.csv";
    let inferred_schema =
        SchemaRef::new(infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap());
    let posql_compatible_schema = get_posql_compatible_schema(&inferred_schema);

    let wood_types_batch = ReaderBuilder::new(posql_compatible_schema)
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
        TableRef::from_names(None, "woods"),
        OwnedTable::try_from(wood_types_batch).unwrap(),
        0,
    );

    prove_and_verify_query(
        "SELECT COUNT(*) AS total_wood_types FROM woods",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT common_use, SUM(density) as total_density, COUNT(*) as wood_count FROM woods GROUP BY common_use",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT common_use, COUNT(*) as c FROM woods GROUP BY common_use",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );

    prove_and_verify_query(
        "SELECT name, hardness FROM woods WHERE density > 0.7",
        &accessor,
        &prover_setup,
        &verifier_setup,
    );
}
