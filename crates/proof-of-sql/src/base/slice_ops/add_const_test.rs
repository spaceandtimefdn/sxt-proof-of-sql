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
fn test_add_const_empty_slice() {
    let mut values: Vec<i32> = Vec::new();
    add_const(&mut values, 10);
    assert!(values.is_empty());
}

#[test]
fn test_add_const_converts_to_result_type() {
    let mut values = [1, 2, 3].map(TestScalar::from);
    add_const(&mut values, 7u64);

    assert_eq!(values, [8, 9, 10].map(TestScalar::from));
}
