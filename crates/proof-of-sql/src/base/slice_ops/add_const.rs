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
    use super::add_const;

    #[test]
    fn add_const_empty_slice_is_noop() {
        let mut result: Vec<i32> = vec![];
        add_const(&mut result, 5i32);
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn add_const_adds_to_every_element() {
        let mut result = vec![1i32, 2, 3, 4];
        add_const(&mut result, 10i32);
        assert_eq!(result, vec![11, 12, 13, 14]);
    }

    #[test]
    fn add_const_zero_is_identity() {
        let mut result = vec![5i32, 10, 15];
        add_const(&mut result, 0i32);
        assert_eq!(result, vec![5, 10, 15]);
    }

    #[test]
    fn add_const_negative_value_decrements() {
        let mut result = vec![10i32, 20, 30];
        add_const(&mut result, -5i32);
        assert_eq!(result, vec![5, 15, 25]);
    }

    #[test]
    fn add_const_single_element() {
        let mut result = vec![42i32];
        add_const(&mut result, 1i32);
        assert_eq!(result, vec![43]);
    }

    #[test]
    fn add_const_with_u64() {
        let mut result = vec![100u64, 200, 300];
        add_const(&mut result, 50u64);
        assert_eq!(result, vec![150, 250, 350]);
    }

    #[test]
    fn add_const_accumulates_over_multiple_calls() {
        let mut result = vec![0i32; 3];
        add_const(&mut result, 5i32);
        add_const(&mut result, 3i32);
        assert_eq!(result, vec![8, 8, 8]);
    }
}
