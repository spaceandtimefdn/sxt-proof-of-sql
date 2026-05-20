use super::*;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn test_mul_add_assign() {
    let mut a = [1, 2, 3, 4].map(TestScalar::from).to_vec();
    let b = vec![2, 3, 4, 5];
    mul_add_assign(&mut a, TestScalar::from(10i32), &b);
    let c = [1 + 10 * 2, 2 + 10 * 3, 3 + 10 * 4, 4 + 10 * 5]
        .map(TestScalar::from)
        .to_vec();
    assert_eq!(a, c);
}

#[test]
fn test_mul_add_assign_empty_input_keeps_result_unchanged() {
    let mut a = [1, 2, 3].map(TestScalar::from).to_vec();
    let b: Vec<i32> = Vec::new();
    let expected = a.clone();
    mul_add_assign(&mut a, TestScalar::from(10i32), &b);
    assert_eq!(a, expected);
}

#[test]
fn test_mul_add_assign_zero_multiplier_keeps_result_unchanged() {
    let mut a = [1, 2, 3].map(TestScalar::from).to_vec();
    let b = vec![10, 20, 30];
    let expected = a.clone();
    mul_add_assign(&mut a, TestScalar::from(0i32), &b);
    assert_eq!(a, expected);
}

/// test [`mul_add_assign`] with uneven vectors
#[test]
fn test_mul_add_assign_uneven() {
    let mut a = [1, 2, 3, 4, 5].map(TestScalar::from).to_vec();
    let b = [2, 3, 4, 5].map(TestScalar::from).to_vec();
    mul_add_assign(&mut a, TestScalar::from(10u32), &b);
    let c = [1 + 10 * 2, 2 + 10 * 3, 3 + 10 * 4, 4 + 10 * 5, 5]
        .map(TestScalar::from)
        .to_vec();
    assert_eq!(a, c);
}

/// test [`mul_add_assign`] with with uneven panics when len(a) < len(b)
#[test]
#[should_panic(
    expected = "The length of result must be greater than or equal to the length of the vector of values to be multiplied and added"
)]
fn test_mul_add_assign_uneven_panic() {
    let mut a = [1u32, 2u32, 3u32, 4u32].map(TestScalar::from).to_vec();
    let b = vec![2, 3, 4, 5, 6];
    mul_add_assign(&mut a, TestScalar::from(10u32), &b);
}

/// test [`mul_add_assign`] with `TestScalar`
#[test]
fn test_mul_add_assign_testscalar() {
    let mut a = [1, 2].map(TestScalar::from).to_vec();
    let b = [2, 3].map(TestScalar::from).to_vec();
    mul_add_assign(&mut a, TestScalar::from(10u64), &b);
    let c = [1 + 10 * 2, 2 + 10 * 3].map(TestScalar::from).to_vec();
    assert_eq!(a, c);
}

/// test [`mul_add_assign`] with uneven `TestScalar`
#[test]
fn test_mul_add_assign_testscalar_uneven() {
    let mut a = [1, 2, 3].map(TestScalar::from).to_vec();
    let b = [2, 3].map(TestScalar::from).to_vec();
    mul_add_assign(&mut a, TestScalar::from(10u64), &b);
    let c = [1 + 10 * 2, 2 + 10 * 3, 3].map(TestScalar::from).to_vec();
    assert_eq!(a, c);
}
