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
