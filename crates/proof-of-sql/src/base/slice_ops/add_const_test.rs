use super::*;

#[test]
fn test_add_const() {
    let mut a = vec![1, 2, 3, 4];
    add_const(&mut a, 10);
    let b = vec![1 + 10, 2 + 10, 3 + 10, 4 + 10];
    assert_eq!(a, b);
}

#[test]
fn we_can_add_const_to_empty_slice() {
    let mut values: Vec<i32> = Vec::new();
    add_const(&mut values, 17);
    assert!(values.is_empty());
}

#[test]
fn we_can_add_const_from_convertible_type() {
    let mut values = vec![1_i64, -2, 3];
    add_const(&mut values, 4_i32);
    assert_eq!(values, [5, 2, 7]);
}
