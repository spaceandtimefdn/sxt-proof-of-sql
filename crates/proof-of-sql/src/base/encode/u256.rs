use crate::base::scalar::{MontScalar, Scalar};
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

/// Convert [u64; 4] to U256
impl From<[u64; 4]> for U256 {
    fn from(buf: [u64; 4]) -> Self {
        let low: u128 = u128::from(buf[0]) | (u128::from(buf[1]) << 64);
        let high: u128 = u128::from(buf[2]) | (u128::from(buf[3]) << 64);
        U256::from_words(low, high)
    }
}

/// Convert &U256 to [u64; 4]
impl From<&U256> for [u64; 4] {
    fn from(val: &U256) -> Self {
        [
            val.low as u64,
            (val.low >> 64) as u64,
            val.high as u64,
            (val.high >> 64) as u64,
        ]
    }
}

/// Convert U256 to [u64; 4]
impl From<U256> for [u64; 4] {
    fn from(val: U256) -> Self {
        <[u64; 4]>::from(&val)
    }
}

/// This trait converts any scalar into a U256 integer
impl<S: Scalar> From<&S> for U256 {
    fn from(val: &S) -> Self {
        U256::from(val.to_limbs())
    }
}

/// This trait converts a U256 integer into a dalek scalar
impl<T: MontConfig<4>> From<&U256> for MontScalar<T> {
    fn from(val: &U256) -> Self {
        let bytes = [val.low.to_le_bytes(), val.high.to_le_bytes()].concat();
        MontScalar::<T>::from_le_bytes_mod_order(&bytes)
    }
}
