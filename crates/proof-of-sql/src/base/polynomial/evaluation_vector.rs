use crate::{base::if_rayon, utils::log};
use core::{
    cmp,
    ops::{Mul, MulAssign, Sub, SubAssign},
};
use num_traits::One;
#[cfg(feature = "rayon")]
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

#[cfg(feature = "rayon")]
const MIN_PARALLEL_LEN: usize = 16; // The minimum size for which we should actually parallelize the compute.

/// This method manipulates left and right such that
/// right[i] = left[i] * p and left[i] = left[i] * (1 - p)
fn compute_evaluation_vector_impl<F>(left: &mut [F], right: &mut [F], p: F)
where
    F: One + Sub<Output = F> + MulAssign + SubAssign + Mul<Output = F> + Send + Sync + Copy,
{
    let k = cmp::min(left.len(), right.len());
    let one_minus_p = F::one() - p;
    if_rayon!(
        left.par_iter_mut().with_min_len(MIN_PARALLEL_LEN),
        left.iter_mut()
    )
    .zip(right)
    .for_each(|(li, ri)| {
        *ri = *li * p;
        *li -= *ri;
    });
    if_rayon!(
        left[k..].par_iter_mut().with_min_len(MIN_PARALLEL_LEN),
        left[k..].iter_mut()
    )
    .for_each(|li| {
        *li *= one_minus_p;
    });
}

/// Given a point of evaluation, computes the vector that allows us
/// to evaluate a multilinear extension as an inner product.
#[tracing::instrument(level = "debug", skip_all)]
pub fn compute_evaluation_vector<F>(v: &mut [F], point: &[F])
where
    F: One + Sub<Output = F> + MulAssign + SubAssign + Mul<Output = F> + Send + Sync + Copy,
{
    log::log_memory_usage("Start");

    assert!(v.len() <= (1 << point.len()));
    if point.is_empty() || v.is_empty() {
        // v is guaranteed to be at most length 1 by the assert!.
        v.fill(F::one());
        return;
    }
    v[0] = F::one() - point[0];
    if v.len() > 1 {
        v[1] = point[0];
    }
    for (level, p) in point[1..].iter().enumerate() {
        let mid = 1 << (level + 1);
        let (left, right): (&mut [F], &mut [F]) = if mid >= v.len() {
            (v, &mut [])
        } else {
            v.split_at_mut(mid)
        };
        compute_evaluation_vector_impl(left, right, *p);
    }

    log::log_memory_usage("End");
}

#[cfg(test)]
mod tests {
    use super::compute_evaluation_vector;

    #[test]
    fn empty_v_with_empty_point_remains_empty() {
        let mut v: alloc::vec::Vec<f64> = alloc::vec![];
        compute_evaluation_vector(&mut v, &[]);
        assert!(v.is_empty());
    }

    #[test]
    fn single_element_v_with_empty_point_becomes_one() {
        let mut v = alloc::vec![0.0f64];
        compute_evaluation_vector(&mut v, &[]);
        assert_eq!(v, alloc::vec![1.0]);
    }

    #[test]
    fn two_element_v_with_point_zero_gives_one_zero() {
        let mut v = alloc::vec![0.0f64; 2];
        compute_evaluation_vector(&mut v, &[0.0]);
        assert_eq!(v, alloc::vec![1.0, 0.0]);
    }

    #[test]
    fn two_element_v_with_point_one_gives_zero_one() {
        let mut v = alloc::vec![0.0f64; 2];
        compute_evaluation_vector(&mut v, &[1.0]);
        assert_eq!(v, alloc::vec![0.0, 1.0]);
    }

    #[test]
    fn four_element_v_with_point_zero_zero_gives_one_then_zeros() {
        let mut v = alloc::vec![0.0f64; 4];
        compute_evaluation_vector(&mut v, &[0.0, 0.0]);
        assert_eq!(v, alloc::vec![1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn four_element_v_with_point_one_one_gives_zeros_then_one() {
        let mut v = alloc::vec![0.0f64; 4];
        compute_evaluation_vector(&mut v, &[1.0, 1.0]);
        assert_eq!(v, alloc::vec![0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn four_element_v_with_half_half_gives_quarter_each() {
        let mut v = alloc::vec![0.0f64; 4];
        compute_evaluation_vector(&mut v, &[0.5, 0.5]);
        for x in &v {
            assert!((*x - 0.25).abs() < 1e-10, "expected 0.25 got {x}");
        }
    }

    #[test]
    fn evaluation_vector_sums_to_one() {
        let mut v = alloc::vec![0.0f64; 8];
        compute_evaluation_vector(&mut v, &[0.3, 0.7, 0.5]);
        let sum: f64 = v.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn single_element_v_with_nonempty_point_fills_with_one() {
        let mut v = alloc::vec![0.0f64; 1];
        compute_evaluation_vector(&mut v, &[0.3]);
        assert_eq!(v, alloc::vec![1.0]);
    }

    #[test]
    fn two_element_v_with_point_half_gives_half_half() {
        let mut v = alloc::vec![0.0f64; 2];
        compute_evaluation_vector(&mut v, &[0.5]);
        assert!((v[0] - 0.5).abs() < 1e-10);
        assert!((v[1] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn four_element_v_with_point_one_zero_gives_zero_one_zero_zero() {
        let mut v = alloc::vec![0.0f64; 4];
        compute_evaluation_vector(&mut v, &[1.0, 0.0]);
        assert_eq!(v, alloc::vec![0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn four_element_v_with_point_zero_one_gives_zero_zero_one_zero() {
        let mut v = alloc::vec![0.0f64; 4];
        compute_evaluation_vector(&mut v, &[0.0, 1.0]);
        assert_eq!(v, alloc::vec![0.0, 0.0, 1.0, 0.0]);
    }
}

