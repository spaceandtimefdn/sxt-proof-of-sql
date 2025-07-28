use super::bit_mask_utils::{is_bit_mask_negative_representation, make_bit_mask};
use crate::base::{
    proof::ProofError,
    scalar::{Scalar, ScalarExt},
};
use ark_std::iterable::Iterable;
use bit_iter::BitIter;
use bnum::types::U256;
use core::{
    convert::Into,
    ops::{Shl, Shr},
};
use itertools::Itertools;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn serialize_limbs<S: Serializer>(limbs: &[u64; 4], serializer: S) -> Result<S::Ok, S::Error> {
    [limbs[3], limbs[2], limbs[1], limbs[0]].serialize(serializer)
}

fn deserialize_to_limbs<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u64; 4], D::Error> {
    let limbs = <[u64; 4]>::deserialize(deserializer)?;
    Ok([limbs[3], limbs[2], limbs[1], limbs[0]])
}

/// Describe the distribution of bit values in a table column
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitDistribution {
    /// Identifies all columns that are not either identical to or the inverse of the leading column (the sign column). The lead bit indicates if the sign column is constant
    #[serde(
        serialize_with = "serialize_limbs",
        deserialize_with = "deserialize_to_limbs"
    )]
    pub(crate) vary_mask: [u64; 4],
    /// Identifies all columns that are the identical to the lead column. The lead bit indicates the sign of the last row of data (only relevant if the sign is constant)
    #[serde(
        serialize_with = "serialize_limbs",
        deserialize_with = "deserialize_to_limbs"
    )]
    pub(crate) leading_bit_mask: [u64; 4],
}

/// Errors associated with `BitDistribution`
#[derive(Debug)]
pub enum BitDistributionError {
    /// No lead bit was provided when the lead bit is variable
    NoLeadBit,
    /// Failed to verify bit decomposition
    Verification,
}

impl From<BitDistributionError> for ProofError {
    fn from(err: BitDistributionError) -> Self {
        match err {
            BitDistributionError::NoLeadBit => {
                panic!("No lead bit available despite variable lead bit.")
            }
            BitDistributionError::Verification => ProofError::VerificationError {
                error: "invalid bit_decomposition",
            },
        }
    }
}

impl BitDistribution {
    pub fn new<S: Scalar, T: Into<S> + Clone + Sync>(data: &[T]) -> Self {
        #[cfg(feature = "rayon")]
        let (sign_mask, inverse_sign_mask) = data
            .par_iter()
            .map(|item| {
                let scalar: S = item.clone().into();
                let bit_mask = make_bit_mask(scalar);

                let adjusted_mask = if is_bit_mask_negative_representation(bit_mask) {
                    bit_mask ^ U256::MAX.shr(1)
                } else {
                    bit_mask
                };

                (adjusted_mask, !adjusted_mask)
            })
            .reduce(
                || (U256::MAX, U256::MAX),
                |(acc_sign, acc_inverse), (mask, inverse_mask)| {
                    (acc_sign & mask, acc_inverse & inverse_mask)
                },
            );

        let bit_masks = data.iter().cloned().map(Into::<S>::into).map(make_bit_mask);
        #[cfg(not(feature = "rayon"))]
        let (sign_mask, inverse_sign_mask) = bit_masks
            .clone()
            .map(|bit_mask| {
                let adjusted_mask = if is_bit_mask_negative_representation(bit_mask) {
                    bit_mask ^ U256::MAX.shr(1)
                } else {
                    bit_mask
                };
                (adjusted_mask, !adjusted_mask)
            })
            .reduce(|acc, item| (acc.0 & item.0, acc.1 & item.1))
            .unwrap_or((U256::MAX, U256::MAX));

        let vary_mask_bit = U256::from(
            !bit_masks
                .map(is_bit_mask_negative_representation)
                .all_equal(),
        ) << 255;
        let vary_mask: U256 = !(sign_mask | inverse_sign_mask) | vary_mask_bit;

        Self {
            leading_bit_mask: sign_mask.into(),
            vary_mask: vary_mask.into(),
        }
    }

    pub fn vary_mask(&self) -> U256 {
        U256::from(self.vary_mask)
    }

    /// Identifies all columns that are the identical to the lead column.
    pub fn leading_bit_mask(&self) -> U256 {
        U256::from(self.leading_bit_mask) | (U256::ONE.shl(255))
    }

    /// Identifies all columns that are the identical to the inverse of the lead column.
    pub fn leading_bit_inverse_mask(&self) -> U256 {
        (!self.vary_mask() ^ self.leading_bit_mask()) & U256::MAX.shr(1)
    }

    /// # Panics
    ///
    /// Panics if conversion from `ExpType` to `usize` fails
    pub fn num_varying_bits(&self) -> usize {
        self.vary_mask().count_ones() as usize
    }

    /// Determines the lead (sign) bit.
    pub fn try_constant_leading_bit_eval<S: ScalarExt>(&self, chi_eval: S) -> Option<S> {
        if U256::from(self.vary_mask) & (U256::ONE.shl(255)) != U256::ZERO {
            None
        } else if U256::from(self.leading_bit_mask) & U256::ONE.shl(255) == U256::ZERO {
            Some(S::ZERO)
        } else {
            Some(chi_eval)
        }
    }

    /// Check if this instance represents a valid bit distribution. `is_valid`
    /// can be used after deserializing a [`BitDistribution`] from an untrusted
    /// source.
    pub fn is_valid(&self) -> bool {
        (self.vary_mask() & self.leading_bit_mask()) & U256::MAX.shr(1) == U256::ZERO
    }

    /// In order to avoid cases with large numbers where there can be both a positive and negative
    /// representation, we restrict the range of bit distributions that we accept.
    ///
    /// Currently this is set to be the minimal value that will include the sum of two signed 128-bit
    /// integers. The range will likely be expanded in the future as we support additional expressions.
    pub fn is_within_acceptable_range(&self) -> bool {
        // signed 128 bit numbers range from
        //      -2^127 to 2^127-1
        // the maximum absolute value of the sum of two signed 128-integers is
        // then
        //       2 * (2^127) = 2^128
        (self.leading_bit_inverse_mask() >> 128) == (U256::MAX.shr(129))
    }

    /// Iterate over each varying bit
    #[expect(clippy::missing_panics_doc)]
    pub fn vary_mask_iter(&self) -> impl Iterator<Item = u8> + '_ {
        (0..4).flat_map(|i| {
            BitIter::from(self.vary_mask[i])
                .iter()
                .map(move |pos| u8::try_from(i * 64 + pos).expect("index greater than 255"))
        })
    }
}
