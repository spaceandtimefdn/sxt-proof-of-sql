use crate::base::{
    polynomial::{
        compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum,
    },
    scalar::Curve25519Scalar,
};

/// Tests for `compute_truncated_lagrange_basis_sum`
#[test]
fn test_truncated_lagrange_basis_sum_length_one() {
    // With nu=0 the truncated basis has 1 term; evaluating at any point
    // the sum should equal 1 (the single basis poly evaluated at that point).
    let point: Vec<Curve25519Scalar> = vec![];
    let result = compute_truncated_lagrange_basis_sum(1, &point);
    assert_eq!(result, Curve25519Scalar::from(1u64));
}

#[test]
fn test_truncated_lagrange_basis_sum_length_two() {
    // With nu=1, two basis polynomials, point = [r].
    // L_0(r) + L_1(r) = (1-r) + r = 1.
    let r = Curve25519Scalar::from(3u64);
    let point = vec![r];
    let result = compute_truncated_lagrange_basis_sum(2, &point);
    assert_eq!(result, Curve25519Scalar::from(1u64));
}

#[test]
fn test_truncated_lagrange_basis_sum_length_four() {
    // With nu=2 and length=4, all 4 basis polynomials sum to 1.
    let r0 = Curve25519Scalar::from(5u64);
    let r1 = Curve25519Scalar::from(7u64);
    let point = vec![r0, r1];
    let result = compute_truncated_lagrange_basis_sum(4, &point);
    assert_eq!(result, Curve25519Scalar::from(1u64));
}

#[test]
fn test_truncated_lagrange_basis_sum_truncated() {
    // length=3 with nu=2: only 3 of 4 basis polynomials contribute.
    // sum = L_0 + L_1 + L_2 = (1-r1)(1-r0) + (1-r1)*r0 + r1*(1-r0)
    //     = (1-r1) + r1*(1-r0)
    //     = 1 - r1*r0
    let r0 = Curve25519Scalar::from(2u64);
    let r1 = Curve25519Scalar::from(3u64);
    let point = vec![r0, r1];
    let result = compute_truncated_lagrange_basis_sum(3, &point);
    let expected = Curve25519Scalar::from(1u64)
        - r0 * r1;
    assert_eq!(result, expected);
}

/// Tests for `compute_truncated_lagrange_basis_inner_product`
#[test]
fn test_truncated_lagrange_basis_inner_product_length_one() {
    // <a, L> with length=1: only L_0 = 1, inner product = a[0].
    let a = vec![Curve25519Scalar::from(42u64)];
    let point: Vec<Curve25519Scalar> = vec![];
    let result = compute_truncated_lagrange_basis_inner_product(&a, &point);
    assert_eq!(result, Curve25519Scalar::from(42u64));
}

#[test]
fn test_truncated_lagrange_basis_inner_product_length_two() {
    // <a, L> with length=2, point=[r]:
    // a[0]*L_0(r) + a[1]*L_1(r) = a[0]*(1-r) + a[1]*r
    let a = vec![
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(7u64),
    ];
    let r = Curve25519Scalar::from(2u64);
    let point = vec![r];
    let result = compute_truncated_lagrange_basis_inner_product(&a, &point);
    // 3*(1-2) + 7*2 = 3*(-1) + 14 = -3 + 14 = 11
    let expected = Curve25519Scalar::from(3u64)
        * (Curve25519Scalar::from(1u64) - r)
        + Curve25519Scalar::from(7u64) * r;
    assert_eq!(result, expected);
}

#[test]
fn test_truncated_lagrange_basis_inner_product_all_zeros() {
    let a = vec![
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
        Curve25519Scalar::from(0u64),
    ];
    let r0 = Curve25519Scalar::from(5u64);
    let r1 = Curve25519Scalar::from(9u64);
    let point = vec![r0, r1];
    let result = compute_truncated_lagrange_basis_inner_product(&a, &point);
    assert_eq!(result, Curve25519Scalar::from(0u64));
}
