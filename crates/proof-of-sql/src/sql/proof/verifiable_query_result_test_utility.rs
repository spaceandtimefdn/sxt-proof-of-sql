use super::{ProofPlan, VerifiableQueryResult};
use crate::{
    base::{
        commitment::{Commitment, CommittableColumn},
        database::{owned_table_utility::*, OwnedColumn, OwnedTable, TableRef, TestAccessor},
        scalar::Scalar,
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
};
use blitzar::proof::InnerProductProof;
use curve25519_dalek::{ristretto::RistrettoPoint, traits::Identity};
use num_traits::One;
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
pub fn exercise_verification(
    res: &VerifiableQueryResult<InnerProductProof>,
    expr: &(impl ProofPlan + Serialize),
    accessor: &impl TestAccessor<RistrettoPoint>,
    table_ref: &TableRef,
) {
    res.clone()
        .verify(expr, accessor, &(), &[])
        .expect("Verification failed");

    // try changing the result
    let mut res_p = res.clone();
    res_p.result = tampered_table(&res.result);
    assert!(res_p.verify(expr, accessor, &(), &[]).is_err());

    // try changing MLE evaluations
    for i in 0..res.proof.pcs_proof_evaluations.final_round.len() {
        let mut res_p = res.clone();
        res_p.proof.pcs_proof_evaluations.final_round[i] += Curve25519Scalar::one();
        assert!(res_p.verify(expr, accessor, &(), &[]).is_err());
    }

    // try changing intermediate commitments
    let commit_p = RistrettoPoint::compute_commitments(
        &[CommittableColumn::BigInt(&[
            353_453_245_i64,
            93_402_346_i64,
        ])],
        0_usize,
        &(),
    )[0];

    for i in 0..res.proof.final_round_message.round_commitments.len() {
        let mut res_p = res.clone();
        res_p.proof.final_round_message.round_commitments[i] = commit_p;
        assert!(res_p.verify(expr, accessor, &(), &[]).is_err());
    }

    // try changing the offset
    //
    // Note: in the n = 1 case with proof.commmitments all the identity element,
    // the inner product proof isn't dependent on the generators since it simply sends the input
    // vector; hence, changing the offset would have no effect.
    if accessor.get_length(table_ref) > 1
        || res
            .proof
            .final_round_message
            .round_commitments
            .iter()
            .any(|&c| c != Identity::identity())
    {
        let offset_generators = accessor.get_offset(table_ref);
        let mut fake_accessor = accessor.clone();
        fake_accessor.update_offset(table_ref, offset_generators);
        res.clone().verify(expr, &fake_accessor, &(), &[]).unwrap();
        fake_accessor.update_offset(table_ref, offset_generators + 1);
        assert!(res.clone().verify(expr, &fake_accessor, &(), &[]).is_err());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn tampered_table_adds_empty_column_for_zero_column_result() {
        let table = owned_table::<TestScalar>([]);
        let tampered = tampered_table(&table);

        assert_eq!(tampered.num_columns(), 1);
        assert_eq!(tampered.num_rows(), 0);
        assert_eq!(tampered["col"], OwnedColumn::BigInt(vec![]));
    }

    #[test]
    fn tampered_table_appends_default_row_to_empty_columns() {
        let table = owned_table::<TestScalar>([
            boolean("boolean", Vec::<bool>::new()),
            uint8("uint8", Vec::<u8>::new()),
            tinyint("tinyint", Vec::<i8>::new()),
            smallint("smallint", Vec::<i16>::new()),
            int("int", Vec::<i32>::new()),
            bigint("bigint", Vec::<i64>::new()),
            int128("int128", Vec::<i128>::new()),
            scalar("scalar", Vec::<TestScalar>::new()),
            varchar("varchar", Vec::<String>::new()),
            varbinary("varbinary", Vec::<Vec<u8>>::new()),
            decimal75("decimal", 12, 2, Vec::<TestScalar>::new()),
            timestamptz(
                "timestamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                Vec::<i64>::new(),
            ),
        ]);

        let tampered = tampered_table(&table);

        assert_eq!(tampered.num_rows(), 1);
        assert_eq!(tampered["boolean"], OwnedColumn::Boolean(vec![false]));
        assert_eq!(tampered["uint8"], OwnedColumn::Uint8(vec![0]));
        assert_eq!(tampered["tinyint"], OwnedColumn::TinyInt(vec![0]));
        assert_eq!(tampered["smallint"], OwnedColumn::SmallInt(vec![0]));
        assert_eq!(tampered["int"], OwnedColumn::Int(vec![0]));
        assert_eq!(tampered["bigint"], OwnedColumn::BigInt(vec![0]));
        assert_eq!(tampered["int128"], OwnedColumn::Int128(vec![0]));
        assert_eq!(
            tampered["scalar"],
            OwnedColumn::Scalar(vec![TestScalar::ZERO])
        );
        assert_eq!(
            tampered["varchar"],
            OwnedColumn::VarChar(vec![String::new()])
        );
        assert_eq!(tampered["varbinary"], OwnedColumn::VarBinary(vec![vec![0]]));
        assert_eq!(
            tampered["decimal"],
            OwnedColumn::Decimal75(
                crate::base::math::decimal::Precision::new(12).unwrap(),
                2,
                vec![TestScalar::ZERO],
            )
        );
        assert_eq!(
            tampered["timestamp"],
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![0])
        );
    }

    #[test]
    fn tamper_first_row_of_column_mutates_each_column_variant() {
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::Boolean(vec![false])),
            OwnedColumn::Boolean(vec![true])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::Uint8(vec![u8::MAX])),
            OwnedColumn::Uint8(vec![0])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::TinyInt(vec![i8::MAX])),
            OwnedColumn::TinyInt(vec![i8::MIN])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::SmallInt(vec![i16::MAX])),
            OwnedColumn::SmallInt(vec![i16::MIN])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::Int(vec![i32::MAX])),
            OwnedColumn::Int(vec![i32::MIN])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::BigInt(vec![i64::MAX])),
            OwnedColumn::BigInt(vec![i64::MIN])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::Int128(vec![i128::MAX])),
            OwnedColumn::Int128(vec![i128::MIN])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::VarChar(vec![
                "value".to_string()
            ])),
            OwnedColumn::VarChar(vec!["value1".to_string()])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::VarBinary(vec![vec![2, 3]])),
            OwnedColumn::VarBinary(vec![vec![2, 3, 1]])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::Decimal75(
                crate::base::math::decimal::Precision::new(12).unwrap(),
                2,
                vec![TestScalar::ZERO],
            )),
            OwnedColumn::Decimal75(
                crate::base::math::decimal::Precision::new(12).unwrap(),
                2,
                vec![TestScalar::ONE],
            )
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![i64::MAX],
            )),
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![i64::MIN])
        );
        assert_eq!(
            tamper_first_row_of_column(&OwnedColumn::<TestScalar>::Scalar(vec![TestScalar::ZERO])),
            OwnedColumn::Scalar(vec![TestScalar::ONE])
        );
    }
}
