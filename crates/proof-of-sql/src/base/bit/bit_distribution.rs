use super::bit_mask_utils::{is_bit_mask_negative_representation, make_bit_mask};
use crate::base::scalar::{Scalar, ScalarExt};
use ark_std::iterable::Iterable;
use bit_iter::BitIter;
use bnum::types::U256;
use core::{
    convert::Into,
    ops::{Shl, Shr},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Describe the distribution of bit values in a table column
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitDistribution {
    /// Identifies all columns that are not either identical to or the inverse of the leading column (the sign column). The lead bit indicates if the sign column is constant
    pub(crate) vary_mask: [u64; 4],
    /// Identifies all columns that are the identical to the lead column. The lead bit indicates the sign of the last row of data (only relevant if the sign is constant)
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

impl BitDistribution {
    /// Creates a new `BitDistribution` by analyzing the bit patterns in the provided data.
    ///
    /// This function examines each element in the data slice, converts them to bit masks,
    /// and determines which bit positions vary across the dataset and which are constant.
    pub fn new<S: Scalar, T: Into<S> + Clone>(data: &[T]) -> Self {
        // Convert each data element to a scalar and then to its bit mask representation
        let bit_masks = data.iter().cloned().map(Into::<S>::into).map(make_bit_mask);
        // Calculate masks for bits that are consistently 1 (sign_mask) or consistently 0 (inverse_sign_mask)
        // across all elements, normalizing negative representations first
        let (sign_mask, inverse_sign_mask) =
            bit_masks
                .clone()
                .fold((U256::MAX, U256::MAX), |acc, bit_mask| {
                    // Normalize negative representations by flipping the sign bit
                    let bit_mask = if is_bit_mask_negative_representation(bit_mask) {
                        bit_mask ^ U256::MAX.shr(1)
                    } else {
                        bit_mask
                    };
                    // Accumulate bits that are consistently 1 and consistently 0
                    (acc.0 & bit_mask, acc.1 & !bit_mask)
                });
        // Set the most significant bit if the sign varies across elements
        let vary_mask_bit = U256::from(
            !bit_masks
                .map(is_bit_mask_negative_representation)
                .all_equal(),
        ) << 255;
        // Combine varying bit positions: bits that are neither consistently 1 nor consistently 0,
        // plus the sign bit if it varies
        let vary_mask: U256 = !(sign_mask | inverse_sign_mask) | vary_mask_bit;

        Self {
            leading_bit_mask: sign_mask.into(),
            vary_mask: vary_mask.into(),
        }
    }

    /// Returns the mask identifying bit positions that vary across the dataset.
    /// A bit is set to 1 if that position has different values across different elements.
    pub fn vary_mask(&self) -> U256 {
        U256::from(self.vary_mask)
    }

    /// Returns the mask identifying bit positions that are identical to the lead (sign) column.
    /// The most significant bit is always set to indicate the sign column itself.
    pub fn leading_bit_mask(&self) -> U256 {
        U256::from(self.leading_bit_mask) | (U256::ONE.shl(255))
    }

    /// Returns the mask identifying bit positions that are identical to the inverse of the lead column.
    /// These are positions where the bit value is always opposite to the sign bit.
    pub fn leading_bit_inverse_mask(&self) -> U256 {
        (!self.vary_mask() ^ self.leading_bit_mask()) & U256::MAX.shr(1)
    }

    /// Returns the number of bit positions that vary across the dataset.
    /// This is calculated by counting the number of 1s in the `vary_mask`.
    ///
    /// # Panics
    ///
    /// Panics if conversion from `ExpType` to `usize` fails
    pub fn num_varying_bits(&self) -> usize {
        self.vary_mask().count_ones() as usize
    }

    /// Determines the evaluation of the lead (sign) bit based on the bit distribution.
    ///
    /// Returns:
    /// - The last bit evaluation if the sign bit varies across elements
    /// - Zero if the sign is constantly 0
    /// - The chi evaluation if the sign is constantly 1
    pub fn leading_bit_eval<S: ScalarExt>(
        &self,
        bit_evals: &[S],
        chi_eval: S,
    ) -> Result<S, BitDistributionError> {
        // Check if the sign bit varies (MSB of vary_mask is set)
        if U256::from(self.vary_mask) & (U256::ONE.shl(255)) != U256::ZERO {
            // Sign varies, use the last bit evaluation
            bit_evals
                .last()
                .ok_or(BitDistributionError::NoLeadBit)
                .copied()
        } else if U256::from(self.leading_bit_mask) & U256::ONE.shl(255) == U256::ZERO {
            // Sign is constantly 0
            Ok(S::ZERO)
        } else {
            // Sign is constantly 1
            Ok(chi_eval)
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

    /// Returns an iterator over the bit positions that vary across the dataset.
    /// Each yielded value is the index (0-255) of a bit position that has different values
    /// across different elements in the original data.
    #[expect(clippy::missing_panics_doc)]
    pub fn vary_mask_iter(&self) -> impl Iterator<Item = u8> + '_ {
        // Iterate through each 64-bit chunk of the 256-bit vary_mask
        (0..4).flat_map(|i| {
            BitIter::from(self.vary_mask[i])
                .iter()
                .map(move |pos| u8::try_from(i * 64 + pos).expect("index greater than 255"))
        })
    }
}
