use crate::base::if_rayon;
use alloc::vec::Vec;
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

/// This operation takes a slice and casts it to a vector of a different type using the provided function.
pub fn slice_cast_with<'a, F, T>(value: &'a [F], cast: impl Fn(&'a F) -> T + Send + Sync) -> Vec<T>
where
    F: Sync,
    T: Send,
{
    if_rayon!(
        value.par_iter().with_min_len(super::MIN_RAYON_LEN),
        value.iter()
    )
    .map(cast)
    .collect()
}

/// This operation takes a slice and casts it to a mutable slice of a different type using the provided function.
pub fn slice_cast_mut_with<'a, F, T>(
    value: &'a [F],
    result: &mut [T],
    cast: impl Fn(&'a F) -> T + Sync,
) where
    F: Sync,
    T: Send + Sync,
{
    if_rayon!(
        value.par_iter().with_min_len(super::MIN_RAYON_LEN),
        value.iter()
    )
    .zip(result)
    .for_each(|(a, b)| *b = cast(a));
}

/// This operation takes a slice and casts it to a vector of a different type using the provided function.
pub fn slice_cast<'a, F, T>(value: &'a [F]) -> Vec<T>
where
    F: Sync,
    T: Send,
    &'a F: Into<T>,
{
    slice_cast_with(value, Into::into)
}

/// This operation takes a slice and casts it to a mutable slice of a different type using the provided function.
pub fn slice_cast_mut<'a, F, T>(value: &'a [F], result: &mut [T])
where
    F: Sync,
    T: Send + Sync,
    &'a F: Into<T>,
{
    slice_cast_mut_with(value, result, Into::into);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_cast_with_i32_to_i64() {
        let src: Vec<i32> = vec![1, 2, 3];
        let result: Vec<i64> = slice_cast_with(&src, |x| i64::from(*x));
        assert_eq!(result, vec![1i64, 2, 3]);
    }

    #[test]
    fn test_slice_cast_with_empty() {
        let src: Vec<i32> = vec![];
        let result: Vec<i64> = slice_cast_with(&src, |x| i64::from(*x));
        assert!(result.is_empty());
    }

    #[test]
    fn test_slice_cast_with_custom_fn() {
        let src = vec![1u32, 2, 3, 4];
        let result: Vec<u32> = slice_cast_with(&src, |x| x * 2);
        assert_eq!(result, vec![2, 4, 6, 8]);
    }

    #[test]
    fn test_slice_cast_mut_with_i32_to_i64() {
        let src: Vec<i32> = vec![10, 20, 30];
        let mut dst = vec![0i64; 3];
        slice_cast_mut_with(&src, &mut dst, |x| i64::from(*x));
        assert_eq!(dst, vec![10i64, 20, 30]);
    }

    #[test]
    fn test_slice_cast_mut_with_empty() {
        let src: Vec<i32> = vec![];
        let mut dst: Vec<i64> = vec![];
        slice_cast_mut_with(&src, &mut dst, |x| i64::from(*x));
        assert!(dst.is_empty());
    }

    #[test]
    fn test_slice_cast_i32_to_i64_via_into() {
        let src: Vec<i32> = vec![5, 10, 15];
        let result: Vec<i64> = slice_cast(&src);
        assert_eq!(result, vec![5i64, 10, 15]);
    }

    #[test]
    fn test_slice_cast_empty_via_into() {
        let src: Vec<i32> = vec![];
        let result: Vec<i64> = slice_cast(&src);
        assert!(result.is_empty());
    }

    #[test]
    fn test_slice_cast_mut_i32_to_i64_via_into() {
        let src: Vec<i32> = vec![1, 2, 3];
        let mut dst = vec![0i64; 3];
        slice_cast_mut(&src, &mut dst);
        assert_eq!(dst, vec![1i64, 2, 3]);
    }
}
