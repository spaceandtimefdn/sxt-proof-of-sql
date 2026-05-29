use super::*;

#[test]
fn test_add_const() {
    let mut a = vec![1, 2, 3, 4];
    add_const(&mut a, 10);
    let b = vec![1 + 10, 2 + 10, 3 + 10, 4 + 10];
    assert_eq!(a, b);
}

#[test]
fn test_add_const_empty_slice() {
    let mut values: Vec<i64> = Vec::new();
    add_const(&mut values, 5);
    assert!(values.is_empty());
}

#[test]
fn test_add_const_converts_addend() {
    let mut values = vec![1_i64, -2, 3];
    add_const(&mut values, 7_i32);
    assert_eq!(values, vec![8, 5, 10]);
}
