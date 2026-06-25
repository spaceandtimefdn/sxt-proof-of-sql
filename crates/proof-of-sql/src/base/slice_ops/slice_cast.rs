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
    use super::{slice_cast_mut_with, slice_cast_with};

    #[test]
    fn slice_cast_with_converts_each_element() {
        let v = alloc::vec![1i32, 2, 3];
        let result = slice_cast_with(&v, |&x| x as i64 * 2);
        assert_eq!(result, alloc::vec![2i64, 4, 6]);
    }

    #[test]
    fn slice_cast_with_empty_slice() {
        let v: &[i32] = &[];
        let result = slice_cast_with(v, |&x| x as i64);
        assert!(result.is_empty());
    }

    #[test]
    fn slice_cast_mut_with_writes_to_output() {
        let v = alloc::vec![10i32, 20, 30];
        let mut result = alloc::vec![0i64; 3];
        slice_cast_mut_with(&v, &mut result, |&x| x as i64 + 1);
        assert_eq!(result, alloc::vec![11i64, 21, 31]);
    }

    #[test]
    fn slice_cast_mut_with_empty_slice() {
        let v: &[i32] = &[];
        let mut result: alloc::vec::Vec<i64> = alloc::vec![];
        slice_cast_mut_with(v, &mut result, |&x| x as i64);
        assert!(result.is_empty());
    }

    #[test]
    fn slice_cast_with_negate() {
        let v = alloc::vec![3i32, -1, 5];
        let result = slice_cast_with(&v, |&x| -x);
        assert_eq!(result, alloc::vec![-3i32, 1, -5]);
    }
}
