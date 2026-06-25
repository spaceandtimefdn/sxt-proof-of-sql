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
    use crate::base::scalar::test_scalar::TestScalar;

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn empty_point_fills_with_one() {
        let mut v = alloc::vec![ts(0); 1];
        compute_evaluation_vector(&mut v, &[]);
        assert_eq!(v[0], ts(1));
    }

    #[test]
    fn single_var_at_zero_gives_one_zero() {
        let mut v = alloc::vec![ts(0), ts(0)];
        compute_evaluation_vector(&mut v, &[ts(0)]);
        assert_eq!(v[0], ts(1)); // 1 - 0 = 1
        assert_eq!(v[1], ts(0)); // 0
    }

    #[test]
    fn single_var_at_one_gives_zero_one() {
        let mut v = alloc::vec![ts(0), ts(0)];
        compute_evaluation_vector(&mut v, &[ts(1)]);
        assert_eq!(v[0], ts(0)); // 1 - 1 = 0
        assert_eq!(v[1], ts(1)); // 1
    }

    #[test]
    fn two_vars_evaluates_to_four_basis_values() {
        // point = [0, 0] => basis = [1, 0, 0, 0]
        let mut v = alloc::vec![ts(0); 4];
        compute_evaluation_vector(&mut v, &[ts(0), ts(0)]);
        assert_eq!(v[0], ts(1));
        assert_eq!(v[1], ts(0));
        assert_eq!(v[2], ts(0));
        assert_eq!(v[3], ts(0));
    }

    #[test]
    fn basis_values_sum_to_one_for_any_point() {
        let mut v = alloc::vec![ts(0); 4];
        compute_evaluation_vector(&mut v, &[ts(3), ts(5)]);
        let sum: TestScalar = v.iter().fold(ts(0), |acc, &x| acc + x);
        assert_eq!(sum, ts(1));
    }

    #[test]
    fn length_one_vector_fills_with_one() {
        let mut v = alloc::vec![ts(0); 1];
        compute_evaluation_vector(&mut v, &[ts(7)]);
        // Only one element, just 1-p term
        assert_eq!(v[0], ts(1) - ts(7));
    }
}
