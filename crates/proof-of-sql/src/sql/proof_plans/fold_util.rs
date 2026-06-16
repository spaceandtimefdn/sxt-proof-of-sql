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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_fold_vals_empty() {
        let result: TestScalar = fold_vals(TestScalar::from(2u64), &[]);
        assert_eq!(result, TestScalar::ZERO);
    }

    #[test]
    fn test_fold_vals_single() {
        // fold_vals(beta, [v]) = v
        let beta = TestScalar::from(3u64);
        let vals = [TestScalar::from(7u64)];
        let result = fold_vals(beta, &vals);
        assert_eq!(result, TestScalar::from(7u64));
    }

    #[test]
    fn test_fold_vals_two_elements() {
        // fold_vals(beta, [a, b]) = a * beta + b
        let beta = TestScalar::from(2u64);
        let a = TestScalar::from(3u64);
        let b = TestScalar::from(5u64);
        let expected = a * beta + b; // 3*2 + 5 = 11
        let result = fold_vals(beta, &[a, b]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fold_vals_three_elements() {
        // fold_vals(beta, [a, b, c]) = (a * beta + b) * beta + c
        let beta = TestScalar::from(2u64);
        let a = TestScalar::from(1u64);
        let b = TestScalar::from(2u64);
        let c = TestScalar::from(3u64);
        // (1*2 + 2)*2 + 3 = 4*2 + 3 = 11
        let expected = (a * beta + b) * beta + c;
        let result = fold_vals(beta, &[a, b, c]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fold_vals_zero_beta() {
        // fold_vals(0, [a, b, c]) = c (all but the last term vanishes)
        let beta = TestScalar::ZERO;
        let vals = [
            TestScalar::from(10u64),
            TestScalar::from(20u64),
            TestScalar::from(30u64),
        ];
        let result = fold_vals(beta, &vals);
        assert_eq!(result, TestScalar::from(30u64));
    }

    #[test]
    fn test_fold_columns_empty_columns() {
        // With zero columns nothing is added to res
        let mut res = vec![TestScalar::from(5u64), TestScalar::from(7u64)];
        let cols: &[&[TestScalar]] = &[];
        fold_columns::<TestScalar>(&mut res, TestScalar::ONE, TestScalar::from(2u64), cols);
        assert_eq!(res, vec![TestScalar::from(5u64), TestScalar::from(7u64)]);
    }
}
