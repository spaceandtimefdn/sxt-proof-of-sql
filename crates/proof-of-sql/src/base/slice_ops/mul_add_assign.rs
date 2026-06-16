use crate::base::if_rayon;
use core::ops::{AddAssign, Mul};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

/// This operation does `result[i] += multiplier * to_mul_add[i]` for `i` in `0..to_mul_add.len()`
/// without creating temporary vectors. Works directly with slice references.
///
/// # Panics
/// Panics if the length of `result` is less than the length of `to_mul_add`.
pub fn mul_add_assign<'a, T, S>(result: &mut [T], multiplier: T, to_mul_add: &'a [S])
where
    T: Send + Sync + Mul<Output = T> + AddAssign + Copy,
    &'a S: Into<T>,
    S: Sync,
{
    assert!(result.len() >= to_mul_add.len(), "The length of result must be greater than or equal to the length of the vector of values to be multiplied and added");
    if_rayon!(
        result.par_iter_mut().with_min_len(super::MIN_RAYON_LEN),
        result.iter_mut()
    )
    .zip(to_mul_add)
    .for_each(|(res_i, data_i)| {
        *res_i += multiplier * data_i.into();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_add_assign_basic() {
        // result[i] += 2 * to_mul_add[i]
        let mut result = vec![1i64, 2, 3];
        let to_mul_add = vec![10i64, 20, 30];
        mul_add_assign(&mut result, 2, &to_mul_add);
        assert_eq!(result, vec![1 + 2 * 10, 2 + 2 * 20, 3 + 2 * 30]);
    }

    #[test]
    fn test_mul_add_assign_zero_multiplier() {
        let mut result = vec![5i64, 6, 7];
        let to_mul_add = vec![100i64, 200, 300];
        mul_add_assign(&mut result, 0, &to_mul_add);
        assert_eq!(result, vec![5, 6, 7]);
    }

    #[test]
    fn test_mul_add_assign_partial_overlap() {
        // result is longer than to_mul_add; only first elements are updated
        let mut result = vec![1i64, 2, 3, 4, 5];
        let to_mul_add = vec![10i64, 20];
        mul_add_assign(&mut result, 3, &to_mul_add);
        assert_eq!(result, vec![1 + 30, 2 + 60, 3, 4, 5]);
    }

    #[test]
    fn test_mul_add_assign_empty_to_mul_add() {
        let mut result = vec![1i64, 2, 3];
        let to_mul_add: Vec<i64> = vec![];
        mul_add_assign(&mut result, 5, &to_mul_add);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "The length of result must be greater than or equal")]
    fn test_mul_add_assign_panics_when_result_shorter() {
        let mut result = vec![1i64, 2];
        let to_mul_add = vec![10i64, 20, 30];
        mul_add_assign(&mut result, 1, &to_mul_add);
    }
}
