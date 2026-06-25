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
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn mul_add_assign_empty_to_mul_add_is_noop() {
        let mut result = alloc::vec![TestScalar::from(1), TestScalar::from(2)];
        mul_add_assign(&mut result, TestScalar::from(5), &[] as &[TestScalar]);
        assert_eq!(result, alloc::vec![TestScalar::from(1), TestScalar::from(2)]);
    }

    #[test]
    fn mul_add_assign_basic_multiply_and_add() {
        let mut result = alloc::vec![TestScalar::from(0); 3];
        let to_add = alloc::vec![TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)];
        mul_add_assign(&mut result, TestScalar::from(2), &to_add);
        assert_eq!(result, alloc::vec![TestScalar::from(2), TestScalar::from(4), TestScalar::from(6)]);
    }

    #[test]
    fn mul_add_assign_accumulates_into_existing_values() {
        let mut result = alloc::vec![TestScalar::from(10), TestScalar::from(20), TestScalar::from(30)];
        let to_add = alloc::vec![TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)];
        mul_add_assign(&mut result, TestScalar::from(3), &to_add);
        assert_eq!(result, alloc::vec![TestScalar::from(13), TestScalar::from(26), TestScalar::from(39)]);
    }

    #[test]
    fn mul_add_assign_zero_multiplier_leaves_result_unchanged() {
        let mut result = alloc::vec![TestScalar::from(5), TestScalar::from(10), TestScalar::from(15)];
        let to_add = alloc::vec![TestScalar::from(100), TestScalar::from(200), TestScalar::from(300)];
        mul_add_assign(&mut result, TestScalar::from(0), &to_add);
        assert_eq!(result, alloc::vec![TestScalar::from(5), TestScalar::from(10), TestScalar::from(15)]);
    }

    #[test]
    fn mul_add_assign_result_longer_than_to_mul_add_only_affects_prefix() {
        let mut result = alloc::vec![
            TestScalar::from(1), TestScalar::from(2), TestScalar::from(3),
            TestScalar::from(4), TestScalar::from(5),
        ];
        let to_add = alloc::vec![TestScalar::from(10), TestScalar::from(20)];
        mul_add_assign(&mut result, TestScalar::from(1), &to_add);
        assert_eq!(result[0], TestScalar::from(11));
        assert_eq!(result[1], TestScalar::from(22));
        assert_eq!(result[2], TestScalar::from(3));
    }

    #[test]
    #[should_panic(expected = "The length of result must be greater than or equal to")]
    fn mul_add_assign_panics_if_result_shorter_than_to_mul_add() {
        let mut result = alloc::vec![TestScalar::from(1), TestScalar::from(2)];
        let to_add = alloc::vec![TestScalar::from(10), TestScalar::from(20), TestScalar::from(30)];
        mul_add_assign(&mut result, TestScalar::from(1), &to_add);
    }

    #[test]
    fn mul_add_assign_single_element() {
        let mut result = alloc::vec![TestScalar::from(5)];
        mul_add_assign(&mut result, TestScalar::from(3), &[TestScalar::from(4)]);
        assert_eq!(result[0], TestScalar::from(17));
    }
}
