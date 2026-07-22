//! These functions are adapted from arkworks. <https://github.com/arkworks-rs/algebra/blob/ab13aa09ae3c11cde0224028dee7b878bbcf9246/ff/src/fields/mod.rs#L347-L410>
//! See `third_party/license/arkworks.LICENSE`
//!
//! They differ in that they don't rely on the `Field` trait, but instead use `core::ops` and `crate::base::scalar` traits.
//! This results in minor modifications.
//!
//! Additionally, `num_elem_per_thread` rounds up instead of down.

use crate::base::if_rayon;
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use core::cmp::max;
use core::ops::{Mul, MulAssign};
use num_traits::{Inv, One, Zero};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/*
 * Adapted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */

/// Given a vector of field elements `{v_i}`, compute the vector `{v_i^(-1)}` using Montgomery's trick.
/// The vector is modified in place.
/// Any zero elements in the vector are left unchanged.
///
/// # Panics
/// - Panics if the inversion of `tmp` fails, which can happen if `tmp` is zero,
///   although this case is guaranteed to be non-zero based on the preceding logic.
#[tracing::instrument(name = "BatchInversion::batch_inversion", level = "debug", skip_all)]
pub fn batch_inversion<F>(v: &mut [F])
where
    F: One + Zero + MulAssign + Inv<Output = Option<F>> + Mul<Output = F> + Send + Sync + Copy,
{
    batch_inversion_and_mul(v, F::one());
}

#[tracing::instrument(
    name = "BatchInversion::batch_inversion_and_mul",
    level = "debug",
    skip_all
)]
pub fn batch_inversion_and_mul<F>(v: &mut [F], coeff: F)
where
    F: One + Zero + MulAssign + Inv<Output = Option<F>> + Mul<Output = F> + Send + Sync + Copy,
{
    if_rayon!(
        {
            // Divide the vector v evenly between all available cores, but make sure that each
            // core has at least MIN_RAYON_LEN elements to work on
            let num_cpus_available = max(1, rayon::current_num_threads());
            let num_elem_per_thread =
                max(v.len().div_ceil(num_cpus_available), super::MIN_RAYON_LEN);

            // Batch invert in parallel, without copying the vector
            v.par_chunks_mut(num_elem_per_thread).for_each(|chunk| {
                serial_batch_inversion_and_mul(chunk, coeff);
            });
        },
        serial_batch_inversion_and_mul(v, coeff)
    );
}

/// # Panics
/// * This function panics if the inversion operation (`inv()`) fails, which can happen if the slice
///   contains any zero elements. However, zero elements are skipped, so this unwrap is guaranteed
///   to succeed unless all elements are zero.
fn serial_batch_inversion_and_mul<F>(v: &mut [F], coeff: F)
where
    F: One + Zero + MulAssign + Inv<Output = Option<F>> + Mul<Output = F> + Copy,
{
    // Montgomery’s Trick and Fast Implementation of Masked AES
    // Genelle, Prouff and Quisquater
    // Section 3.2
    // but with an optimization to multiply every element in the returned vector by
    // coeff

    // First pass: compute [a, ab, abc, ...]
    let mut prod = Vec::with_capacity(v.len());
    let mut tmp = F::one();
    for &f in v.iter().filter(|f| !f.is_zero()) {
        tmp *= f;
        prod.push(tmp);
    }

    // Invert `tmp`.
    tmp = tmp.inv().unwrap(); // Guaranteed to be nonzero.

    // Multiply product by coeff, so all inverses will be scaled by coeff
    tmp *= coeff;

    // Second pass: iterate backwards to compute inverses
    for (f, s) in v
        .iter_mut()
        // Backwards
        .rev()
        // Ignore normalized elements
        .filter(|f| !f.is_zero())
        // Backwards, skip last element, fill in one for last term.
        .zip(prod.into_iter().rev().skip(1).chain(Some(F::one())))
    {
        // tmp := tmp * f; f := tmp * s = 1/f
        let new_tmp = tmp * *f;
        *f = tmp * s;
        tmp = new_tmp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::{test_scalar::TestScalar, Scalar};
    use alloc::vec;

    fn s(x: i64) -> TestScalar {
        TestScalar::from(x)
    }

    #[test]
    fn batch_inversion_on_empty_slice_is_a_noop() {
        let mut v: Vec<TestScalar> = vec![];
        batch_inversion(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn batch_inversion_of_a_single_nonzero_element_yields_its_inverse() {
        let mut v = vec![s(7)];
        batch_inversion(&mut v);
        assert_eq!(v[0] * s(7), TestScalar::ONE);
    }

    #[test]
    fn batch_inversion_leaves_a_single_zero_unchanged() {
        let mut v = vec![TestScalar::ZERO];
        batch_inversion(&mut v);
        assert_eq!(v[0], TestScalar::ZERO);
    }

    #[test]
    fn batch_inversion_inverts_each_nonzero_element() {
        let original: Vec<TestScalar> = (1..=8).map(s).collect();
        let mut v = original.clone();
        batch_inversion(&mut v);
        for (orig, inv) in original.iter().zip(v.iter()) {
            assert_eq!(*orig * *inv, TestScalar::ONE);
        }
    }

    #[test]
    fn batch_inversion_handles_zeros_mixed_with_nonzeros() {
        let original = vec![s(2), TestScalar::ZERO, s(3), TestScalar::ZERO, s(5)];
        let mut v = original.clone();
        batch_inversion(&mut v);
        // Zeros stay zero, non-zeros are inverted.
        assert_eq!(v[1], TestScalar::ZERO);
        assert_eq!(v[3], TestScalar::ZERO);
        assert_eq!(v[0] * original[0], TestScalar::ONE);
        assert_eq!(v[2] * original[2], TestScalar::ONE);
        assert_eq!(v[4] * original[4], TestScalar::ONE);
    }

    #[test]
    fn batch_inversion_is_idempotent_round_trip() {
        // Applying inversion twice should recover the original (modulo the field).
        let original: Vec<TestScalar> = vec![s(2), s(3), s(5), s(7), s(11)];
        let mut v = original.clone();
        batch_inversion(&mut v);
        batch_inversion(&mut v);
        assert_eq!(v, original);
    }

    #[test]
    fn batch_inversion_and_mul_with_one_matches_plain_inversion() {
        let original: Vec<TestScalar> = (1..=5).map(s).collect();
        let mut a = original.clone();
        let mut b = original;
        batch_inversion(&mut a);
        batch_inversion_and_mul(&mut b, TestScalar::ONE);
        assert_eq!(a, b);
    }

    #[test]
    fn batch_inversion_and_mul_scales_each_inverse_by_coeff() {
        let original: Vec<TestScalar> = (1..=5).map(s).collect();
        let coeff = s(13);
        let mut scaled = original.clone();
        batch_inversion_and_mul(&mut scaled, coeff);
        for (orig, scaled_inv) in original.iter().zip(scaled.iter()) {
            // scaled_inv should equal coeff / orig, so orig * scaled_inv == coeff.
            assert_eq!(*orig * *scaled_inv, coeff);
        }
    }

    #[test]
    fn batch_inversion_and_mul_skips_zeros_even_when_scaled() {
        let original = vec![s(2), TestScalar::ZERO, s(4)];
        let mut v = original.clone();
        let coeff = s(7);
        batch_inversion_and_mul(&mut v, coeff);
        assert_eq!(v[1], TestScalar::ZERO);
        assert_eq!(original[0] * v[0], coeff);
        assert_eq!(original[2] * v[2], coeff);
    }
}
