use crate::base::{
    if_rayon,
    scalar::{Scalar, ScalarExt},
};
use alloc::vec::Vec;
use core::{iter::Sum, ops::Mul};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// This operation takes the inner product of two slices. In other words, it does `a[0] * b[0] + a[1] * b[1] + ... + a[n] * b[n]`.
/// If one of the slices is longer than the other, the extra elements are ignored/considered to be 0.
pub fn inner_product<'a, F, T>(a: &[F], b: &'a [T]) -> F
where
    F: Sync + Send + Mul<Output = F> + Sum + Copy,
    &'a T: Into<F>,
    T: Sync,
{
    if_rayon!(a.par_iter().with_min_len(super::MIN_RAYON_LEN), a.iter())
        .zip(b)
        .map(|(&a, b)| a * b.into())
        .sum()
}

pub fn inner_product_ref_cast<F, T>(a: &[F], b: &[T]) -> T
where
    for<'a> &'a F: Into<T>,
    F: Send + Sync,
    T: Sync + Send + Mul<Output = T> + Sum + Copy,
{
    if_rayon!(a.par_iter().with_min_len(super::MIN_RAYON_LEN), a.iter())
        .zip(b)
        .map(|(a, b)| a.into() * *b)
        .sum()
}

/// Cannot use blanket impls for `Vec<u8>` because bytes might have different embeddings as scalars
pub fn inner_product_with_bytes<S: Scalar>(a: &[Vec<u8>], b: &[S]) -> S {
    if_rayon!(a.par_iter().with_min_len(super::MIN_RAYON_LEN), a.iter())
        .zip(b)
        .map(|(lhs_bytes, &rhs)| S::from_byte_slice_via_hash(lhs_bytes) * rhs)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::inner_product;

    #[test]
    fn inner_product_empty_slices_returns_zero() {
        let result = inner_product::<i64, i64>(&[], &[]);
        assert_eq!(result, 0);
    }

    #[test]
    fn inner_product_single_element() {
        let result = inner_product::<i64, i64>(&[5], &[3]);
        assert_eq!(result, 15);
    }

    #[test]
    fn inner_product_two_elements() {
        let result = inner_product::<i64, i64>(&[1, 2], &[3, 4]);
        assert_eq!(result, 11); // 1*3 + 2*4 = 3 + 8 = 11
    }

    #[test]
    fn inner_product_three_elements() {
        let result = inner_product::<i64, i64>(&[1, 2, 3], &[4, 5, 6]);
        assert_eq!(result, 32); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    }

    #[test]
    fn inner_product_first_slice_longer_ignores_extra() {
        let result = inner_product::<i64, i64>(&[1, 2, 100], &[3, 4]);
        assert_eq!(result, 11); // 1*3 + 2*4 = 11; 100 is ignored
    }

    #[test]
    fn inner_product_with_zeros() {
        let result = inner_product::<i64, i64>(&[0, 0, 0], &[1, 2, 3]);
        assert_eq!(result, 0);
    }

    #[test]
    fn inner_product_with_negative_values() {
        let result = inner_product::<i64, i64>(&[-1, 2], &[3, -4]);
        assert_eq!(result, -11); // -1*3 + 2*(-4) = -3 - 8 = -11
    }

    #[test]
    fn inner_product_identity_like_operation() {
        let a = vec![1i64, 2, 3, 4, 5];
        let b = vec![1i64; 5];
        let result = inner_product(&a, &b);
        assert_eq!(result, 15); // sum of a
    }
}
