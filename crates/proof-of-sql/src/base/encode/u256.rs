use crate::base::scalar::Scalar;

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

/// Convert any `Scalar` value into a `U256`, packing its canonical limbs.
///
/// The orphan rule lets us blanket this conversion because `U256` is local
/// to this crate. The body only depends on the `Scalar` trait surface
/// (`to_limbs`), so future `Scalar` impls work for free.
impl<S: Scalar> From<&S> for U256 {
    fn from(val: &S) -> Self {
        let buf: [u64; 4] = val.to_limbs();
        let low: u128 = u128::from(buf[0]) | (u128::from(buf[1]) << 64);
        let high: u128 = u128::from(buf[2]) | (u128::from(buf[3]) << 64);
        U256::from_words(low, high)
    }
}

/// Convert a `U256` into a `Scalar`, reducing modulo the field's prime.
///
/// Unlike the `From<&S>` direction, this conversion can't be expressed as
/// a `From` blanket impl (orphan rule: both `From` and the generic `S`
/// would be foreign at the impl site). Use `Scalar::from_le_bytes_mod_order`
/// directly via this helper; it preserves the previous
/// `From<&U256> for MontScalar<T>` semantics for any `S: Scalar`.
#[inline]
pub fn u256_to_scalar<S: Scalar>(val: &U256) -> S {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(&val.low.to_le_bytes());
    bytes[16..].copy_from_slice(&val.high.to_le_bytes());
    S::from_le_bytes_mod_order(&bytes)
}
