#[cfg(test)]
mod tests {
    use crate::base::{
        polynomial::{
            compute_truncated_lagrange_basis_inner_product,
            compute_truncated_lagrange_basis_sum,
        },
        scalar::Curve25519Scalar,
    };

    /// Helper: build a point vector of length `n` from a slice of i64 values.
    fn pt<const N: usize>(vals: [i64; N]) -> Vec<Curve25519Scalar> {
        vals.iter().map(|&v| Curve25519Scalar::from(v)).collect()
    }

    // -----------------------------------------------------------------------
    // compute_truncated_lagrange_basis_sum
    // -----------------------------------------------------------------------

    #[test]
    fn test_truncated_lagrange_basis_sum_length_zero() {
        // With table_length == 0 the sum over an empty range is zero.
        let point = pt([3, 5]);
        let result = compute_truncated_lagrange_basis_sum(0, &point);
        assert_eq!(result, Curve25519Scalar::from(0_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_sum_length_one() {
        // With table_length == 1 only the L_0 term contributes.
        // L_0(x) = prod_i (1 - x_i), so for point = [0, 0] -> L_0 = 1.
        let point = pt([0, 0]);
        let result = compute_truncated_lagrange_basis_sum(1, &point);
        assert_eq!(result, Curve25519Scalar::from(1_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_sum_length_one_nonzero_point() {
        // For point = [1, 0]: L_0 = (1-1)*(1-0) = 0.
        let point = pt([1, 0]);
        let result = compute_truncated_lagrange_basis_sum(1, &point);
        assert_eq!(result, Curve25519Scalar::from(0_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_sum_full_hypercube() {
        // When table_length == 2^n (all leaves included) the sum of all
        // Lagrange basis polynomials at any point in the hypercube equals 1
        // because the basis forms a partition of unity.
        // We use a 1-dimensional point so 2^1 = 2 entries.
        let point = pt([0]);
        let result = compute_truncated_lagrange_basis_sum(2, &point);
        // L_0(0) + L_1(0) = (1-0) + 0 = 1
        assert_eq!(result, Curve25519Scalar::from(1_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_sum_partial_table() {
        // 1-D point: x = [2] (outside {0,1}).
        // L_0(2) = 1-2 = -1,  L_1(2) = 2.
        // Sum over first 1 entry: L_0(2) = -1.
        let point: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(2_i64)];
        let result = compute_truncated_lagrange_basis_sum(1, &point);
        assert_eq!(result, Curve25519Scalar::from(-1_i64));
    }

    // -----------------------------------------------------------------------
    // compute_truncated_lagrange_basis_inner_product
    // -----------------------------------------------------------------------

    #[test]
    fn test_truncated_lagrange_basis_inner_product_length_zero() {
        let a = pt([3, 1]);
        let b = pt([2, 4]);
        let result = compute_truncated_lagrange_basis_inner_product(0, &a, &b);
        assert_eq!(result, Curve25519Scalar::from(0_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_inner_product_length_one() {
        // <L_0(a), L_0(b)> where a=[0,0], b=[0,0].
        // L_0(a)=1, L_0(b)=1  =>  inner product = 1.
        let a = pt([0, 0]);
        let b = pt([0, 0]);
        let result = compute_truncated_lagrange_basis_inner_product(1, &a, &b);
        assert_eq!(result, Curve25519Scalar::from(1_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_inner_product_orthogonal() {
        // 1-D: a=[1], b=[0].
        // L_0(a)=0, L_1(a)=1; L_0(b)=1, L_1(b)=0.
        // inner product over 2 entries = 0*1 + 1*0 = 0.
        let a: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(1_i64)];
        let b: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(0_i64)];
        let result = compute_truncated_lagrange_basis_inner_product(2, &a, &b);
        assert_eq!(result, Curve25519Scalar::from(0_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_inner_product_same_point() {
        // 1-D: a=b=[1].
        // L_0(1)=0, L_1(1)=1.
        // inner product over 2 entries = 0*0 + 1*1 = 1.
        let p: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(1_i64)];
        let result = compute_truncated_lagrange_basis_inner_product(2, &p, &p);
        assert_eq!(result, Curve25519Scalar::from(1_i64));
    }

    #[test]
    fn test_truncated_lagrange_basis_inner_product_partial_table() {
        // 1-D: a=[2], b=[3], table_length=1 (only L_0 term).
        // L_0(2) = -1,  L_0(3) = -2.
        // inner product = (-1)*(-2) = 2.
        let a: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(2_i64)];
        let b: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(3_i64)];
        let result = compute_truncated_lagrange_basis_inner_product(1, &a, &b);
        assert_eq!(result, Curve25519Scalar::from(2_i64));
    }
}
