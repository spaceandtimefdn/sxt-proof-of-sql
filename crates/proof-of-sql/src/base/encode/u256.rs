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

    #[inline]
    pub(crate) const fn from_limbs(limbs: [u64; 4]) -> Self {
        let low = limbs[0] as u128 | ((limbs[1] as u128) << 64);
        let high = limbs[2] as u128 | ((limbs[3] as u128) << 64);
        U256::from_words(low, high)
    }

    #[expect(clippy::cast_possible_truncation)]
    #[inline]
    pub(crate) const fn to_limbs(self) -> [u64; 4] {
        [
            self.low as u64,
            (self.low >> 64) as u64,
            self.high as u64,
            (self.high >> 64) as u64,
        ]
    }
}

/// This trait converts a dalek scalar into a U256 integer
impl<T: MontConfig<4>> From<&MontScalar<T>> for U256 {
    fn from(val: &MontScalar<T>) -> Self {
        let limbs: [u64; 4] = val.into();
        U256::from_limbs(limbs)
    }
}

/// This trait converts a U256 integer into a dalek scalar
impl<T: MontConfig<4>> From<&U256> for MontScalar<T> {
    fn from(val: &U256) -> Self {
        MontScalar::<T>::from(val.to_limbs())
    }
}

#[cfg(test)]
mod tests {
    use super::U256;

    #[test]
    fn limbs_round_trip_in_little_endian_word_order() {
        let limbs = [
            0x0123_4567_89ab_cdef,
            0xfedc_ba98_7654_3210,
            0x0f1e_2d3c_4b5a_6978,
            0x8877_6655_4433_2211,
        ];

        let value = U256::from_limbs(limbs);

        let expected = U256::from_words(
            0xfedc_ba98_7654_3210_0123_4567_89ab_cdef,
            0x8877_6655_4433_2211_0f1e_2d3c_4b5a_6978,
        );
        assert_eq!(value.low, expected.low);
        assert_eq!(value.high, expected.high);
        assert_eq!(value.to_limbs(), limbs);
    }
}
