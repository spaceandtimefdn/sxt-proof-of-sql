use crate::base::{polynomial::MultilinearExtension, scalar::Scalar};

/// This function takes a set of columns and fold it into a slice of scalars.
///
/// The result `res` is updated with
/// `res[i] += mul * sum (beta^(n-j-1) * columns[j][i]) for j in 0..columns.len()` where n is the number of columns.
/// where each column is padded with 0s as needed.
///
/// This is similar to adding `mul * fold_vals(beta,...)` on each row.
#[tracing::instrument(name = "FoldUtil::fold_columns", level = "debug", skip_all)]
pub fn fold_columns<S: Scalar>(
    res: &mut [S],
    mul: S,
    beta: S,
    columns: &[impl MultilinearExtension<S>],
) {
    for (m, col) in powers(mul, beta).zip(columns.iter().rev()) {
        col.mul_add(res, &m);
    }
}

/// This function takes a set of values and returns a scalar that is the
/// result of folding the values.
///
/// The result is
/// `sum (beta^(n-j-1) * vals[j]) for j in 0..vals.len()` where n is the number of vals.
pub fn fold_vals<S: Scalar>(beta: S, vals: &[S]) -> S {
    vals.iter().fold(S::zero(), |acc, &v| acc * beta + v)
}

/// Returns an iterator for the lazily evaluated sequence `init, init * base, init * base^2, ...`
fn powers<S: Scalar>(init: S, base: S) -> impl Iterator<Item = S> {
    core::iter::successors(Some(init), move |&m| Some(m * base))
}

#[cfg(test)]
mod tests {
    use super::{fold_vals, powers};
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn fold_vals_empty_slice_returns_zero() {
        let result = fold_vals::<TestScalar>(TestScalar::from(2), &[]);
        assert_eq!(result, TestScalar::from(0));
    }

    #[test]
    fn fold_vals_single_element_returns_element_unchanged() {
        let result = fold_vals::<TestScalar>(TestScalar::from(5), &[TestScalar::from(7)]);
        assert_eq!(result, TestScalar::from(7));
    }

    #[test]
    fn fold_vals_two_elements_applies_horner_scheme() {
        // beta=2, vals=[3, 1]: 3*2 + 1 = 7
        let result = fold_vals::<TestScalar>(
            TestScalar::from(2),
            &[TestScalar::from(3), TestScalar::from(1)],
        );
        assert_eq!(result, TestScalar::from(7));
    }

    #[test]
    fn fold_vals_three_elements_computes_correctly() {
        // beta=2, vals=[1, 2, 3]: 1*4 + 2*2 + 3 = 11
        let result = fold_vals::<TestScalar>(
            TestScalar::from(2),
            &[TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)],
        );
        assert_eq!(result, TestScalar::from(11));
    }

    #[test]
    fn fold_vals_beta_zero_returns_last_element() {
        // beta=0, vals=[5, 99]: 5*0 + 99 = 99
        let result = fold_vals::<TestScalar>(
            TestScalar::from(0),
            &[TestScalar::from(5), TestScalar::from(99)],
        );
        assert_eq!(result, TestScalar::from(99));
    }

    #[test]
    fn fold_vals_beta_one_sums_all_values() {
        // beta=1, vals=[1, 2, 3]: 1 + 2 + 3 = 6
        let result = fold_vals::<TestScalar>(
            TestScalar::from(1),
            &[TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)],
        );
        assert_eq!(result, TestScalar::from(6));
    }

    #[test]
    fn powers_produces_geometric_sequence() {
        let vals: Vec<TestScalar> = powers(TestScalar::from(1), TestScalar::from(3))
            .take(4)
            .collect();
        assert_eq!(
            vals,
            vec![
                TestScalar::from(1),
                TestScalar::from(3),
                TestScalar::from(9),
                TestScalar::from(27),
            ]
        );
    }

    #[test]
    fn powers_with_init_two_and_base_two() {
        let vals: Vec<TestScalar> = powers(TestScalar::from(2), TestScalar::from(2))
            .take(4)
            .collect();
        assert_eq!(
            vals,
            vec![
                TestScalar::from(2),
                TestScalar::from(4),
                TestScalar::from(8),
                TestScalar::from(16),
            ]
        );
    }
}
