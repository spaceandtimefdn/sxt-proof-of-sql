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
    let mut values: Vec<i64> = vec![];

    add_const(&mut values, 10);

    assert!(values.is_empty());
}

#[test]
fn we_can_add_convertible_const() {
    let mut values = vec![10_i64, 20, 30];

    add_const(&mut values, 7_i32);

    assert_eq!(values, vec![17, 27, 37]);
}
