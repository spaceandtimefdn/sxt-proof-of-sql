use super::bit_mask_utils::make_bit_mask;
use crate::base::{
    bit::bit_mask_utils::is_bit_mask_negative_representation,
    scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
};
use bnum::types::U256;
use core::ops::Shl;

#[test]
fn we_can_make_positive_bit_mask() {
    // ARRANGE
    let positive_scalar = TestScalar::TWO;

    // ACT
    let bit_mask = make_bit_mask(positive_scalar);

    // ASSERT
    assert_eq!(bit_mask, (U256::ONE.shl(255)) + U256::TWO);
}

#[test]
fn we_can_make_negative_bit_mask() {
    // ARRANGE
    let negative_scalar = -TestScalar::TWO;

    // ACT
    let bit_mask = make_bit_mask(negative_scalar);

    // ASSERT
    assert_eq!(bit_mask, (U256::ONE.shl(255)) - U256::TWO);
}

#[test]
fn we_can_verify_positive_bit_mask_is_positive_representation() {
    // ARRANGE
    let positive_scalar = TestScalar::TWO;
    let bit_mask = make_bit_mask(positive_scalar);

    // ACT
    let is_positive = !is_bit_mask_negative_representation(bit_mask);

    // ASSERT
    assert!(is_positive);
}

#[test]
fn we_can_verify_negative_bit_mask_is_negative_representation() {
    // ARRANGE
    let negative_scalar = -TestScalar::TWO;
    let bit_mask = make_bit_mask(negative_scalar);

    // ACT
    let is_negative = is_bit_mask_negative_representation(bit_mask);

    // ASSERT
    assert!(is_negative);
}

#[test]
fn zero_maps_to_msb_only_and_is_not_negative_representation() {
    // ACT
    let bit_mask = make_bit_mask(TestScalar::ZERO);

    // ASSERT
    assert_eq!(bit_mask, U256::ONE.shl(255));
    assert!(!is_bit_mask_negative_representation(bit_mask));
}

#[test]
fn max_signed_stays_in_positive_representation_branch() {
    // ACT
    let bit_mask = make_bit_mask(TestScalar::MAX_SIGNED);

    // ASSERT
    assert_eq!(bit_mask, U256::ONE.shl(255) + TestScalar::MAX_SIGNED_U256);
    assert!(!is_bit_mask_negative_representation(bit_mask));
}

#[test]
fn first_value_above_max_signed_switches_to_negative_representation() {
    // ARRANGE
    let first_negative_like_value = TestScalar::from_wrapping(TestScalar::MAX_SIGNED_U256 + U256::ONE);

    // ACT
    let bit_mask = make_bit_mask(first_negative_like_value);

    // ASSERT
    assert_eq!(bit_mask, (U256::ONE.shl(255)) - TestScalar::MAX_SIGNED_U256);
    assert!(is_bit_mask_negative_representation(bit_mask));
}
