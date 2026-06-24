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
    fn from_words_stores_low_and_high() {
        let u = U256::from_words(42, 100);
        assert_eq!(u.low, 42);
        assert_eq!(u.high, 100);
    }

    #[test]
    fn from_words_zero_both_parts() {
        let u = U256::from_words(0, 0);
        assert_eq!(u.low, 0);
        assert_eq!(u.high, 0);
    }

    #[test]
    fn from_words_max_values() {
        let u = U256::from_words(u128::MAX, u128::MAX);
        assert_eq!(u.low, u128::MAX);
        assert_eq!(u.high, u128::MAX);
    }

    #[test]
    fn u256_equality() {
        let a = U256::from_words(1, 2);
        let b = U256::from_words(1, 2);
        assert_eq!(a, b);
    }

    #[test]
    fn u256_inequality_different_low() {
        let a = U256::from_words(1, 0);
        let b = U256::from_words(2, 0);
        assert_ne!(a, b);
    }

    #[test]
    fn u256_inequality_different_high() {
        let a = U256::from_words(0, 1);
        let b = U256::from_words(0, 2);
        assert_ne!(a, b);
    }

    #[test]
    fn u256_copy_trait_works() {
        let a = U256::from_words(5, 10);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn scalar_zero_converts_to_zero_u256() {
        let scalar = TestScalar::from(0);
        let u: U256 = (&scalar).into();
        assert_eq!(u.low, 0);
        assert_eq!(u.high, 0);
    }

    #[test]
    fn small_scalar_converts_to_u256_with_correct_low() {
        let scalar = TestScalar::from(255);
        let u: U256 = (&scalar).into();
        assert_eq!(u.low, 255);
        assert_eq!(u.high, 0);
    }

    #[test]
    fn u256_zero_roundtrip_via_scalar() {
        let u = U256::from_words(0, 0);
        let scalar: TestScalar = (&u).into();
        assert_eq!(scalar, TestScalar::from(0));
    }
}
