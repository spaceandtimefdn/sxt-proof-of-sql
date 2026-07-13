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

    fn assert_u256_eq(actual: U256, expected: U256) {
        assert_eq!(actual.low, expected.low);
        assert_eq!(actual.high, expected.high);
    }

    #[test]
    fn from_words_preserves_low_and_high_halves() {
        let value = U256::from_words(
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
            0x1111_2222_3333_4444_5555_6666_7777_8888,
        );

        assert_eq!(value.low, 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
        assert_eq!(value.high, 0x1111_2222_3333_4444_5555_6666_7777_8888);
    }

    #[test]
    fn scalar_round_trip_preserves_u256_word_order() {
        let value = U256::from_words(
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
            0x0000_0000_0000_0007_0123_4567_89ab_cdef,
        );

        let scalar: TestScalar = (&value).into();
        let round_tripped = U256::from(&scalar);

        assert_u256_eq(round_tripped, value);
    }
}
