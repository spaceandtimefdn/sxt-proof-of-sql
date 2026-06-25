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

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn mul_add_assign_basic() {
        // result[i] += mul * to_mul_add[i]
        // [0, 0] += 2 * [3, 4] => [6, 8]
        let mut result = alloc::vec![ts(0), ts(0)];
        let to_mul = alloc::vec![ts(3), ts(4)];
        mul_add_assign(&mut result, ts(2), &to_mul);
        assert_eq!(result[0], ts(6));
        assert_eq!(result[1], ts(8));
    }

    #[test]
    fn mul_add_assign_adds_to_existing() {
        // result = [10, 20], += 1 * [5, 5] => [15, 25]
        let mut result = alloc::vec![ts(10), ts(20)];
        let to_mul = alloc::vec![ts(5), ts(5)];
        mul_add_assign(&mut result, ts(1), &to_mul);
        assert_eq!(result[0], ts(15));
        assert_eq!(result[1], ts(25));
    }

    #[test]
    fn mul_add_assign_with_zero_multiplier() {
        // mul=0 => result unchanged
        let mut result = alloc::vec![ts(7), ts(8)];
        let to_mul = alloc::vec![ts(100), ts(200)];
        mul_add_assign(&mut result, ts(0), &to_mul);
        assert_eq!(result[0], ts(7));
        assert_eq!(result[1], ts(8));
    }

    #[test]
    fn mul_add_assign_empty_to_mul() {
        let mut result = alloc::vec![ts(5), ts(6)];
        let to_mul: alloc::vec::Vec<TestScalar> = alloc::vec![];
        mul_add_assign(&mut result, ts(3), &to_mul);
        assert_eq!(result[0], ts(5)); // unchanged
    }

    #[test]
    fn mul_add_assign_partial_update() {
        // result has 3 elements, to_mul has 2 → only first 2 updated
        let mut result = alloc::vec![ts(0), ts(0), ts(99)];
        let to_mul = alloc::vec![ts(1), ts(2)];
        mul_add_assign(&mut result, ts(3), &to_mul);
        assert_eq!(result[0], ts(3));
        assert_eq!(result[1], ts(6));
        assert_eq!(result[2], ts(99)); // unchanged
    }
}
