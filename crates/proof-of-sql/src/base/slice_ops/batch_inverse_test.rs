use crate::base::{scalar::test_scalar::TestScalar, slice_ops};
use num_traits::{Inv, Zero};

#[test]
fn we_can_pseudo_invert_empty_arrays() {
    let input: Vec<TestScalar> = Vec::new();
    let mut res = Vec::new();
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);
}

#[test]
fn we_can_pseudo_invert_arrays_of_length_1_with_non_zero() {
    let input = [TestScalar::from(2_u32)];
    let mut res = vec![TestScalar::from(0_u32)];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);
    assert!(res == vec![input[0].inv().unwrap()]);
}

#[test]
fn we_can_pseudo_invert_arrays_of_length_1_with_zero() {
    let input = [TestScalar::from(0_u32)];
    let mut res = vec![TestScalar::from(0_u32)];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);
    assert!(res == vec![input[0]]);
}

#[test]
fn we_can_pseudo_invert_arrays_of_length_bigger_than_1_with_zeros_and_non_zeros() {
    let input = [
        TestScalar::from(0_u32),
        TestScalar::from(2_u32),
        (-33_i32).into(),
        TestScalar::from(0_u32),
        TestScalar::from(45_u32),
        TestScalar::from(0_u32),
        TestScalar::from(47_u32),
    ];
    let mut res = vec![TestScalar::from(0_u32); input.len()];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);

    for (input_val, res_val) in input.iter().zip(res) {
        if *input_val == TestScalar::zero() {
            assert!(TestScalar::zero() == res_val);
        } else {
            assert!(input_val.inv().unwrap() == res_val);
        }
    }
}

#[test]
fn we_can_pseudo_invert_and_multiply_by_a_coefficient() {
    let input = [
        TestScalar::from(0_u32),
        TestScalar::from(2_u32),
        TestScalar::from(5_u32),
        TestScalar::from(0_u32),
        TestScalar::from(7_u32),
    ];
    let coeff = TestScalar::from(11_u32);
    let mut res = input.to_vec();

    slice_ops::batch_inversion_and_mul(&mut res[..], coeff);

    for (input_val, res_val) in input.iter().zip(res) {
        if input_val.is_zero() {
            assert_eq!(TestScalar::zero(), res_val);
        } else {
            assert_eq!(input_val.inv().unwrap() * coeff, res_val);
        }
    }
}

#[test]
fn we_can_pseudo_invert_arrays_with_nonzero_count_bigger_than_min_chunking_size_with_zeros_and_non_zeros(
) {
    let input: Vec<_> = vec![
        TestScalar::from(0_u32),
        TestScalar::from(2_u32),
        (-33_i32).into(),
        TestScalar::from(0_u32),
        TestScalar::from(45_u32),
        TestScalar::from(0_u32),
        TestScalar::from(47_u32),
    ]
    .into_iter()
    .cycle()
    .take(slice_ops::MIN_RAYON_LEN * 10)
    .collect();

    let mut res = vec![TestScalar::from(0_u32); input.len()];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);

    for (input_val, res_val) in input.iter().zip(res) {
        if *input_val == TestScalar::zero() {
            assert!(TestScalar::zero() == res_val);
        } else {
            assert!(input_val.inv().unwrap() == res_val);
        }
    }
}

#[test]
fn we_can_pseudo_invert_arrays_with_nonzero_count_smaller_than_min_chunking_size_with_zeros_and_non_zeros(
) {
    let input: Vec<_> = vec![
        TestScalar::from(0_u32),
        TestScalar::from(2_u32),
        (-33_i32).into(),
        TestScalar::from(0_u32),
        TestScalar::from(45_u32),
        TestScalar::from(0_u32),
        TestScalar::from(47_u32),
    ]
    .into_iter()
    .cycle()
    .take(slice_ops::MIN_RAYON_LEN - 1)
    .collect();

    let mut res = vec![TestScalar::from(0_u32); input.len()];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);

    for (input_val, res_val) in input.iter().zip(res) {
        if *input_val == TestScalar::zero() {
            assert!(TestScalar::zero() == res_val);
        } else {
            assert!(input_val.inv().unwrap() == res_val);
        }
    }
}
