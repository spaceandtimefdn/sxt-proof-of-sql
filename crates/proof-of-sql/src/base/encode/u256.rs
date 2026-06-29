use crate::base::scalar::MontScalar;
use ark_ff::MontConfig;

/// U256 represents an unsigned 256-bits integer number
///
/// low is the lower bytes of the u256 number (from 0 to 127 bits)
/// high is the upper bytes of the u256 number (from 128 to 255 bits)
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct U256 {
    pub low: u128,
    pub high: u128,
}

impl U256 {
    #[inline]
    pub const fn from_words(low: u128, high: u128) -> Self {
        U256 { low, high }
    }
}

/// This trait converts a dalek scalar into a U256 integer
impl<T: MontConfig<4>> From<&MontScalar<T>> for U256 {
    fn from(val: &MontScalar<T>) -> Self {
        let buf: [u64; 4] = val.into();
        let low: u128 = u128::from(buf[0]) | (u128::from(buf[1]) << 64);
        let high: u128 = u128::from(buf[2]) | (u128::from(buf[3]) << 64);
        U256::from_words(low, high)
    }
}

/// This trait converts a U256 integer into a dalek scalar
impl<T: MontConfig<4>> From<&U256> for MontScalar<T> {
    fn from(val: &U256) -> Self {
        let bytes = [val.low.to_le_bytes(), val.high.to_le_bytes()].concat();
        MontScalar::<T>::from_le_bytes_mod_order(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn from_words_preserves_low_and_high_halves() {
        let value = U256::from_words(0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210);

        assert_eq!(value.low, 0x0123_4567_89ab_cdef);
        assert_eq!(value.high, 0xfedc_ba98_7654_3210);
    }

    #[test]
    fn mont_scalar_to_u256_combines_little_endian_limbs() {
        let scalar = TestScalar::from([0x0123_4567_89ab_cdef, 0x1111_2222_3333_4444, 0, 0]);

        let value = U256::from(&scalar);

        assert_eq!(
            value.low,
            0x0123_4567_89ab_cdef | (0x1111_2222_3333_4444_u128 << 64)
        );
        assert_eq!(value.high, 0);
    }

    #[test]
    fn u256_to_mont_scalar_uses_low_then_high_little_endian_bytes() {
        let value = U256::from_words(
            0x0123_4567_89ab_cdef_1111_2222_3333_4444,
            0x5555_6666_7777_8888_9999_aaaa_bbbb_cccc,
        );
        let bytes = [value.low.to_le_bytes(), value.high.to_le_bytes()].concat();

        let scalar = TestScalar::from(&value);

        assert_eq!(scalar, TestScalar::from_le_bytes_mod_order(&bytes));
    }
}
