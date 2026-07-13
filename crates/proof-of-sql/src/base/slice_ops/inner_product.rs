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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_inner_product_basic() {
        // [1,2,3] · [4,5,6] = 4 + 10 + 18 = 32
        let a = [
            TestScalar::from(1u64),
            TestScalar::from(2u64),
            TestScalar::from(3u64),
        ];
        let b = [
            TestScalar::from(4u64),
            TestScalar::from(5u64),
            TestScalar::from(6u64),
        ];
        let result = inner_product(&a, &b);
        assert_eq!(result, TestScalar::from(32u64));
    }

    #[test]
    fn test_inner_product_empty() {
        let a: Vec<TestScalar> = vec![];
        let b: Vec<TestScalar> = vec![];
        let result = inner_product(&a, &b);
        assert_eq!(result, TestScalar::ZERO);
    }

    #[test]
    fn test_inner_product_single_element() {
        let a = [TestScalar::from(7u64)];
        let b = [TestScalar::from(8u64)];
        let result = inner_product(&a, &b);
        assert_eq!(result, TestScalar::from(56u64));
    }

    #[test]
    fn test_inner_product_longer_b_truncated() {
        // a has 2 elements, b has 3; result should only use first 2 pairs
        let a = [TestScalar::from(1u64), TestScalar::from(2u64)];
        let b = [
            TestScalar::from(3u64),
            TestScalar::from(4u64),
            TestScalar::from(999u64),
        ];
        let result = inner_product(&a, &b);
        assert_eq!(result, TestScalar::from(1 * 3 + 2 * 4));
    }

    #[test]
    fn test_inner_product_zeros() {
        let a = [TestScalar::ZERO, TestScalar::ZERO];
        let b = [TestScalar::from(100u64), TestScalar::from(200u64)];
        let result = inner_product(&a, &b);
        assert_eq!(result, TestScalar::ZERO);
    }
}
