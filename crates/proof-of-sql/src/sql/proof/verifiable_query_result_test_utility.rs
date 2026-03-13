use super::{ProofPlan, VerifiableQueryResult};
use crate::base::{
    commitment::CommitmentEvaluationProof,
    database::{owned_table_utility::*, OwnedColumn, OwnedTable, TableRef, TestAccessor},
    scalar::Scalar,
};
use serde::Serialize;

/// This function takes a valid `verifiable_result`, copies it, tweaks it, and checks that
/// verification fails.
///
/// It's useful as a tool for testing proof code.
///
/// # Panics
///
/// Will panic if:
/// - The verification of `res` does not succeed, causing the assertion `assert!(res.verify(...).is_ok())` to fail.
/// - `fake_accessor.update_offset` fails, causing a panic if it is designed to do so in the implementation.
pub fn exercise_verification<CP>(
    res: &VerifiableQueryResult<CP>,
    expr: &(impl ProofPlan + Serialize),
    accessor: &impl TestAccessor<CP::Commitment>,
    table_ref: &TableRef,
) where
    CP: CommitmentEvaluationProof + Clone,
    for<'a> CP::VerifierPublicSetup<'a>: Default,
{
    let verifier_setup = CP::VerifierPublicSetup::default();
    res.clone()
        .verify(expr, accessor, &verifier_setup, &[])
        .expect("Verification failed");

    // try changing the result
    let mut res_p = res.clone();
    res_p.result = tampered_table(&res.result);
    let verifier_setup = CP::VerifierPublicSetup::default();
    assert!(res_p.verify(expr, accessor, &verifier_setup, &[]).is_err());

    // try changing MLE evaluations
    for i in 0..res.proof.pcs_proof_evaluations.final_round.len() {
        let mut res_p = res.clone();
        res_p.proof.pcs_proof_evaluations.final_round[i] += CP::Scalar::ONE;
        let verifier_setup = CP::VerifierPublicSetup::default();
        assert!(res_p.verify(expr, accessor, &verifier_setup, &[]).is_err());
    }

    // try changing the offset
    //
    // Note: in the n = 1 case with proof.commmitments all the identity element,
    // the inner product proof isn't dependent on the generators since it simply sends the input
    // vector; hence, changing the offset would have no effect.
    if accessor.get_length(table_ref) > 1 {
        let offset_generators = accessor.get_offset(table_ref);
        let mut fake_accessor = accessor.clone();
        fake_accessor.update_offset(table_ref, offset_generators);
        let verifier_setup = CP::VerifierPublicSetup::default();
        res.clone()
            .verify(expr, &fake_accessor, &verifier_setup, &[])
            .unwrap();
        fake_accessor.update_offset(table_ref, offset_generators + 1);
        let verifier_setup = CP::VerifierPublicSetup::default();
        assert!(res
            .clone()
            .verify(expr, &fake_accessor, &verifier_setup, &[])
            .is_err());
    }
}

fn tampered_table<S: Scalar>(table: &OwnedTable<S>) -> OwnedTable<S> {
    if table.num_columns() == 0 {
        owned_table([bigint("col", [0; 0])])
    } else if table.num_rows() == 0 {
        append_single_row_to_table(table)
    } else {
        tamper_first_element_of_table(table)
    }
}
fn append_single_row_to_table<S: Scalar>(table: &OwnedTable<S>) -> OwnedTable<S> {
    OwnedTable::try_from_iter(
        table
            .inner_table()
            .iter()
            .map(|(name, col)| (name.clone(), append_single_row_to_column(col))),
    )
    .expect("Failed to create table")
}
fn append_single_row_to_column<S: Scalar>(column: &OwnedColumn<S>) -> OwnedColumn<S> {
    let mut column = column.clone();
    match &mut column {
        OwnedColumn::Boolean(col) => col.push(false),
        OwnedColumn::Uint8(col) => col.push(0),
        OwnedColumn::TinyInt(col) => col.push(0),
        OwnedColumn::SmallInt(col) => col.push(0),
        OwnedColumn::Int(col) => col.push(0),
        OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.push(0),
        OwnedColumn::VarChar(col) => col.push(String::new()),
        OwnedColumn::VarBinary(col) => col.push(vec![0u8]),
        OwnedColumn::Int128(col) => col.push(0),
        OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.push(S::ZERO),
    }
    column
}
fn tamper_first_element_of_table<S: Scalar>(table: &OwnedTable<S>) -> OwnedTable<S> {
    OwnedTable::try_from_iter(
        table
            .inner_table()
            .iter()
            .enumerate()
            .map(|(i, (name, col))| {
                (
                    name.clone(),
                    if i == 0 {
                        tamper_first_row_of_column(col)
                    } else {
                        col.clone()
                    },
                )
            }),
    )
    .expect("Failed to create table")
}
pub fn tamper_first_row_of_column<S: Scalar>(column: &OwnedColumn<S>) -> OwnedColumn<S> {
    let mut column = column.clone();
    match &mut column {
        OwnedColumn::Boolean(col) => col[0] ^= true,
        OwnedColumn::Uint8(col) => col[0] = col[0].wrapping_add(1),
        OwnedColumn::TinyInt(col) => col[0] = col[0].wrapping_add(1),
        OwnedColumn::SmallInt(col) => col[0] = col[0].wrapping_add(1),
        OwnedColumn::Int(col) => col[0] = col[0].wrapping_add(1),
        OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
            col[0] = col[0].wrapping_add(1);
        }
        OwnedColumn::VarChar(col) => col[0].push('1'),
        OwnedColumn::VarBinary(col) => col[0].push(1u8),
        OwnedColumn::Int128(col) => col[0] = col[0].wrapping_add(1),
        OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col[0] += S::ONE,
    }
    column
}
