use super::bit_mask_utils::make_bit_mask;
use crate::base::{bit::BitDistribution, scalar::Scalar};
use alloc::vec::Vec;
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shl;

/// Let `x1, ..., xn` denote the values of a data column. Let
/// `b1, ..., bk` denote the bit positions of `abs(x1), ..., abs(xn)`
/// that vary.
///
/// `compute_varying_bit_matrix` returns the matrix M where
///   `M_ij = abs(xi) & (1 << bj) == 1`
/// The last column of M corresponds to the sign bit if it varies.
pub fn compute_varying_bit_matrix<'a, S: Scalar>(
    alloc: &'a Bump,
    vals: &[S],
    dist: &BitDistribution,
) -> Vec<&'a [bool]> {
    let number_of_scalars = vals.len();
    let num_varying_bits = dist.num_varying_bits();
    let data: &'a mut [bool] = alloc.alloc_slice_fill_default(number_of_scalars * num_varying_bits);

    // decompose
    for (scalar_index, val) in vals.iter().enumerate() {
        let mask = make_bit_mask(*val);
        for (vary_index, bit_index) in dist.vary_mask_iter().enumerate() {
            data[scalar_index + vary_index * number_of_scalars] =
                (mask & U256::ONE.shl(bit_index)) != U256::ZERO;
        }
    }

    // make result
    let mut res = Vec::with_capacity(num_varying_bits);
    for bit_index in 0..num_varying_bits {
        let first = number_of_scalars * bit_index;
        let last = number_of_scalars * (bit_index + 1);
        res.push(&data[first..last]);
    }
    res
}
