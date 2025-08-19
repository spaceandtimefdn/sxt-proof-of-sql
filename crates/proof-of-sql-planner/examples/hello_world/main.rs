#![doc = include_str!("README.md")]
use datafusion::{
    arrow::{record_batch::RecordBatch, util::pretty::pretty_format_batches},
    config::ConfigOptions,
};
use proof_of_sql::{
    base::database::{
        owned_table_utility::{bigint, owned_table, varchar},
        OwnedTableTestAccessor, TableRef, TestAccessor,
    },
    proof_primitive::dory::{
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::proof::VerifiableQueryResult,
};
use proof_of_sql_planner::sql_to_proof_plans;
use rand::{rngs::StdRng, SeedableRng};
use sqlparser::{dialect::GenericDialect, parser::Parser};
use std::{
    io::{stdout, Write},
    time::Instant,
};
/// # Panics
///
/// Will panic if flushing the output fails, which can happen due to issues with the underlying output stream.
fn start_timer(message: &str) -> Instant {
    print!("{message}...");
    stdout().flush().unwrap();
    Instant::now()
}
/// # Panics
///
/// This function does not panic under normal circumstances but may panic if the internal printing fails due to issues with the output stream.
fn end_timer(instant: Instant) {
    println!(" {:?}", instant.elapsed());
}

/// # Panics
///
/// - Will panic if the GPU initialization fails during `init_backend`.
/// - Will panic if the table reference cannot be parsed in `add_table`.
/// - Will panic if the offset provided to `add_table` is invalid.
/// - Will panic if the query string cannot be parsed or if the proof fails to verify.
fn main() {
    let timer = start_timer("Loading data");
    // Use a fixed seed for deterministic results
    let mut rng = StdRng::from_seed([0u8; 32]);
    let public_parameters = PublicParameters::rand(5, &mut rng);
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let mut accessor =
        OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(&prover_setup);
    accessor.add_table(
        TableRef::from_names(None, "tab"),
        owned_table([
            bigint("a", [1, 2, 3, 2]),
            varchar("b", ["hi", "hello", "there", "world"]),
        ]),
        0,
    );
    end_timer(timer);

    let sql = "SELECT b FROM tab WHERE a = 2";

    let timer = start_timer("Parsing Query");
    let config = ConfigOptions::default();
    let statements = Parser::parse_sql(&GenericDialect {}, sql).unwrap();
    let query_plan = &sql_to_proof_plans(&statements, &accessor, &config).unwrap()[0];
    end_timer(timer);

    let timer = start_timer("Generating Proof");
    let verifiable_result = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
        query_plan,
        &accessor,
        &&prover_setup,
        &[],
    )
    .unwrap();
    end_timer(timer);

    let timer = start_timer("Verifying Proof");
    let result: RecordBatch = verifiable_result
        .verify(query_plan, &accessor, &&verifier_setup, &[])
        .unwrap()
        .table
        .try_into()
        .unwrap();
    end_timer(timer);

    println!("Valid proof!");
    println!("Query Result:");
    println!("{}", pretty_format_batches(&[result]).unwrap());
}
