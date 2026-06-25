use alloc::{vec, vec::Vec};
/*
 * Adapted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use core::ops::{Add, AddAssign, Mul, MulAssign, SubAssign};
use core::{cmp::PartialEq, iter::Product};
use num_traits::{Inv, One, Zero};

/// Interpolate a uni-variate degree-`polynomial.len()-1` polynomial and evaluate this
/// polynomial at `x`:
/// For any polynomial, `f(x)`, with degree less than or equal to `d`, we have that:
/// `f(x) = sum_{i=0}^{d} (-1)^(d-i) * (f(i) / (i! * (d-i)! * (x-i))) * prod_{i=0}^{d} (x-i)`
// Allow missing panics documentation because the function should not panic under normal conditions.
/// unless x is one of 0,1,...,d, in which case, f(x) is already known.
#[expect(clippy::missing_panics_doc)]
#[cfg_attr(not(test), expect(dead_code))]
pub fn interpolate_uni_poly<F>(polynomial: &[F], x: F) -> F
where
    F: Copy
        + Inv<Output = Option<F>>
        + One
        + Zero
        + AddAssign
        + Mul<Output = F>
        + MulAssign
        + SubAssign
        + PartialEq,
{
    if polynomial.is_empty() {
        return F::zero();
    }
    let degree = polynomial.len() - 1;

    // Construct a vector of factorials, where `factorials[i] = i!`.
    let mut factorials: Vec<F> = Vec::with_capacity(degree + 1);
    let mut factorial = F::one();
    let mut i = F::zero();
    for eval in polynomial {
        factorials.push(factorial);
        if i == x {
            return *eval;
        }
        i += F::one();
        factorial *= i;
    }

    // This will become `sum_{i=0}^{d} (-1)^(d-i) * (f(i) / (i! * (d-i)! * (x-i)))`.
    let mut sum = F::zero();
    // This will become `prod_{i=0}^{d} (x-i)`.
    let mut product = F::one();
    // This will be `x-i`.
    let mut x_minus_i = x;
    for i in 0..=degree {
        // This is `f(i) / (i! * (d-i)! * (x-i))`
        let new_term = polynomial[i]
            * (factorials[i] * factorials[degree - i] * x_minus_i)
                .inv()
                .expect(
                    "Inverse computation failed unexpectedly. This should not happen as `x != i`.",
                );

        // This handles the (-1)^(d-i) sign.
        if (degree - i).is_multiple_of(2) {
            sum += new_term;
        } else {
            sum -= new_term;
        }
        product *= x_minus_i;
        x_minus_i -= F::one();
    }
    sum * product
}

/// Let `d` be `evals.len() - 1` and let `f` be the polynomial such that `f(i) = evals[i]`.
/// The output of this function is the vector of coefficients of `f`, with the leading coefficient first.
/// That is, `f(x) = evals[j] * x^(d - j)`.
#[expect(
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
// This function is guaranteed not to panic because:
// - The product in `inv()` will never be zero, as the numbers being multiplied are all non-zero by construction.
// - If there are no elements to reduce, `unwrap_or(vec![])` provides an empty vector as a safe fallback.
pub fn interpolate_evaluations_to_reverse_coefficients<S>(evals: &[S]) -> Vec<S>
where
    S: Zero
        + Copy
        + From<i32>
        + Mul<Output = S>
        + Add<Output = S>
        + Inv<Output = Option<S>>
        + Product,
{
    assert!(i32::try_from(evals.len()).is_ok());
    let n = evals.len().max(1) - 1;
    evals
        .iter()
        .enumerate()
        .map(|(idx, &eval_i)| {
            let i = idx as i32;
            let mut scaled_lagrange_basis = vec![S::zero(); n + 1];
            // First compute the constant factor of this lagrange basis polynomial:
            scaled_lagrange_basis[0] = (i - n as i32..0)
                .chain(1..=i)
                .map(S::from)
                .product::<S>()
                .inv()
                .expect("Product will never be zero because the terms being multiplied are non-zero by construction.")
                * eval_i;
            // Then multiply by the appropriate linear terms:
            // for j in 0..=n if j != i {
            for neg_j in (-(n as i32)..-i).chain(1 - i..=0).map(S::from) {
                for k in (0..n).rev() {
                    scaled_lagrange_basis[k + 1] =
                        scaled_lagrange_basis[k + 1] + neg_j * scaled_lagrange_basis[k];
                }
            }
            scaled_lagrange_basis
        })
        // Finally, sum up all the resulting polynomials
        .reduce(|mut acc, b| {
            acc.iter_mut().zip(b).for_each(|(a, b)| *a = *a + b);
            acc
        })
        .unwrap_or(vec![])
}

#[cfg(test)]
mod tests {
    use super::{interpolate_evaluations_to_reverse_coefficients, interpolate_uni_poly};
    use crate::base::scalar::test_scalar::TestScalar;

    fn ts(n: i32) -> TestScalar {
        TestScalar::from(n)
    }

    #[test]
    fn empty_polynomial_evaluates_to_zero() {
        let result = interpolate_uni_poly::<TestScalar>(&[], ts(5));
        assert_eq!(result, ts(0));
    }

    #[test]
    fn constant_polynomial_evaluates_to_constant() {
        // f(x) = 3 at x=0, so f(7) = 3
        let result = interpolate_uni_poly::<TestScalar>(&[ts(3)], ts(7));
        assert_eq!(result, ts(3));
    }

    #[test]
    fn linear_polynomial_evaluates_correctly() {
        // f(0)=0, f(1)=1 => f(x) = x => f(2) = 2
        let result = interpolate_uni_poly::<TestScalar>(&[ts(0), ts(1)], ts(2));
        assert_eq!(result, ts(2));
    }

    #[test]
    fn evaluating_at_known_point_returns_exact_value() {
        // f(0)=5, f(1)=7 => f(0) = 5 exactly (short-circuit)
        let result = interpolate_uni_poly::<TestScalar>(&[ts(5), ts(7)], ts(0));
        assert_eq!(result, ts(5));
    }

    #[test]
    fn reverse_coefficients_empty_returns_empty() {
        let result = interpolate_evaluations_to_reverse_coefficients::<TestScalar>(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn reverse_coefficients_single_element_constant() {
        // f(x) = c => coefficients [c]
        let result = interpolate_evaluations_to_reverse_coefficients::<TestScalar>(&[ts(4)]);
        assert_eq!(result, alloc::vec![ts(4)]);
    }
}
