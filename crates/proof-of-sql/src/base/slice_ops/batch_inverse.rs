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
    // Montgomery's Trick and Fast Implementation of Masked AES
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
    use num_traits::{Inv, One, Zero};

    #[test]
    fn batch_inversion_empty_slice_is_noop() {
        let mut v: alloc::vec::Vec<TestScalar> = alloc::vec![];
        batch_inversion(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn batch_inversion_single_nonzero_gives_inverse() {
        let a = TestScalar::from(5u64);
        let mut v = alloc::vec![a];
        batch_inversion(&mut v);
        assert_eq!(a * v[0], TestScalar::one());
    }

    #[test]
    fn batch_inversion_two_elements_gives_inverses() {
        let a = TestScalar::from(3u64);
        let b = TestScalar::from(7u64);
        let mut v = alloc::vec![a, b];
        batch_inversion(&mut v);
        assert_eq!(a * v[0], TestScalar::one());
        assert_eq!(b * v[1], TestScalar::one());
    }

    #[test]
    fn batch_inversion_skips_zero_elements() {
        let zero = TestScalar::zero();
        let a = TestScalar::from(4u64);
        let mut v = alloc::vec![zero, a];
        batch_inversion(&mut v);
        assert_eq!(v[0], zero);
        assert_eq!(a * v[1], TestScalar::one());
    }

    #[test]
    fn batch_inversion_five_elements_all_correct() {
        let originals: alloc::vec::Vec<TestScalar> = (1u64..=5).map(TestScalar::from).collect();
        let mut v = originals.clone();
        batch_inversion(&mut v);
        for (orig, inv) in originals.iter().zip(v.iter()) {
            assert_eq!(*orig * *inv, TestScalar::one());
        }
    }

    #[test]
    fn batch_inversion_and_mul_coeff_one_equals_batch_inversion() {
        let a = TestScalar::from(11u64);
        let b = TestScalar::from(13u64);
        let mut v1 = alloc::vec![a, b];
        let mut v2 = alloc::vec![a, b];
        batch_inversion(&mut v1);
        batch_inversion_and_mul(&mut v2, TestScalar::one());
        assert_eq!(v1, v2);
    }

    #[test]
    fn batch_inversion_and_mul_scales_result_by_coeff() {
        let a = TestScalar::from(2u64);
        let coeff = TestScalar::from(3u64);
        let mut v = alloc::vec![a];
        batch_inversion_and_mul(&mut v, coeff);
        let expected = coeff * a.inv().unwrap();
        assert_eq!(v[0], expected);
    }

    #[test]
    fn batch_inversion_single_zero_stays_zero() {
        let zero = TestScalar::zero();
        let mut v = alloc::vec![zero];
        batch_inversion(&mut v);
        assert_eq!(v[0], zero);
    }
}
