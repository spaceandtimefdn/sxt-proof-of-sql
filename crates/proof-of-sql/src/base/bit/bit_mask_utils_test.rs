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
fn we_can_make_zero_bit_mask() {
    let bit_mask = make_bit_mask(TestScalar::ZERO);

    assert_eq!(bit_mask, U256::ONE.shl(255));
    assert!(!is_bit_mask_negative_representation(bit_mask));
}

#[test]
fn we_can_make_signed_boundary_bit_masks() {
    let max_signed_mask = make_bit_mask(TestScalar::MAX_SIGNED);
    assert_eq!(
        max_signed_mask,
        (U256::ONE.shl(255)) + TestScalar::MAX_SIGNED.into_u256_wrapping()
    );
    assert!(!is_bit_mask_negative_representation(max_signed_mask));

    let min_signed_mask = make_bit_mask(-TestScalar::MAX_SIGNED);
    assert_eq!(
        min_signed_mask,
        (U256::ONE.shl(255)) - TestScalar::MAX_SIGNED_U256
    );
    assert!(is_bit_mask_negative_representation(min_signed_mask));
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
