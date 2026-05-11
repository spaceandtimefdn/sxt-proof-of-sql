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
    fn from_words_preserves_low_and_high_words() {
        let value = U256::from_words(17, 23);

        assert_eq!(value.low, 17);
        assert_eq!(value.high, 23);
    }

    #[test]
    fn we_can_convert_scalar_to_u256_words() {
        let scalar = TestScalar::from_bigint([1, 2, 3, 4]);
        let value = U256::from(&scalar);

        assert_eq!(value.low, 1 | (2_u128 << 64));
        assert_eq!(value.high, 3 | (4_u128 << 64));
    }

    #[test]
    fn we_can_convert_u256_to_scalar_mod_order() {
        let value = U256::from_words(5 | (6_u128 << 64), 7 | (8_u128 << 64));
        let expected = TestScalar::from_le_bytes_mod_order(
            &[value.low.to_le_bytes(), value.high.to_le_bytes()].concat(),
        );

        assert_eq!(TestScalar::from(&value), expected);
    }

    #[test]
    fn u256_scalar_conversion_round_trips_canonical_scalar() {
        let scalar = TestScalar::from(123_456_789_u64);
        let value = U256::from(&scalar);

        assert_eq!(TestScalar::from(&value), scalar);
    }
}
