use super::EmptyExec;
use crate::{
    base::{
        database::{OwnedTableTestAccessor, TableRef},
        scalar::Scalar,
    },
    sql::proof::{ProofPlan, VerifiableQueryResult},
};
#[cfg(feature = "blitzar")]
use blitzar::proof::InnerProductProof;

#[test]
fn empty_exec_get_column_result_fields_is_empty() {
    let exec = EmptyExec::new();
    assert!(exec.get_column_result_fields().is_empty());
}

#[test]
fn empty_exec_get_column_references_is_empty() {
    let exec = EmptyExec::new();
    assert!(exec.get_column_references().is_empty());
}

#[test]
fn empty_exec_get_table_references_is_empty() {
    let exec = EmptyExec::new();
    assert!(exec.get_table_references().is_empty());
}

#[test]
#[cfg(feature = "blitzar")]
fn empty_exec_prover_evaluate_returns_empty_table() {
    use crate::base::map::IndexMap;
    use crate::sql::proof::ProverEvaluate;
    use bumpalo::Bump;

    let exec = EmptyExec::new();
    let alloc = Bump::new();
    let table_map = IndexMap::default();

    // Test first round evaluate
    let res1 = exec
        .first_round_evaluate::<curve25519_dalek::scalar::Scalar>(
            &mut crate::sql::proof::FirstRoundBuilder::new(&alloc),
            &alloc,
            &table_map,
            &[],
        )
        .unwrap();
    assert_eq!(res1.num_rows(), 1);
    assert_eq!(res1.num_columns(), 0);

    // Test final round evaluate
    let res2 = exec
        .final_round_evaluate::<curve25519_dalek::scalar::Scalar>(
            &mut crate::sql::proof::FinalRoundBuilder::new(&alloc),
            &alloc,
            &table_map,
            &[],
        )
        .unwrap();
    assert_eq!(res2.num_rows(), 1);
    assert_eq!(res2.num_columns(), 0);
}

#[test]
#[cfg(feature = "blitzar")]
fn we_can_verify_empty_exec_plan() {
    let table_ref = "namespace.table_name".parse::<TableRef>().unwrap();
    let exec = EmptyExec::new();

    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref,
        crate::base::database::owned_table_utility::owned_table([]),
        0_usize,
        (),
    );

    let verifiable_res =
        VerifiableQueryResult::<InnerProductProof>::new(&exec, &accessor, &(), &[]).unwrap();

    let res = verifiable_res
        .verify(&exec, &accessor, &(), &[])
        .expect("verification should succeed");

    assert_eq!(res.table.num_rows(), 1);
    assert_eq!(res.table.num_columns(), 0);
}
