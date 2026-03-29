use super::{
    compute_rho_eval, compute_truncated_lagrange_basis_inner_product,
    compute_truncated_lagrange_basis_sum,
};
use crate::base::scalar::test_scalar::TestScalar;
use num_traits::{One, Zero};

/// Tests for `compute_truncated_lagrange_basis_sum`
#[test]
fn test_truncated_lagrange_basis_sum_single_point() {
    // With a single point, the sum over all Lagrange basis evaluations should be 1
    let point: Vec<TestScalar> = vec![];
    let sum = compute_truncated_lagrange_basis_sum(1, &point);
    assert_eq!(sum, TestScalar::one());
}

#[test]
fn test_truncated_lagrange_basis_sum_two_variables_full() {
    // With 2 variables and length=4, sum equals 1
    let point: Vec<TestScalar> = vec![TestScalar::zero(), TestScalar::zero()];
    let sum = compute_truncated_lagrange_basis_sum(4, &point);
    assert_eq!(sum, TestScalar::one());
}

#[test]
fn test_truncated_lagrange_basis_sum_partial_length() {
    // With partial length (not filling all 2^n), the sum should still be a valid scalar
    let point: Vec<TestScalar> = vec![TestScalar::zero(), TestScalar::zero()];
    let sum = compute_truncated_lagrange_basis_sum(2, &point);
    // partial sum should not panic and should equal (1 - point[1]) = 1 for zero point
    assert_eq!(sum, TestScalar::one());
}

#[test]
fn test_truncated_lagrange_basis_sum_length_zero() {
    let point: Vec<TestScalar> = vec![TestScalar::zero()];
    let sum = compute_truncated_lagrange_basis_sum(0, &point);
    assert_eq!(sum, TestScalar::zero());
}

/// Tests for `compute_rho_eval`
#[test]
fn test_rho_eval_full_length_is_zero() {
    // When length equals 2^n, rho should be zero (all Lagrange terms used)
    let point: Vec<TestScalar> = vec![TestScalar::zero()];
    // length = 2 = 2^1, so rho should be zero
    let rho = compute_rho_eval(2, &point);
    assert_eq!(rho, TestScalar::zero());
}

#[test]
fn test_rho_eval_partial_length() {
    // When length is partial, rho is non-zero
    let point: Vec<TestScalar> = vec![TestScalar::zero()];
    // length = 1 < 2^1 = 2
    let rho = compute_rho_eval(1, &point);
    // rho should equal (1 - point[0]) = 1 for zero point
    assert_eq!(rho, TestScalar::one());
}

#[test]
fn test_rho_eval_zero_length() {
    let point: Vec<TestScalar> = vec![TestScalar::zero()];
    let rho = compute_rho_eval(0, &point);
    assert_eq!(rho, TestScalar::one());
}

/// Tests for `compute_truncated_lagrange_basis_inner_product`
#[test]
fn test_inner_product_matches_expected_for_zero_point() {
    // For a zero point evaluating the inner product against unit scalars
    let a: Vec<TestScalar> = vec![TestScalar::one(), TestScalar::one()];
    let b: Vec<TestScalar> = vec![TestScalar::one(), TestScalar::zero()];
    let point: Vec<TestScalar> = vec![TestScalar::zero()];
    // L_0(0) = 1, L_1(0) = 0 → inner product with a=b=1 is 1*1 + 0*1 = 1
    // With point=[0], the full Lagrange basis L_0 = (1-0) = 1, L_1 = 0
    // inner product of a and b against Lagrange:
    // sum_i a[i] * b[i] style — the function computes sum_i chi_i(point) * a[i] * b[i]
    let ip = compute_truncated_lagrange_basis_inner_product(&a, &b, &point);
    // L_0(point=[0]) = 1, L_1(point=[0]) = 0
    // result = L_0*a[0]*b[0] + L_1*a[1]*b[1] = 1*1*1 + 0*1*0 = 1
    assert_eq!(ip, TestScalar::one());
}

#[test]
fn test_inner_product_empty_slices() {
    let a: Vec<TestScalar> = vec![];
    let b: Vec<TestScalar> = vec![];
    let point: Vec<TestScalar> = vec![];
    let ip = compute_truncated_lagrange_basis_inner_product(&a, &b, &point);
    assert_eq!(ip, TestScalar::zero());
}

#[test]
fn test_inner_product_one_element() {
    let a: Vec<TestScalar> = vec![TestScalar::one()];
    let b: Vec<TestScalar> = vec![TestScalar::one()];
    let point: Vec<TestScalar> = vec![];
    // With empty point, only L_0 = 1
    let ip = compute_truncated_lagrange_basis_inner_product(&a, &b, &point);
    assert_eq!(ip, TestScalar::one());
}

#[test]
fn test_truncated_lagrange_basis_sum_all_variables_one() {
    // With point = [1], sum over length=1 should reflect L_0(1) = 1 - 1 = 0
    let point: Vec<TestScalar> = vec![TestScalar::one()];
    let sum = compute_truncated_lagrange_basis_sum(1, &point);
    assert_eq!(sum, TestScalar::zero());
}
