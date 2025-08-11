use super::bit_mask_utils::make_bit_mask;
use crate::{
    base::{bit::BitDistribution, if_rayon, scalar::Scalar},
    utils::log,
};
use alloc::vec::Vec;
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shl;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::{span, Level};

/// Let `x1, ..., xn` denote the values of a data column. Let
/// `b1, ..., bk` denote the bit positions of `abs(x1), ..., abs(xn)`
/// that vary.
///
/// `compute_varying_bit_matrix` returns the matrix M where
///   `M_ij = abs(xi) & (1 << bj) == 1`
/// The last column of M corresponds to the sign bit if it varies.
#[tracing::instrument(
    name = "BitMatrix::compute_varying_bit_matrix",
    level = "debug",
    skip_all
)]
pub fn compute_varying_bit_matrix<'a, S: Scalar>(
    alloc: &'a Bump,
    vals: &[S],
    dist: &BitDistribution,
) -> Vec<&'a [bool]> {
    log::start();

    let span = span!(Level::DEBUG, "allocate").entered();
    let number_of_scalars = vals.len();
    let num_varying_bits = dist.num_varying_bits();
    let data: &'a mut [bool] = alloc.alloc_slice_fill_default(number_of_scalars * num_varying_bits);
    span.exit();

    // decompose
    let span = span!(Level::DEBUG, "decompose").entered();
    let masks: Vec<U256> = if_rayon!(vals.par_iter(), vals.iter())
        .copied()
        .map(make_bit_mask)
        .collect();
    log::log_vector("masks", &masks);

    let shifted_masks: Vec<U256> = dist
        .vary_mask_iter()
        .map(|bit_index| U256::ONE.shl(bit_index))
        .collect();
    log::log_vector("shifted_masks", &shifted_masks);

    let span_fill_data = span!(Level::DEBUG, "fill data").entered();
    for (scalar_index, mask) in masks.into_iter().enumerate() {
        for (vary_index, shifted_mask) in shifted_masks.iter().enumerate() {
            data[scalar_index + vary_index * number_of_scalars] =
                (mask & shifted_mask) != U256::ZERO;
        }
    }
    span_fill_data.exit();
    span.exit();

    // make result
    let mut res = Vec::with_capacity(num_varying_bits);
    for bit_index in 0..num_varying_bits {
        let first = number_of_scalars * bit_index;
        let last = number_of_scalars * (bit_index + 1);
        res.push(&data[first..last]);
    }

    log::stop();

    res
}
