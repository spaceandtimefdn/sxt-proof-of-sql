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
    use ark_std::Zero;
    use alloc::vec;

    #[test]
    fn fold_vals_evaluates_values_as_beta_polynomial() {
        let beta = TestScalar::from(2_u64);
        let vals = [
            TestScalar::from(1_u64),
            TestScalar::from(2_u64),
            TestScalar::from(3_u64),
        ];

        assert_eq!(fold_vals(beta, &vals), TestScalar::from(11_u64));
        assert_eq!(fold_vals(beta, &[]), TestScalar::zero());
    }

    #[test]
    fn fold_columns_accumulates_reversed_beta_powers() {
        let mut res = vec![TestScalar::from(10_u64); 3];
        let first = [1_u64, 2, 3];
        let second = [4_u64, 5, 6];
        let third = [7_u64, 8, 9];
        let columns: [&[u64]; 3] = [&first, &second, &third];

        fold_columns(
            &mut res,
            TestScalar::from(3_u64),
            TestScalar::from(2_u64),
            &columns,
        );

        assert_eq!(
            res,
            vec![
                TestScalar::from(67_u64),
                TestScalar::from(88_u64),
                TestScalar::from(109_u64)
            ]
        );
    }
}
