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
    use alloc::{string::String, vec};

    #[test]
    fn slice_cast_with_empty_slice_returns_empty_vec() {
        let result: Vec<i64> = slice_cast_with(&[] as &[i32], |&x| x as i64);
        assert_eq!(result, Vec::<i64>::new());
    }

    #[test]
    fn slice_cast_with_converts_i32_to_i64() {
        let result: Vec<i64> = slice_cast_with(&[1i32, 2, 3], |&x| x as i64);
        assert_eq!(result, vec![1i64, 2, 3]);
    }

    #[test]
    fn slice_cast_with_applies_transformation() {
        let result: Vec<i32> = slice_cast_with(&[1i32, 2, 3, 4], |&x| x * x);
        assert_eq!(result, vec![1, 4, 9, 16]);
    }

    #[test]
    fn slice_cast_with_converts_to_string() {
        let result: Vec<String> = slice_cast_with(&[1i32, 2, 3], |&x| x.to_string());
        assert_eq!(result, vec!["1", "2", "3"]);
    }

    #[test]
    fn slice_cast_mut_with_empty_slices_is_noop() {
        let mut result: Vec<i64> = vec![];
        slice_cast_mut_with(&[] as &[i32], &mut result, |&x| x as i64);
        assert_eq!(result, Vec::<i64>::new());
    }

    #[test]
    fn slice_cast_mut_with_writes_to_result() {
        let mut result = vec![0i64; 3];
        slice_cast_mut_with(&[10i32, 20, 30], &mut result, |&x| x as i64 * 2);
        assert_eq!(result, vec![20i64, 40, 60]);
    }

    #[test]
    fn slice_cast_mut_with_partial_write() {
        let mut result = vec![0i64; 5];
        slice_cast_mut_with(&[1i32, 2], &mut result, |&x| x as i64);
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 2);
        assert_eq!(result[2], 0); // untouched
    }

    #[test]
    fn slice_cast_with_single_element() {
        let result: Vec<i64> = slice_cast_with(&[42i32], |&x| x as i64);
        assert_eq!(result, vec![42i64]);
    }
}
