use crate::base::scalar::ScalarExt;
use bnum::types::U256;
use core::ops::Shl;

#[inline]
pub fn make_bit_mask<S: ScalarExt>(x: S) -> U256 {
    let x_as_u256 = x.into_u256_wrapping();
    if x > S::MAX_SIGNED {
        x_as_u256 - S::MAX_SIGNED_U256 + (U256::ONE.shl(255)) - S::MAX_SIGNED_U256 - U256::ONE
    } else {
        x_as_u256 + (U256::ONE.shl(255))
    }
}

#[inline]
pub fn is_bit_mask_negative_representation(bit_mask: U256) -> bool {
    bit_mask & (U256::ONE.shl(255)) == U256::ZERO
}
