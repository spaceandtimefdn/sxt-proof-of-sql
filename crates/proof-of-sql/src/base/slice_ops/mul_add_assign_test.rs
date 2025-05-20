use super::*;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn test_mul_add_assign() {
    let mut a = vec![
        TestScalar::from(1i32),
        TestScalar::from(2i32),
        TestScalar::from(3i32),
        TestScalar::from(4i32),
    ];
    let b = vec![2, 3, 4, 5];
    mul_add_assign(&mut a, TestScalar::from(10i32), &b);
    let c = vec![
        TestScalar::from(1 + 10 * 2),
        TestScalar::from(2 + 10 * 3),
        TestScalar::from(3 + 10 * 4),
        TestScalar::from(4 + 10 * 5),
    ];
    assert_eq!(a, c);
}

/// test [`mul_add_assign`] with uneven vectors
#[test]
fn test_mul_add_assign_uneven() {
    let mut a = vec![
        TestScalar::from(1u32),
        TestScalar::from(2u32),
        TestScalar::from(3u32),
        TestScalar::from(4u32),
        TestScalar::from(5u32),
    ];
    let b = vec![
        TestScalar::from(2u32),
        TestScalar::from(3u32),
        TestScalar::from(4u32),
        TestScalar::from(5u32),
    ];
    mul_add_assign(&mut a, TestScalar::from(10u32), &b);
    let c = vec![
        TestScalar::from(1 + 10 * 2),
        TestScalar::from(2 + 10 * 3),
        TestScalar::from(3 + 10 * 4),
        TestScalar::from(4 + 10 * 5),
        TestScalar::from(5u32),
    ];
    assert_eq!(a, c);
}

/// test [`mul_add_assign`] with with uneven panics when len(a) < len(b)
#[test]
#[should_panic(
    expected = "The length of result must be greater than or equal to the length of the vector of values to be multiplied and added"
)]
fn test_mul_add_assign_uneven_panic() {
    let mut a = vec![
        TestScalar::from(1u32),
        TestScalar::from(2u32),
        TestScalar::from(3u32),
        TestScalar::from(4u32),
    ];
    let b = vec![2, 3, 4, 5, 6];
    mul_add_assign(&mut a, TestScalar::from(10u32), &b);
}

/// test [`mul_add_assign`] with `TestScalar`
#[test]
fn test_mul_add_assign_testscalar() {
    let mut a = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b = vec![TestScalar::from(2u64), TestScalar::from(3u64)];
    mul_add_assign(&mut a, TestScalar::from(10u64), &b);
    let c = vec![
        TestScalar::from(1u64) + TestScalar::from(10u64) * TestScalar::from(2u64),
        TestScalar::from(2u64) + TestScalar::from(10u64) * TestScalar::from(3u64),
    ];
    assert_eq!(a, c);
}

/// test [`mul_add_assign`] with uneven `TestScalar`
#[test]
fn test_mul_add_assign_testscalar_uneven() {
    let mut a = vec![
        TestScalar::from(1u64),
        TestScalar::from(2u64),
        TestScalar::from(3u64),
    ];
    let b = vec![TestScalar::from(2u64), TestScalar::from(3u64)];
    mul_add_assign(&mut a, TestScalar::from(10u64), &b);
    let c = vec![
        TestScalar::from(1u64) + TestScalar::from(10u64) * TestScalar::from(2u64),
        TestScalar::from(2u64) + TestScalar::from(10u64) * TestScalar::from(3u64),
        TestScalar::from(3u64),
    ];
    assert_eq!(a, c);
}
