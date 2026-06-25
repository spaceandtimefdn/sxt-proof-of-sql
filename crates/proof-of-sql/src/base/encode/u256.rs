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

    /// Converts this integer into four little-endian `u64` limbs.
    pub(crate) fn to_limbs(self) -> [u64; 4] {
        let low = self.low.to_le_bytes();
        let high = self.high.to_le_bytes();

        [
            u64::from_le_bytes([
                low[0], low[1], low[2], low[3], low[4], low[5], low[6], low[7],
            ]),
            u64::from_le_bytes([
                low[8], low[9], low[10], low[11], low[12], low[13], low[14], low[15],
            ]),
            u64::from_le_bytes([
                high[0], high[1], high[2], high[3], high[4], high[5], high[6], high[7],
            ]),
            u64::from_le_bytes([
                high[8], high[9], high[10], high[11], high[12], high[13], high[14], high[15],
            ]),
        ]
    }
}

/// Converts a scalar into a U256 integer using its non-Montgomery limbs.
impl<S: Scalar> From<&S> for U256 {
    fn from(val: &S) -> Self {
        let buf: [u64; 4] = (*val).into();
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
