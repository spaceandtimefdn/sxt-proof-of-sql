//! Module containing the `DoryCommitment` type and its implementation.
//!
//! While this can be used as a black box, it can be helpful to understand the underlying structure of the commitment.
//! Ultimately, the commitment is a commitment to a Matrix. This matrix is filled out from a column in the following fashion.
//!
//! We let `sigma` be a parameter that specifies the number of non-zero columns in the matrix.
//! More specifically, the number of non-zero columns is `2^sigma`.
//!
//! For an example, we will set `sigma=2` and thus, the number of columns is 4.
//! The column `[100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115]` with offset 9 is converted to the following matrix:
//! ```ignore
//!  0   0   0   0
//!  0   0   0   0
//!  0  100 101 102
//! 103 104 105 106
//! 107 108 109 110
//! 111 112 113 114
//! 115  0   0   0
//! ```
//! This matrix is then committed to using a matrix commitment.
//!
//! Note: the `VecCommitmentExt` trait requires using this offset when computing commitments.
//! This is to allow for updateability of the commitments as well as to allow for smart indexing/partitioning.

use super::{DoryScalar, ProverSetup, GT};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    impl_serde_for_ark_serde_checked,
};
use alloc::vec::Vec;
use ark_ec::pairing::PairingOutput;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::Mul;
use derive_more::{AddAssign, Neg, Sub, SubAssign};
use num_traits::One;

#[derive(
    Debug,
    Sub,
    Eq,
    PartialEq,
    Neg,
    Copy,
    Clone,
    AddAssign,
    SubAssign,
    CanonicalSerialize,
    CanonicalDeserialize,
)]
/// The Dory commitment type.
pub struct DynamicDoryCommitment(pub(super) GT);

/// The default for GT is the the additive identity, but should be the multiplicative identity.
impl Default for DynamicDoryCommitment {
    fn default() -> Self {
        Self(PairingOutput(One::one()))
    }
}

// Traits required for `DoryCommitment` to impl `Commitment`.
impl_serde_for_ark_serde_checked!(DynamicDoryCommitment);
impl Mul<DynamicDoryCommitment> for DoryScalar {
    type Output = DynamicDoryCommitment;
    fn mul(self, rhs: DynamicDoryCommitment) -> Self::Output {
        DynamicDoryCommitment(rhs.0 * self.0)
    }
}
impl<'a> Mul<&'a DynamicDoryCommitment> for DoryScalar {
    type Output = DynamicDoryCommitment;
    fn mul(self, rhs: &'a DynamicDoryCommitment) -> Self::Output {
        DynamicDoryCommitment(rhs.0 * self.0)
    }
}
impl Commitment for DynamicDoryCommitment {
    type Scalar = DoryScalar;
    type PublicSetup<'a> = &'a ProverSetup<'a>;

    fn compute_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        super::compute_dynamic_dory_commitments(committable_columns, offset, setup)
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.0.compressed_size());
        self.0.serialize_compressed(&mut buf).unwrap();
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::{DynamicDoryCommitment, GT};
    use crate::{
        base::{
            commitment::{ColumnCommitments, Commitment, TableCommitment},
            database::{
                owned_table_utility::{
                    bigint, boolean, decimal75, int, int128, owned_table, scalar, smallint,
                    timestamptz, tinyint, uint8, varbinary, varchar,
                },
                OwnedTable,
            },
            posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
            try_standard_binary_deserialization, try_standard_binary_serialization,
        },
        proof_primitive::dory::{test_rng, DoryScalar, ProverSetup, PublicParameters},
    };
    use ark_ff::UniformRand;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn we_get_different_transcript_bytes_from_different_dynamic_dory_commitments() {
        let mut rng = StdRng::seed_from_u64(42);
        let commitment1 = DynamicDoryCommitment(GT::rand(&mut rng));
        let commitment2 = DynamicDoryCommitment(GT::rand(&mut rng));
        assert_ne!(
            commitment1.to_transcript_bytes(),
            commitment2.to_transcript_bytes()
        );
    }

    /// Under no circumstances should this test be modified. Unless you know what you're doing.
    /// This tests is solely meant to confirm that the serialization of a table commitment has not changed.
    #[test]
    fn commitment_serialization_does_not_change() {
        let expected_serialization =
            include_bytes!("./test_table_commitmet_do_not_modify.bin").to_vec();
        let public_parameters = PublicParameters::test_rand(5, &mut test_rng());
        let setup = ProverSetup::from(&public_parameters);

        let base_table: OwnedTable<DoryScalar> = owned_table([
            uint8("uint8_column", [1, 2, 3, 4]),
            tinyint("tinyint_column", [1, -2, 3, 4]),
            smallint("smallint_column", [1i16, 2, -3, 4]),
            int("int_column", [1, 2, 3, -14]),
            bigint("bigint_column", [1, 2, -333, 4]),
            int128("int128_column", [1, 2, 3, i128::MIN]),
            boolean("bool_column", [true, true, true, false]),
            decimal75("decimal_column", 3, 1, [1, 300, -1, 2]),
            varchar("varchar_column", ["Lorem", "ipsum", "dolor", "sit"]),
            scalar("scalar_column", [1, 3, -1, 2]),
            timestamptz(
                "timestamp_column",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [-18, -17, 17, 18],
            ),
            varbinary(
                "varbinary_column",
                [
                    [1, 2, 3, 0].as_slice(),
                    &[4, 5, 6, 7],
                    &[4, 5, u8::MAX, 7],
                    &[4, 0, 6, 7],
                ],
            ),
        ]);
        let base_commitments =
            ColumnCommitments::<DynamicDoryCommitment>::try_from_columns_with_offset(
                base_table.inner_table(),
                0,
                &&setup,
            )
            .unwrap();
        let table_commitment = TableCommitment::try_new(base_commitments, 0..4).unwrap();
        let serialized_table_commitment =
            try_standard_binary_serialization(table_commitment.clone()).unwrap();
        assert_eq!(serialized_table_commitment, expected_serialization);
        let (deserialized_table_commitment, _) = try_standard_binary_deserialization::<
            TableCommitment<DynamicDoryCommitment>,
        >(&serialized_table_commitment)
        .unwrap();
        assert_eq!(deserialized_table_commitment, table_commitment);
    }
}
