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
    fn we_can_add_const_to_an_empty_slice() {
        let mut values: [i32; 0] = [];

        add_const(&mut values, 10);

        assert!(values.is_empty());
    }
}
