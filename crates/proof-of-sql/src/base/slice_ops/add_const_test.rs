use super::*;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn test_add_const() {
    let mut a = vec![1, 2, 3, 4];
    add_const(&mut a, 10);
    let b = vec![1 + 10, 2 + 10, 3 + 10, 4 + 10];
    assert_eq!(a, b);
}

#[test]
fn we_can_add_const_to_empty_slices() {
    let mut a: Vec<TestScalar> = Vec::new();
    add_const(&mut a, TestScalar::from(10u64));
    assert!(a.is_empty());
}

#[test]
fn we_can_add_convertible_const_to_scalar_slices() {
    let mut a = [1, 2, 3, 4].map(TestScalar::from).to_vec();
    add_const(&mut a, 10u64);
    let expected = [11, 12, 13, 14].map(TestScalar::from).to_vec();
    assert_eq!(a, expected);
}
