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
    let mut values: [i64; 0] = [];
    add_const(&mut values, 5_i64);
    assert!(values.is_empty());
}

#[test]
fn we_can_add_const_from_convertible_value() {
    let mut values = vec![10_i64, -5, 0, 12];
    add_const(&mut values, 7_i16);
    assert_eq!(values, [17_i64, 2, 7, 19]);
}
