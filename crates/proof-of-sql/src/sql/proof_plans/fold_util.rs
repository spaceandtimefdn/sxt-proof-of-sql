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
    use super::fold_columns;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn we_leave_fold_result_unchanged_when_there_are_no_columns() {
        let mut result = [1_u32, 2, 3].map(TestScalar::from);
        let original = result;
        let columns: [&[TestScalar]; 0] = [];

        fold_columns(
            &mut result,
            TestScalar::from(5_u32),
            TestScalar::from(7_u32),
            &columns,
        );

        assert_eq!(result, original);
    }
}
