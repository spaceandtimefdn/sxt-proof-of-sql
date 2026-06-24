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
    use super::mul_add_assign;

    #[test]
    fn mul_add_assign_empty_to_mul_add_is_noop() {
        let mut result = vec![1i32, 2, 3];
        mul_add_assign(&mut result, 5i32, &[] as &[i32]);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn mul_add_assign_basic_multiply_and_add() {
        let mut result = vec![0i32; 3];
        let to_add = vec![1i32, 2, 3];
        mul_add_assign(&mut result, 2i32, &to_add);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn mul_add_assign_accumulates_into_existing_values() {
        let mut result = vec![10i32, 20, 30];
        let to_add = vec![1i32, 2, 3];
        mul_add_assign(&mut result, 3i32, &to_add);
        assert_eq!(result, vec![13, 26, 39]);
    }

    #[test]
    fn mul_add_assign_zero_multiplier_leaves_result_unchanged() {
        let mut result = vec![5i32, 10, 15];
        let to_add = vec![100i32, 200, 300];
        mul_add_assign(&mut result, 0i32, &to_add);
        assert_eq!(result, vec![5, 10, 15]);
    }

    #[test]
    fn mul_add_assign_result_longer_than_to_mul_add_only_affects_prefix() {
        let mut result = vec![1i32, 2, 3, 4, 5];
        let to_add = vec![10i32, 20];
        mul_add_assign(&mut result, 1i32, &to_add);
        assert_eq!(result, vec![11, 22, 3, 4, 5]);
    }

    #[test]
    #[should_panic(expected = "The length of result must be greater than or equal to")]
    fn mul_add_assign_panics_if_result_shorter_than_to_mul_add() {
        let mut result = vec![1i32, 2];
        let to_add = vec![10i32, 20, 30];
        mul_add_assign(&mut result, 1i32, &to_add);
    }

    #[test]
    fn mul_add_assign_negative_multiplier() {
        let mut result = vec![100i32, 200, 300];
        let to_add = vec![10i32, 20, 30];
        mul_add_assign(&mut result, -1i32, &to_add);
        assert_eq!(result, vec![90, 180, 270]);
    }

    #[test]
    fn mul_add_assign_single_element() {
        let mut result = vec![5i32];
        mul_add_assign(&mut result, 3i32, &[4i32]);
        assert_eq!(result, vec![17]);
    }
}
