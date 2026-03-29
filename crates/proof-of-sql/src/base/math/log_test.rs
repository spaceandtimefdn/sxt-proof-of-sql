use super::{log2_up, max_log2_up};

#[test]
fn test_log2_up_zero() {
    // log2_up(0) should be 0
    assert_eq!(log2_up(0usize), 0);
}

#[test]
fn test_log2_up_one() {
    // log2_up(1) = 0 (2^0 = 1 covers value 1)
    assert_eq!(log2_up(1usize), 0);
}

#[test]
fn test_log2_up_exact_powers_of_two() {
    assert_eq!(log2_up(2usize), 1);
    assert_eq!(log2_up(4usize), 2);
    assert_eq!(log2_up(8usize), 3);
    assert_eq!(log2_up(16usize), 4);
    assert_eq!(log2_up(1024usize), 10);
}

#[test]
fn test_log2_up_non_powers_of_two() {
    // log2_up rounds up to the next power of two exponent
    assert_eq!(log2_up(3usize), 2); // 2^2 = 4 >= 3
    assert_eq!(log2_up(5usize), 3); // 2^3 = 8 >= 5
    assert_eq!(log2_up(7usize), 3); // 2^3 = 8 >= 7
    assert_eq!(log2_up(9usize), 4); // 2^4 = 16 >= 9
    assert_eq!(log2_up(15usize), 4); // 2^4 = 16 >= 15
}

#[test]
fn test_log2_up_large_value() {
    // 2^20 = 1048576
    assert_eq!(log2_up(1_048_576usize), 20);
    // 1_048_577 requires 2^21
    assert_eq!(log2_up(1_048_577usize), 21);
}

#[test]
fn test_max_log2_up_empty_slice() {
    let values: &[usize] = &[];
    assert_eq!(max_log2_up(values), 0);
}

#[test]
fn test_max_log2_up_single_element() {
    assert_eq!(max_log2_up(&[8usize]), 3);
}

#[test]
fn test_max_log2_up_multiple_elements() {
    // max of log2_up over [3, 8, 15] = max(2, 3, 4) = 4
    assert_eq!(max_log2_up(&[3usize, 8, 15]), 4);
}

#[test]
fn test_max_log2_up_all_same() {
    assert_eq!(max_log2_up(&[4usize, 4, 4]), 2);
}
