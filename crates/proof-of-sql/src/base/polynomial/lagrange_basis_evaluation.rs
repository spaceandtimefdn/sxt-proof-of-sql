use core::ops::{Add, Mul, Sub};
use num_traits::{One, Zero};

/// Given the points a and b with length nu, we can evaluate the lagrange basis of length 2^nu at the two points.
/// This is what [`super::compute_evaluation_vector`] does.
/// Call the resulting evaluation vectors A and B. This function computes `sum A[i] * B[i] for i in 0..length`. That is:
/// ```text
/// (1-a[0])(1-a[1])...(1-a[nu-1]) * (1-b[0])(1-b[1])...(1-b[nu-1]) +
/// (a[0])(1-a[1])...(1-a[nu-1]) * (b[0])(1-b[1])...(1-b[nu-1]) +
/// (1-a[0])(a[1])...(1-a[nu-1]) * (1-b[0])(b[1])...(1-b[nu-1]) +
/// (a[0])(a[1])...(1-a[nu-1]) * (b[0])(b[1])...(1-b[nu-1]) + ...
/// ```
pub fn compute_truncated_lagrange_basis_inner_product<F>(length: usize, a: &[F], b: &[F]) -> F
where
    F: One + Zero + Mul<Output = F> + Add<Output = F> + Sub<Output = F> + Copy,
{
    compute_truncated_lagrange_basis_inner_product_impl(length, a, b).0
}

// The returned value from this function is (part, full).
// The full value is what the result would be if it were not truncated. (In other words, if length==2^nu.)
// This can be iteratively used to compute the actual result.
/// # Panics
/// this function requires that `a` and `b` have the same length.
/// This function requires that `length` is less than or equal to `1 << nu` where `nu` is the length of `a` and `b`.
fn compute_truncated_lagrange_basis_inner_product_impl<F>(
    part_length: usize,
    a: &[F],
    b: &[F],
) -> (F, F)
where
    F: One + Zero + Mul<Output = F> + Add<Output = F> + Sub<Output = F> + Copy,
{
    let nu = a.len();
    assert_eq!(nu, b.len());
    if nu == 0 {
        assert!(part_length <= 1);
        if part_length == 1 {
            (F::one(), F::one())
        } else {
            (F::zero(), F::one())
        }
    } else {
        // We split the imaginary full evaluation vector in half.
        // This is the value that needs to be multiplied by every element in the first half.
        let first_half_term = (F::one() - a[nu - 1]) * (F::one() - b[nu - 1]);
        // This is the value that needs to be multiplied by every element in the second half.
        let second_half_term = a[nu - 1] * b[nu - 1];
        let half_full_length = 1 << (nu - 1);

        // `sub` referrs to the sub-iteration. (In other words, removing the last variable, cutting this into two halves.)
        let sub_part_length = if part_length >= half_full_length {
            part_length - half_full_length
        } else {
            part_length
        };
        let (sub_part, sub_full) = compute_truncated_lagrange_basis_inner_product_impl(
            sub_part_length,
            &a[..nu - 1],
            &b[..nu - 1],
        );

        // This is the primary iteration formula.
        let part = if part_length >= half_full_length {
            sub_full * first_half_term + sub_part * second_half_term
        } else {
            sub_part * first_half_term
        };
        // This is the iteration formula for the non truncated version.
        let full = sub_full * (first_half_term + second_half_term);
        (part, full)
    }
}

fn next_chi_accumulator<F>(length: usize, previous_accumulator: F, i: usize, alpha: F) -> F
where
    F: One + Zero + Mul<Output = F> + Sub<Output = F> + Copy,
{
    if (length >> i) & 1 == 0 {
        previous_accumulator * (F::one() - alpha)
    } else {
        F::one() - (F::one() - previous_accumulator) * alpha
    }
}

fn next_rho_accumulator<F>(
    length: usize,
    previous_accumulator: F,
    i: usize,
    alpha: F,
    previous_chi: F,
    rho_of_power_of_two: F,
    power_of_two: F,
) -> F
where
    F: One + Zero + Mul<Output = F> + Sub<Output = F> + Copy,
{
    if (length >> i) & 1 == 0 {
        previous_accumulator * (F::one() - alpha)
    } else {
        (F::one() - alpha) * rho_of_power_of_two
            + alpha * (previous_accumulator + power_of_two * previous_chi)
    }
}

/// Given the point `point` (or `a`) with length nu, we can evaluate the lagrange basis of length 2^nu at that point.
/// This is what [`super::compute_evaluation_vector`] does.
///
/// NOTE: if length is greater than 2^nu, this function will pad `point` with 0s, which
/// will result in padding the basis with 0s.
///
/// Call the resulting evaluation vector A. This function computes `sum A[i] for i in 0..length`. That is:
/// ```text
/// (1-a[0])(1-a[1])...(1-a[nu-1]) +
/// (a[0])(1-a[1])...(1-a[nu-1]) +
/// (1-a[0])(a[1])...(1-a[nu-1]) +
/// (a[0])(a[1])...(1-a[nu-1]) + ...
/// ```
pub fn compute_truncated_lagrange_basis_sum<F>(length: usize, point: &[F]) -> F
where
    F: One + Zero + Mul<Output = F> + Sub<Output = F> + Copy,
{
    if length >= 1 << point.len() {
        F::one()
    } else {
        point
            .iter()
            .enumerate()
            .fold(F::zero(), |chi, (i, &alpha)| {
                next_chi_accumulator(length, chi, i, alpha)
            })
    }
}

pub fn compute_rho_eval<F>(length: usize, point: &[F]) -> F
where
    F: One + Zero + Mul<Output = F> + Sub<Output = F> + Copy,
{
    if length == 1 << point.len() {
        let res = point.iter().fold(
            (F::zero(), F::one()),
            |(acc, current_power_of_two), &alpha| {
                (
                    acc + current_power_of_two * alpha,
                    current_power_of_two + current_power_of_two,
                )
            },
        );
        res.0
    } else {
        let (rho, _, _, _) = point.iter().enumerate().fold(
            (F::zero(), F::zero(), F::zero(), F::one()),
            |(previous_rho, previous_chi, current_rho_of_power_of_two, current_power_of_two),
             (i, &alpha)| {
                let next_rho_of_power_of_two =
                    current_power_of_two * alpha + current_rho_of_power_of_two;
                let next_power_of_two = current_power_of_two + current_power_of_two;
                (
                    next_rho_accumulator(
                        length,
                        previous_rho,
                        i,
                        alpha,
                        previous_chi,
                        current_rho_of_power_of_two,
                        current_power_of_two,
                    ),
                    next_chi_accumulator(length, previous_chi, i, alpha),
                    next_rho_of_power_of_two,
                    next_power_of_two,
                )
            },
        );
        rho
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_rho_eval, compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum};

    #[test]
    fn lagrange_sum_full_length_returns_one() {
        // For length == 2^nu the sum is always 1
        let sum = compute_truncated_lagrange_basis_sum(4, &[0.5f64, 0.5]);
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_length_one_with_empty_point_is_one() {
        let sum = compute_truncated_lagrange_basis_sum(1, &[] as &[f64]);
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_length_zero_is_zero() {
        let sum = compute_truncated_lagrange_basis_sum(0, &[0.5f64]);
        assert!((sum - 0.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_length_two_half_point_is_one() {
        // Full 2^1 = 2 elements, both contribute
        let sum = compute_truncated_lagrange_basis_sum(2, &[0.5f64]);
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_length_one_with_point_zero_is_one() {
        // At point 0: basis[0] = 1-0 = 1, so sum of first 1 = 1
        let sum = compute_truncated_lagrange_basis_sum(1, &[0.0f64]);
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_length_one_with_point_one_is_zero() {
        // At point 1: basis[0] = 1-1 = 0, so sum of first 1 = 0
        let sum = compute_truncated_lagrange_basis_sum(1, &[1.0f64]);
        assert!((sum - 0.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_length_exceeding_full_returns_one() {
        // length >= 2^nu should return 1
        let sum = compute_truncated_lagrange_basis_sum(8, &[0.3f64, 0.7]);
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_sum_three_point_full_is_one() {
        let sum = compute_truncated_lagrange_basis_sum(8, &[0.1f64, 0.5, 0.9]);
        assert!((sum - 1.0).abs() < 1e-10, "sum = {sum}");
    }

    #[test]
    fn lagrange_inner_product_zero_length_is_zero() {
        let ip = compute_truncated_lagrange_basis_inner_product(0, &[0.5f64], &[0.5f64]);
        assert!((ip - 0.0).abs() < 1e-10, "ip = {ip}");
    }

    #[test]
    fn lagrange_inner_product_full_length_equal_points_is_one() {
        // With a == b and full length, inner product = sum A[i]^2 = 1 (for evaluation vector)
        // Actually, for unit point [1,0,..], evaluaton vector is indicator, so A[i]*B[i] = A[i]
        // but for equal points with full length: inner product = sum of products of basis evals
        // For a = b, sum A[i]^2. At [0.5, 0.5], each A[i] = 0.25, and 4 * 0.25^2 = 0.25
        let ip = compute_truncated_lagrange_basis_inner_product(4, &[0.5f64, 0.5], &[0.5f64, 0.5]);
        // sum = 4 * (0.25)^2 = 4 * 0.0625 = 0.25
        assert!((ip - 0.25).abs() < 1e-10, "ip = {ip}");
    }

    #[test]
    fn lagrange_inner_product_full_length_orthogonal_points() {
        // At point a=[1], basis A = [0, 1]; at point b=[0], basis B = [1, 0]
        // inner product = 0*1 + 1*0 = 0
        let ip = compute_truncated_lagrange_basis_inner_product(2, &[1.0f64], &[0.0f64]);
        assert!((ip - 0.0).abs() < 1e-10, "ip = {ip}");
    }

    #[test]
    fn lagrange_inner_product_full_length_same_indicator_is_product() {
        // At point a=b=[1,0], basis A = B = [0, 1, 0, 0] (indicator of index 1)
        // inner product = 0^2 + 1^2 + 0^2 + 0^2 = 1
        let ip = compute_truncated_lagrange_basis_inner_product(4, &[1.0f64, 0.0], &[1.0f64, 0.0]);
        assert!((ip - 1.0).abs() < 1e-10, "ip = {ip}");
    }

    #[test]
    fn lagrange_inner_product_empty_points_length_one() {
        // With empty points, nu=0, full length is 1
        // A = [1], B = [1], inner product of length 1 = 1
        let ip = compute_truncated_lagrange_basis_inner_product(1, &[] as &[f64], &[] as &[f64]);
        assert!((ip - 1.0).abs() < 1e-10, "ip = {ip}");
    }

    #[test]
    fn rho_eval_length_one_with_point_zero() {
        // length = 1, point = [0]: rho = sum_{i=0}^{0} i * A[i] = 0 * 1 = 0
        let rho = compute_rho_eval(1, &[0.0f64]);
        assert!((rho - 0.0).abs() < 1e-10, "rho = {rho}");
    }

    #[test]
    fn rho_eval_full_length_with_standard_point() {
        // length = 2^nu case
        // For length=2, point=[0.5]: rho = sum_{i=0}^{1} i * A[i] = 0*(0.5) + 1*(0.5) = 0.5
        let rho = compute_rho_eval(2, &[0.5f64]);
        assert!((rho - 0.5).abs() < 1e-10, "rho = {rho}");
    }

    #[test]
    fn rho_eval_length_two_full_with_point_one() {
        // length=2, point=[1.0]: A=[0,1], rho = 0*0 + 1*1 = 1
        let rho = compute_rho_eval(2, &[1.0f64]);
        assert!((rho - 1.0).abs() < 1e-10, "rho = {rho}");
    }

    #[test]
    fn rho_eval_length_two_full_with_point_zero() {
        // length=2, point=[0.0]: A=[1,0], rho = 0*1 + 1*0 = 0
        let rho = compute_rho_eval(2, &[0.0f64]);
        assert!((rho - 0.0).abs() < 1e-10, "rho = {rho}");
    }
}
