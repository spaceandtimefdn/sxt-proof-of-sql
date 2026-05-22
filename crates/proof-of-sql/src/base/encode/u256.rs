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
    use super::U256;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn from_words_preserves_low_and_high_limbs() {
        let value = U256::from_words(
            0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00,
            0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10,
        );

        assert_eq!(value.low, 0x1122_3344_5566_7788_99aa_bbcc_ddee_ff00);
        assert_eq!(value.high, 0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10);
    }

    #[test]
    fn scalar_conversion_splits_limbs_into_low_and_high_words() {
        let scalar = TestScalar::from_bigint([
            0x0123_4567_89ab_cdef,
            0xfedc_ba98_7654_3210,
            0x0f0e_0d0c_0b0a_0908,
            0x0000_0000_0000_0007,
        ]);
        let value = U256::from(&scalar);

        assert_eq!(value.low, 0xfedc_ba98_7654_3210_0123_4567_89ab_cdef);
        assert_eq!(value.high, 0x0000_0000_0000_0007_0f0e_0d0c_0b0a_0908);
    }

    #[test]
    fn u256_conversion_restores_scalar_limbs() {
        let value = U256::from_words(
            0x89ab_cdef_0123_4567_0011_2233_4455_6677,
            0x0000_0000_0000_0004_7788_99aa_bbcc_ddee,
        );
        let scalar = TestScalar::from(&value);

        assert_eq!(
            <[u64; 4]>::from(scalar),
            [
                0x0011_2233_4455_6677,
                0x89ab_cdef_0123_4567,
                0x7788_99aa_bbcc_ddee,
                0x0000_0000_0000_0004,
            ]
        );
    }
}
