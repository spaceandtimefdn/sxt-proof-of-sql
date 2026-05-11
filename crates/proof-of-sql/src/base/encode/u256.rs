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
    fn from_words_keeps_low_and_high_halves() {
        let value = U256::from_words(123, 456);

        assert_eq!(value.low, 123);
        assert_eq!(value.high, 456);
    }

    #[test]
    fn mont_scalar_to_u256_uses_little_endian_word_layout() {
        let scalar = TestScalar::from([1, 2, 3, 4]);

        let value = U256::from(&scalar);

        assert_eq!(value.low, u128::from(1_u64) | (u128::from(2_u64) << 64));
        assert_eq!(value.high, u128::from(3_u64) | (u128::from(4_u64) << 64));
    }

    #[test]
    fn u256_to_mont_scalar_uses_little_endian_word_layout() {
        let value = U256::from_words(
            u128::from(11_u64) | (u128::from(22_u64) << 64),
            u128::from(33_u64) | (u128::from(44_u64) << 64),
        );

        let scalar: TestScalar = (&value).into();

        assert_eq!(scalar, TestScalar::from([11, 22, 33, 44]));
    }

    #[test]
    fn u256_roundtrips_back_into_the_same_scalar() {
        let original = TestScalar::from([9, 8, 7, 6]);

        let value = U256::from(&original);
        let recovered: TestScalar = (&value).into();

        assert_eq!(recovered, original);
    }
}
