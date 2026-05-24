use super::*;
use crate::base::{bit::BitDistribution, scalar::test_scalar::TestScalar};
use bumpalo::Bump;
use num_traits::{One, Zero};

#[test]
fn we_can_compute_the_bit_matrix_of_empty_data() {
    let data: Vec<TestScalar> = vec![];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert!(matrix.is_empty());
}

#[test]
fn we_can_compute_the_bit_matrix_for_a_single_element() {
    let data: Vec<TestScalar> = vec![TestScalar::one()];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert!(matrix.is_empty());
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_a_single_varying_bit() {
    let data: Vec<TestScalar> = vec![TestScalar::one(), TestScalar::zero()];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 1);
    let slice1 = vec![true, false];
    assert_eq!(matrix[0], slice1);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_a_varying_sign_bit() {
    let data: Vec<TestScalar> = vec![TestScalar::one(), -TestScalar::one()];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![true, true];
    let slice2 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_varying_bits_in_different_positions() {
    let data: Vec<TestScalar> = vec![TestScalar::from(2), TestScalar::one()];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![false, true];
    let slice2 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_with_varying_bits_and_constant_bits() {
    let data: Vec<TestScalar> = vec![TestScalar::from(3), TestScalar::from(-1)];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 3);
    let slice1 = vec![true, true];
    let slice2 = vec![true, true];
    let slice3 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
    assert_eq!(matrix[2], slice3);
}

#[test]
fn we_preserve_varying_bit_order_across_multiple_rows() {
    let data: Vec<TestScalar> = vec![
        TestScalar::zero(),
        TestScalar::one(),
        TestScalar::from(2),
        TestScalar::from(4),
    ];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 3);
    let slice1 = vec![false, true, false, false];
    let slice2 = vec![false, false, true, false];
    let slice3 = vec![false, false, false, true];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
    assert_eq!(matrix[2], slice3);
}

#[test]
fn we_preserve_cross_limb_varying_bit_order() {
    let mut val = [0; 4];
    val[2] = 1 << 2;
    let data: Vec<TestScalar> = vec![
        TestScalar::zero(),
        TestScalar::one(),
        TestScalar::from_bigint(val),
    ];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![false, true, false];
    let slice2 = vec![false, false, true];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}

#[test]
fn we_can_compute_the_bit_matrix_for_data_entries_bigger_than_64_bit_integers() {
    let mut val = [0; 4];
    val[3] = 1 << 2;
    let data: Vec<TestScalar> = vec![TestScalar::from_bigint(val), TestScalar::one()];
    let dist = BitDistribution::new::<TestScalar, _>(&data);
    let alloc = Bump::new();
    let matrix = compute_varying_bit_matrix(&alloc, &data, &dist);
    assert_eq!(matrix.len(), 2);
    let slice1 = vec![false, true];
    let slice2 = vec![true, false];
    assert_eq!(matrix[0], slice1);
    assert_eq!(matrix[1], slice2);
}
