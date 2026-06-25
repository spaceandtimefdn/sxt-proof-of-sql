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
    use crate::base::scalar::test_scalar::TestScalar;

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn inner_product_empty_slices_returns_zero() {
        let result = inner_product::<TestScalar, TestScalar>(&[], &[]);
        assert_eq!(result, ts(0));
    }

    #[test]
    fn inner_product_single_element() {
        let result = inner_product(&[ts(5)], &[ts(3)]);
        assert_eq!(result, ts(15));
    }

    #[test]
    fn inner_product_two_elements() {
        // 1*3 + 2*4 = 3 + 8 = 11
        let result = inner_product(&[ts(1), ts(2)], &[ts(3), ts(4)]);
        assert_eq!(result, ts(11));
    }

    #[test]
    fn inner_product_three_elements() {
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        let result = inner_product(&[ts(1), ts(2), ts(3)], &[ts(4), ts(5), ts(6)]);
        assert_eq!(result, ts(32));
    }

    #[test]
    fn inner_product_first_slice_longer_ignores_extra() {
        // 1*3 + 2*4 = 11; third element (100) ignored
        let result = inner_product(&[ts(1), ts(2), ts(100)], &[ts(3), ts(4)]);
        assert_eq!(result, ts(11));
    }

    #[test]
    fn inner_product_with_zeros() {
        let result = inner_product(&[ts(0), ts(0), ts(0)], &[ts(1), ts(2), ts(3)]);
        assert_eq!(result, ts(0));
    }

    #[test]
    fn inner_product_identity_like_operation() {
        // a · [1,1,1,1,1] = sum(a)
        let a = alloc::vec![ts(1), ts(2), ts(3), ts(4), ts(5)];
        let ones = alloc::vec![ts(1); 5];
        let result = inner_product(&a, &ones);
        assert_eq!(result, ts(15));
    }

    #[test]
    fn inner_product_with_negative_values() {
        // (-1)*3 + 2*(-4) = -3 - 8 = -11
        let result = inner_product(&[ts(-1), ts(2)], &[ts(3), ts(-4)]);
        assert_eq!(result, ts(-11));
    }
}
