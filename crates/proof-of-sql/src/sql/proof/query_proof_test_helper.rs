/// Helper assertions for `QueryProof` integration tests.
///
/// This module provides shared utilities used across `QueryProof`-level tests,
/// such as verifying that a proof roundtrips correctly for a given expression.
use crate::{
    base::{
        commitment::naive_commitment::NaiveCommitment,
        database::{OwnedTable, OwnedTableTestAccessor, TableRef},
        scalar::test_scalar::TestScalar,
    },
    sql::proof::{ProofPlan, QueryProof, VerificationError},
};

/// Verify that constructing and verifying a `QueryProof` for `expr` over `table`
/// returns `Ok(result_table)` equal to `expected`.
///
/// Panics on any mismatch or on a verification error.
pub fn assert_query_proof_roundtrip<E: ProofPlan<NaiveCommitment>>(
    expr: &E,
    table_ref: TableRef,
    table: OwnedTable<TestScalar>,
    expected: &OwnedTable<TestScalar>,
) {
    let accessor =
        OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, table, 0, ());
    let (proof, result) = QueryProof::<NaiveCommitment>::new(expr, &accessor, &());
    let verified = proof
        .verify(expr, &accessor, &result, &())
        .expect("proof verification failed");
    assert_eq!(
        &verified.into_owned_table::<TestScalar>().unwrap(),
        expected,
        "verified result does not match expected table"
    );
}

/// Verify that proof verification fails with a [`VerificationError`] for a
/// deliberately tampered result table.
pub fn assert_query_proof_verification_fails<E: ProofPlan<NaiveCommitment>>(
    expr: &E,
    table_ref: TableRef,
    table: OwnedTable<TestScalar>,
) {
    let accessor =
        OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, table, 0, ());
    let (proof, mut result) = QueryProof::<NaiveCommitment>::new(expr, &accessor, &());
    // Corrupt the serialised result so verification must fail
    result.flip_bit_for_testing();
    let outcome = proof.verify(expr, &accessor, &result, &());
    assert!(
        matches!(outcome, Err(VerificationError::VerificationError { .. })),
        "expected VerificationError but got: {outcome:?}"
    );
}

#[cfg(test)]
mod tests {
    // Self-compilation smoke test: ensure helper types resolve correctly.
    #[allow(unused_imports)]
    use super::*;
}
