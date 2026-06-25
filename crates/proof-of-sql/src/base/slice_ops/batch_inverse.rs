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
    use super::{batch_inversion, batch_inversion_and_mul};
    use crate::base::scalar::test_scalar::TestScalar;

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn batch_inversion_empty_slice_does_not_panic() {
        let mut v: Vec<TestScalar> = alloc::vec![];
        batch_inversion(&mut v);
    }

    #[test]
    fn batch_inversion_single_element_inverts() {
        // inv(2) * 2 == 1
        let mut v = alloc::vec![ts(2)];
        batch_inversion(&mut v);
        assert_eq!(v[0] * ts(2), ts(1));
    }

    #[test]
    fn batch_inversion_skips_zeros_leaving_them_unchanged() {
        let mut v = alloc::vec![ts(0), ts(2)];
        batch_inversion(&mut v);
        assert_eq!(v[0], ts(0)); // zero unchanged
        assert_eq!(v[1] * ts(2), ts(1)); // inv(2) correct
    }

    #[test]
    fn batch_inversion_and_mul_multiplies_by_coeff() {
        // inv(2) * coeff=3 => 3 * inv(2) 
        // check: result * 2 == 3
        let mut v = alloc::vec![ts(2)];
        batch_inversion_and_mul(&mut v, ts(3));
        assert_eq!(v[0] * ts(2), ts(3));
    }

    #[test]
    fn batch_inversion_and_mul_empty_does_not_panic() {
        let mut v: Vec<TestScalar> = alloc::vec![];
        batch_inversion_and_mul(&mut v, ts(5));
    }

    #[test]
    fn batch_inversion_multiple_elements_all_correct() {
        let mut v = alloc::vec![ts(2), ts(3)];
        batch_inversion(&mut v);
        // inv(2) * 2 = 1
        assert_eq!(v[0] * ts(2), ts(1));
        // inv(3) * 3 = 1
        assert_eq!(v[1] * ts(3), ts(1));
    }
}
