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
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_is_bit_mask_negative_representation_msb_set() {
        // MSB set → NOT a negative representation → returns false
        assert!(!is_bit_mask_negative_representation(MSB_MASK));
    }

    #[test]
    fn test_is_bit_mask_negative_representation_msb_clear() {
        // MSB clear → negative representation → returns true
        assert!(is_bit_mask_negative_representation(U256::ZERO));
        assert!(is_bit_mask_negative_representation(U256::ONE));
    }

    #[test]
    fn test_make_bit_mask_non_negative_value() {
        // For a non-negative value (x <= MAX_SIGNED), the bit mask
        // has the MSB set (i.e. the result is NOT a negative representation).
        let x = TestScalar::ZERO;
        let mask = make_bit_mask(x);
        assert!(!is_bit_mask_negative_representation(mask));
    }

    #[test]
    fn test_make_bit_mask_consistency() {
        // Two identical scalars should produce the same bit mask.
        let x = TestScalar::from(42u64);
        assert_eq!(make_bit_mask(x), make_bit_mask(x));
    }
}
