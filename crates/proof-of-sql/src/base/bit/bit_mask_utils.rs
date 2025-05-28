use crate::base::scalar::ScalarExt;
use bnum::types::U256;
use core::ops::Shl;

/// Creates a bit mask representation for scalar values.
///
/// Sign handling for scalars can be tricky because scalars represent elements in
/// a finite field, not traditional signed integers. The concept of "negative" numbers
/// in a field is based on whether a value is greater than the midpoint of the field
/// (`S::MAX_SIGNED`).
///
/// We use `U256::ONE.shl(255)` to transform the comparison with `S::MAX_SIGNED` into
/// a simple check of the leading bit in U256. This transformation maps:
/// - Values <= `S::MAX_SIGNED` to bit masks with MSB = 1 (representing "positive")
/// - Values > `S::MAX_SIGNED` to bit masks with MSB = 0 (representing "negative")
///
/// For negative scalars, we map -x = |S| - x to 2^255 - x, since 2 * `S::MAX_SIGNED` + 1 = |S|.
pub fn make_bit_mask<S: ScalarExt>(x: S) -> U256 {
    let x_as_u256 = x.into_u256_wrapping();
    if x > S::MAX_SIGNED {
        // For "negative" scalars: map -x = |S| - x to 2^255 - x
        x_as_u256 - S::into_u256_wrapping(S::MAX_SIGNED) + (U256::ONE.shl(255))
            - S::into_u256_wrapping(S::MAX_SIGNED)
            - U256::ONE
    } else {
        // For "positive" scalars: set the MSB by adding 2^255
        x_as_u256 + (U256::ONE.shl(255))
    }
}

pub fn is_bit_mask_negative_representation(bit_mask: U256) -> bool {
    bit_mask & (U256::ONE.shl(255)) == U256::ZERO
}
