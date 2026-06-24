use crate::base::scalar::ScalarExt;
use bnum::types::U256;

/// Mask with only the most significant bit (255) set
/// Equivalent to `U256::ONE.shl(255)`
const MSB_MASK: U256 = U256::from_digits([0, 0, 0, 1 << 63]);

#[inline]
pub fn make_bit_mask<S: ScalarExt>(x: S) -> U256 {
    let x_as_u256 = x.into_u256_wrapping();
    if x > S::MAX_SIGNED {
        x_as_u256 - S::MAX_SIGNED_U256 + MSB_MASK - S::MAX_SIGNED_U256 - U256::ONE
    } else {
        x_as_u256 + MSB_MASK
    }
}

#[inline]
pub fn is_bit_mask_negative_representation(bit_mask: U256) -> bool {
    bit_mask & MSB_MASK == U256::ZERO
}

#[cfg(test)]
mod tests {
    use super::is_bit_mask_negative_representation;
    use bnum::types::U256;

    fn msb_mask() -> U256 {
        U256::ONE << 255u32
    }

    #[test]
    fn zero_has_no_msb_so_is_negative_representation() {
        assert!(is_bit_mask_negative_representation(U256::ZERO));
    }

    #[test]
    fn one_has_no_msb_so_is_negative_representation() {
        assert!(is_bit_mask_negative_representation(U256::ONE));
    }

    #[test]
    fn msb_set_alone_is_not_negative_representation() {
        assert!(!is_bit_mask_negative_representation(msb_mask()));
    }

    #[test]
    fn max_u256_has_msb_set_so_not_negative_representation() {
        assert!(!is_bit_mask_negative_representation(U256::MAX));
    }

    #[test]
    fn msb_plus_lower_bits_is_not_negative_representation() {
        let val = msb_mask() | U256::ONE;
        assert!(!is_bit_mask_negative_representation(val));
    }

    #[test]
    fn large_value_without_msb_is_negative_representation() {
        let val = msb_mask() - U256::ONE;
        assert!(is_bit_mask_negative_representation(val));
    }
}
