use super::{BNScalar, HyperKZGPublicSetup};
#[cfg(any(not(feature = "blitzar"), test))]
use crate::base::if_rayon;
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    scalar::Scalar,
    slice_ops,
};
use alloc::vec::Vec;
use ark_bn254::{G1Affine, G1Projective};
use ark_ec::AffineRepr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::{AddAssign, Mul, Neg, Sub, SubAssign};
#[cfg(all(feature = "rayon", any(not(feature = "blitzar"), test)))]
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// This is the commitment type used in the hyperkzg proof system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CanonicalSerialize, CanonicalDeserialize, Default)]
pub struct HyperKZGCommitment {
    /// The underlying commitment.
    pub commitment: G1Projective,
}
impl Serialize for HyperKZGCommitment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let affine: G1Affine = self.commitment.into();
        match affine.xy() {
            None => ([0u8; 32], [0u8; 32]).serialize(serializer),
            Some((x, y)) => {
                let mut x_bytes = [0u8; 32];
                CanonicalSerialize::serialize_uncompressed(&x, &mut x_bytes[..])
                    .map_err(serde::ser::Error::custom)?;
                x_bytes.reverse();
                let mut y_bytes = [0u8; 32];
                CanonicalSerialize::serialize_uncompressed(&y, &mut y_bytes[..])
                    .map_err(serde::ser::Error::custom)?;
                y_bytes.reverse();
                (x_bytes, y_bytes).serialize(serializer)
            }
        }
    }
}
impl<'de> Deserialize<'de> for HyperKZGCommitment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (mut x_bytes, mut y_bytes) = <([u8; 32], [u8; 32])>::deserialize(deserializer)?;
        let affine: G1Affine = if (x_bytes, y_bytes) == ([0u8; 32], [0u8; 32]) {
            G1Affine::identity()
        } else {
            x_bytes.reverse();
            y_bytes.reverse();
            let x = CanonicalDeserialize::deserialize_uncompressed(&x_bytes[..])
                .map_err(serde::de::Error::custom)?;
            let y = CanonicalDeserialize::deserialize_uncompressed(&y_bytes[..])
                .map_err(serde::de::Error::custom)?;
            G1Affine::new_unchecked(x, y)
        };
        Ok(Self {
            commitment: affine.into(),
        })
    }
}

impl AddAssign for HyperKZGCommitment {
    fn add_assign(&mut self, rhs: Self) {
        self.commitment = self.commitment + rhs.commitment;
    }
}
impl From<&G1Affine> for HyperKZGCommitment {
    fn from(value: &G1Affine) -> Self {
        Self {
            commitment: (*value).into(),
        }
    }
}

impl Mul<&HyperKZGCommitment> for BNScalar {
    type Output = HyperKZGCommitment;
    fn mul(self, rhs: &HyperKZGCommitment) -> Self::Output {
        Self::Output {
            commitment: rhs.commitment * self.0,
        }
    }
}

impl Mul<HyperKZGCommitment> for BNScalar {
    type Output = HyperKZGCommitment;
    #[expect(clippy::op_ref)]
    fn mul(self, rhs: HyperKZGCommitment) -> Self::Output {
        self * &rhs
    }
}
impl Neg for HyperKZGCommitment {
    type Output = Self;
    fn neg(self) -> Self::Output {
        (-BNScalar::ONE) * self
    }
}
impl SubAssign for HyperKZGCommitment {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}
impl Sub for HyperKZGCommitment {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

#[cfg(any(not(feature = "blitzar"), test))]
#[tracing::instrument(
    name = "compute_commitment_generic_impl (cpu)",
    level = "debug",
    skip_all
)]
fn compute_commitment_generic_impl<T: Into<BNScalar> + Clone + Sync>(
    setup: HyperKZGPublicSetup<'_>,
    offset: usize,
    scalars: &[T],
) -> HyperKZGCommitment {
    assert!(offset + scalars.len() <= setup.len());
    let product: G1Projective = if_rayon!(scalars.par_iter(), scalars.iter())
        .zip(&setup[offset..offset + scalars.len()])
        .map(|(t, s)| *s * Into::<BNScalar>::into(t).0)
        .sum();
    HyperKZGCommitment {
        commitment: G1Projective::from(product),
    }
}

#[cfg(any(not(feature = "blitzar"), test))]
#[tracing::instrument(name = "compute_commitments_impl (cpu)", level = "debug", skip_all)]
fn compute_commitments_impl(
    committable_columns: &[crate::base::commitment::CommittableColumn],
    offset: usize,
    setup: &<HyperKZGCommitment as Commitment>::PublicSetup<'_>,
) -> Vec<HyperKZGCommitment> {
    if_rayon!(committable_columns.par_iter(), committable_columns.iter())
        .map(|column| match column {
            CommittableColumn::Boolean(vals) => {
                compute_commitment_generic_impl(setup, offset, vals)
            }
            CommittableColumn::Uint8(vals) => compute_commitment_generic_impl(setup, offset, vals),
            CommittableColumn::TinyInt(vals) => {
                compute_commitment_generic_impl(setup, offset, vals)
            }
            CommittableColumn::SmallInt(vals) => {
                compute_commitment_generic_impl(setup, offset, vals)
            }
            CommittableColumn::Int(vals) => compute_commitment_generic_impl(setup, offset, vals),
            CommittableColumn::BigInt(vals) | CommittableColumn::TimestampTZ(_, _, vals) => {
                compute_commitment_generic_impl(setup, offset, vals)
            }
            CommittableColumn::Int128(vals) => compute_commitment_generic_impl(setup, offset, vals),
            CommittableColumn::Decimal75(_, _, vals)
            | CommittableColumn::Scalar(vals)
            | CommittableColumn::VarChar(vals)
            | CommittableColumn::VarBinary(vals) => {
                compute_commitment_generic_impl(setup, offset, vals)
            }
        })
        .collect()
}

impl Commitment for HyperKZGCommitment {
    type Scalar = BNScalar;
    type PublicSetup<'a> = HyperKZGPublicSetup<'a>;

    #[cfg(not(feature = "blitzar"))]
    #[tracing::instrument(name = "compute_commitments (cpu)", level = "debug", skip_all)]
    fn compute_commitments(
        committable_columns: &[crate::base::commitment::CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        compute_commitments_impl(committable_columns, offset, setup)
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "compute_commitments (gpu)", level = "debug", skip_all)]
    fn compute_commitments(
        committable_columns: &[crate::base::commitment::CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        if committable_columns.is_empty() {
            return Vec::new();
        }

        // Find the maximum length of the columns to get number of generators to use
        let max_column_len = committable_columns
            .iter()
            .map(CommittableColumn::len)
            .max()
            .expect("You must have at least one column");

        let mut blitzar_commitments = vec![G1Affine::default(); committable_columns.len()];

        blitzar::compute::compute_bn254_g1_uncompressed_commitments_with_generators(
            &mut blitzar_commitments,
            &slice_ops::slice_cast(committable_columns),
            &setup[offset..offset + max_column_len],
        );

        slice_ops::slice_cast(&blitzar_commitments)
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        let mut writer = Vec::with_capacity(self.commitment.compressed_size());
        self.commitment.serialize_compressed(&mut writer).unwrap();
        writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "hyperkzg_proof")]
    use crate::base::database::OwnedColumn;
    use crate::base::math::decimal::Precision;
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use crate::base::{try_standard_binary_deserialization, try_standard_binary_serialization};
    #[cfg(feature = "hyperkzg_proof")]
    use crate::proof_primitive::hyperkzg::nova_commitment_key_to_hyperkzg_public_setup;
    #[cfg(feature = "hyperkzg_proof")]
    use crate::proof_primitive::hyperkzg::HyperKZGEngine;
    use ark_ec::AffineRepr;
    #[cfg(feature = "hyperkzg_proof")]
    use nova_snark::provider::hyperkzg::{CommitmentEngine, CommitmentKey};
    #[cfg(feature = "hyperkzg_proof")]
    use nova_snark::traits::commitment::CommitmentEngineTrait;
    #[cfg(feature = "hyperkzg_proof")]
    use proptest::prelude::*;

    #[test]
    fn we_can_convert_default_point_to_a_hyperkzg_commitment_from_ark_bn254_g1_affine() {
        let commitment: HyperKZGCommitment = HyperKZGCommitment::from(&G1Affine::default());
        assert_eq!(commitment.commitment, G1Affine::default());
    }

    #[test]
    fn we_can_convert_generator_to_a_hyperkzg_commitment_from_ark_bn254_g1_affine() {
        let commitment: HyperKZGCommitment = (&G1Affine::generator()).into();
        let expected: HyperKZGCommitment = HyperKZGCommitment::from(&G1Affine::generator());
        assert_eq!(commitment.commitment, expected.commitment);
    }

    #[test]
    fn we_can_compute_a_hyperkzg_commitment_with_an_offset() {
        let generator = G1Affine::generator();
        let setup = vec![
            generator,
            (generator * BNScalar::from(2_u64).0).into(),
            (generator * BNScalar::from(3_u64).0).into(),
            (generator * BNScalar::from(5_u64).0).into(),
        ];
        let scalars = [7_u8, 11_u8];

        let commitment = compute_commitment_generic_impl(&setup, 1, &scalars);

        let expected = setup[1] * BNScalar::from(7_u8).0 + setup[2] * BNScalar::from(11_u8).0;
        assert_eq!(commitment.commitment, expected);
    }

    #[test]
    fn we_can_dispatch_all_committable_column_types_to_hyperkzg_commitments() {
        let generator = G1Affine::generator();
        let setup = vec![generator, (generator * BNScalar::from(2_u64).0).into()];
        let bools = [true, false];
        let uint8s = [1_u8, 2_u8];
        let tinyints = [-1_i8, 2_i8];
        let smallints = [-3_i16, 4_i16];
        let ints = [-5_i32, 6_i32];
        let bigints = [-7_i64, 8_i64];
        let int128s = [-9_i128, 10_i128];
        let limbs = vec![[11, 0, 0, 0], [12, 0, 0, 0]];
        let timestamps = [13_i64, 14_i64];
        let columns = [
            CommittableColumn::Boolean(&bools),
            CommittableColumn::Uint8(&uint8s),
            CommittableColumn::TinyInt(&tinyints),
            CommittableColumn::SmallInt(&smallints),
            CommittableColumn::Int(&ints),
            CommittableColumn::BigInt(&bigints),
            CommittableColumn::Int128(&int128s),
            CommittableColumn::Decimal75(Precision::new(75).unwrap(), 0, limbs.clone()),
            CommittableColumn::Scalar(limbs.clone()),
            CommittableColumn::VarChar(limbs.clone()),
            CommittableColumn::VarBinary(limbs.clone()),
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &timestamps,
            ),
        ];

        let commitments = compute_commitments_impl(&columns, 0, &&setup[..]);

        assert_eq!(commitments.len(), columns.len());
        assert_eq!(
            commitments[0],
            compute_commitment_generic_impl(&setup, 0, &bools)
        );
        assert_eq!(
            commitments[1],
            compute_commitment_generic_impl(&setup, 0, &uint8s)
        );
        assert_eq!(
            commitments[2],
            compute_commitment_generic_impl(&setup, 0, &tinyints)
        );
        assert_eq!(
            commitments[3],
            compute_commitment_generic_impl(&setup, 0, &smallints)
        );
        assert_eq!(
            commitments[4],
            compute_commitment_generic_impl(&setup, 0, &ints)
        );
        assert_eq!(
            commitments[5],
            compute_commitment_generic_impl(&setup, 0, &bigints)
        );
        assert_eq!(
            commitments[6],
            compute_commitment_generic_impl(&setup, 0, &int128s)
        );
        assert_eq!(
            commitments[7],
            compute_commitment_generic_impl(&setup, 0, &limbs)
        );
        assert_eq!(
            commitments[8],
            compute_commitment_generic_impl(&setup, 0, &limbs)
        );
        assert_eq!(
            commitments[9],
            compute_commitment_generic_impl(&setup, 0, &limbs)
        );
        assert_eq!(
            commitments[10],
            compute_commitment_generic_impl(&setup, 0, &limbs)
        );
        assert_eq!(
            commitments[11],
            compute_commitment_generic_impl(&setup, 0, &timestamps)
        );
    }

    #[cfg(feature = "hyperkzg_proof")]
    proptest! {
        #[test]
        fn blitzar_and_non_blitzar_commitments_are_equal(owned_column: OwnedColumn<BNScalar>) {
            let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", owned_column.len());

            let public_setup = nova_commitment_key_to_hyperkzg_public_setup(&ck);

            let committable_columns = [CommittableColumn::from(&owned_column)];

            let non_blitzar_commitments = compute_commitments_impl(&committable_columns, 0, &&public_setup[..]);
            let blitzar_commitments = HyperKZGCommitment::compute_commitments(&committable_columns, 0, &&public_setup[..]);

            prop_assert_eq!(non_blitzar_commitments, blitzar_commitments);
        }
    }

    #[test]
    fn we_can_serialize_and_deserialize_hyperkzg_commitment_generator() {
        let commitment: HyperKZGCommitment = (&G1Affine::generator()).into();
        let bytes = try_standard_binary_serialization(commitment).unwrap();
        assert_eq!(bytes, [&[0u8; 31][..], &[1], &[0; 31], &[2]].concat());

        let (deserialized_commitment, _): (HyperKZGCommitment, _) =
            try_standard_binary_deserialization(&bytes[..]).unwrap();
        assert_eq!(deserialized_commitment.commitment, G1Affine::generator());
    }
    #[test]
    fn we_can_serialize_and_deserialize_hyperkzg_commitment_identity() {
        let commitment: HyperKZGCommitment = (&G1Affine::identity()).into();
        let bytes = try_standard_binary_serialization(commitment).unwrap();
        assert_eq!(bytes, [&[0u8; 31][..], &[0], &[0; 31], &[0]].concat());

        let (deserialized_commitment, _): (HyperKZGCommitment, _) =
            try_standard_binary_deserialization(&bytes[..]).unwrap();
        assert_eq!(deserialized_commitment.commitment, G1Affine::identity());
    }
    #[test]
    fn we_can_round_trip_serialize_and_deserialize_random_hyperkzg_commitments() {
        use ark_std::UniformRand;

        let mut rng = ark_std::test_rng();

        for _ in 0..100 {
            let commitment: HyperKZGCommitment = (&G1Affine::rand(&mut rng)).into();
            let bytes = try_standard_binary_serialization(commitment).unwrap();
            let (deserialized_commitment, _): (HyperKZGCommitment, _) =
                try_standard_binary_deserialization(&bytes[..]).unwrap();
            assert_eq!(deserialized_commitment.commitment, commitment.commitment);
        }
    }
}
