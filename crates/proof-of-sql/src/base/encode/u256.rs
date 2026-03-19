use crate::base::scalar::MontScalar;
use ark_ff::MontConfig;

/// U256 represents an unsigned 256-bits integer number
///
/// low is the lower bytes of the u256 number (from 0 to 127 bits)
/// high is the upper bytes of the u256 number (from 128 to 255 bits)
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
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
    fn we_can_construct_u256_from_words() {
        let val = U256::from_words(42, 0);
        assert_eq!(val.low, 42);
        assert_eq!(val.high, 0);
    }

    #[test]
    fn u256_equality_works() {
        let a = U256::from_words(1, 2);
        let b = U256::from_words(1, 2);
        let c = U256::from_words(1, 3);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn u256_copy_clone_works() {
        let a = U256::from_words(100, 200);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn we_can_convert_scalar_to_u256_and_back() {
        let scalar = TestScalar::from(12345u64);
        let u256_val: U256 = U256::from(&scalar);
        let back: TestScalar = MontScalar::from(&u256_val);
        assert_eq!(scalar, back);
    }

    #[test]
    fn we_can_convert_zero_scalar_to_u256_and_back() {
        let scalar = TestScalar::from(0u64);
        let u256_val: U256 = U256::from(&scalar);
        let back: TestScalar = MontScalar::from(&u256_val);
        assert_eq!(scalar, back);
    }

    #[test]
    fn we_can_convert_u256_with_high_bits_to_scalar() {
        let u256_val = U256::from_words(0, 1);
        let scalar: TestScalar = MontScalar::from(&u256_val);
        // Verify roundtrip preserves the value mod the scalar field order
        let back: U256 = U256::from(&scalar);
        let back_scalar: TestScalar = MontScalar::from(&back);
        assert_eq!(scalar, back_scalar);
    }
}
