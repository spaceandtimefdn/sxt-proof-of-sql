use crate::base::if_rayon;
use core::ops::AddAssign;
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

/// This operation does `result[i] += to_add` for `i` in `0..result.len()`.
pub fn add_const<T, S>(result: &mut [T], to_add: S)
where
    T: Send + Sync + AddAssign<T> + Copy,
    S: Into<T> + Sync + Copy,
{
    if_rayon!(
        result.par_iter_mut().with_min_len(super::MIN_RAYON_LEN),
        result.iter_mut()
    )
    .for_each(|res_i| {
        *res_i += to_add.into();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_const_basic() {
        let mut result = vec![1i64, 2, 3];
        add_const(&mut result, 10i64);
        assert_eq!(result, vec![11, 12, 13]);
    }

    #[test]
    fn test_add_const_zero() {
        let mut result = vec![5i64, 6, 7];
        add_const(&mut result, 0i64);
        assert_eq!(result, vec![5, 6, 7]);
    }

    #[test]
    fn test_add_const_empty_slice() {
        let mut result: Vec<i64> = vec![];
        add_const(&mut result, 42i64); // no-op on empty
        assert!(result.is_empty());
    }

    #[test]
    fn test_add_const_negative() {
        let mut result = vec![10i64, 20, 30];
        add_const(&mut result, -5i64);
        assert_eq!(result, vec![5, 15, 25]);
    }

    #[test]
    fn test_add_const_single_element() {
        let mut result = vec![100u64];
        add_const(&mut result, 1u64);
        assert_eq!(result, vec![101u64]);
    }
}
