/**
 * Adapted from integer-encoding-rs
 *
 * See third_party/license/integer-encoding.LICENSE
 */
// ---------------------------------------------------------------------------------------------------------------
// The following chunk of code is copied from the `integer-encoding`. This is for two reasons:
// 1) it makes the `VarInt` no longer a foreign trait
// 2) there is a bug in `integer-encoding` that made it so that large decodings didn't fail when they should have
// There were significant code changes to simplify the code
// ---------------------------------------------------------------------------------------------------------------
use super::{
    scalar_varint::{
        read_scalar_varint, read_u256_varint, scalar_varint_size, u256_varint_size,
        write_scalar_varint, write_u256_varint,
    },
    U256,
};
use crate::base::scalar::MontScalar;
#[cfg(test)]
use alloc::{vec, vec::Vec};
use ark_ff::MontConfig;

/// Most-significant byte, == 0x80
pub const MSB: u8 = 0b1000_0000;
/// All bits except for the most significant. Can be used as bitmask to drop the most-significant
/// bit using `&` (binary-and).
const DROP_MSB: u8 = 0b0111_1111;

/// Varint (variable length integer) encoding, as described in
/// <https://developers.google.com/protocol-buffers/docs/encoding>.
///
/// Uses zigzag encoding (also described there) for signed integer representation.
pub trait VarInt: Sized + Copy {
    /// Returns the number of bytes this number needs in its encoded form. Note: This varies
    /// depending on the actual number you want to encode.
    fn required_space(self) -> usize;
    /// Decode a value from the slice. Returns the value and the number of bytes read from the
    /// slice (can be used to read several consecutive values from a big slice)
    /// return None if the decoded value overflows this type.
    fn decode_var(src: &[u8]) -> Option<(Self, usize)>;
    /// Encode a value into the slice. The slice must be at least `required_space()` bytes long.
    /// The number of bytes taken by the encoded integer is returned.
    fn encode_var(self, src: &mut [u8]) -> usize;

    /// Helper: Encode a value and return the encoded form as Vec. The Vec must be at least
    /// `required_space()` bytes long.
    #[cfg(test)]
    fn encode_var_vec(self) -> Vec<u8> {
        let mut v = vec![0; self.required_space()];
        self.encode_var(&mut v);
        v
    }
}

#[expect(clippy::cast_sign_loss)]
#[inline]
fn zigzag_encode(from: i64) -> u64 {
    ((from << 1) ^ (from >> 63)) as u64
}

// see: http://stackoverflow.com/a/2211086/56332
// casting required because operations like unary negation
// cannot be performed on unsigned integers
#[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
#[inline]
fn zigzag_decode(from: u64) -> i64 {
    ((from >> 1) ^ (-((from & 1) as i64)) as u64) as i64
}

/// Macro to implement [`VarInt`] for integer types (both signed and unsigned).
macro_rules! impl_varint {
    ($t:ty, unsigned) => {
        impl VarInt for $t {
            #[allow(clippy::cast_lossless, clippy::allow_attributes, reason = "In a macro different instances can differ hence allow can not be replaced with expect")]
            fn required_space(self) -> usize {
                (self as u64).required_space()
            }

            #[expect(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_lossless, clippy::allow_attributes, reason = "In a macro different instances can differ hence allow can not be replaced with expect")]
            fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
                let (n, s) = u64::decode_var(src)?;
                // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `Self`.
                if n > (Self::MAX as u64) {
                    None
                } else {
                    Some((n as Self, s))
                }
            }

            #[allow(clippy::cast_lossless, clippy::allow_attributes, reason = "In a macro different instances can differ hence allow can not be replaced with expect")]
            fn encode_var(self, dst: &mut [u8]) -> usize {
                (self as u64).encode_var(dst)
            }
        }
    };
    ($t:ty, signed) => {
        impl VarInt for $t {
            #[allow(clippy::cast_lossless, clippy::allow_attributes, reason = "In a macro different instances can differ hence allow can not be replaced with expect")]
            fn required_space(self) -> usize {
                (self as i64).required_space()
            }

            #[expect(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_lossless, clippy::allow_attributes, reason = "In a macro different instances can differ hence allow can not be replaced with expect")]
            fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
                let (n, s) = i64::decode_var(src)?;
                // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `Self`.
                if n > (Self::MAX as i64) || n < (Self::MIN as i64) {
                    None
                } else {
                    Some((n as Self, s))
                }
            }

            #[allow(clippy::cast_lossless, clippy::allow_attributes, reason = "In a macro different instances can differ hence allow can not be replaced with expect")]
            fn encode_var(self, dst: &mut [u8]) -> usize {
                (self as i64).encode_var(dst)
            }
        }
    };
}

impl_varint!(usize, unsigned);
impl_varint!(u32, unsigned);
impl_varint!(u16, unsigned);
impl_varint!(u8, unsigned);

impl_varint!(isize, signed);
impl_varint!(i32, signed);
impl_varint!(i16, signed);
impl_varint!(i8, signed);

impl VarInt for bool {
    fn required_space(self) -> usize {
        u64::from(self).required_space()
    }

    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let (n, s) = u64::decode_var(src)?;
        // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `Self`.
        match n {
            0 => Some((false, s)),
            1 => Some((true, s)),
            _ => None,
        }
    }

    fn encode_var(self, dst: &mut [u8]) -> usize {
        u64::from(self).encode_var(dst)
    }
}

// Below are the "base implementations" doing the actual encodings; all other integer types are
// first cast to these biggest types before being encoded.

impl VarInt for u64 {
    fn required_space(self) -> usize {
        let bits = 64 - self.leading_zeros() as usize;
        core::cmp::max(1, bits.div_ceil(7))
    }

    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let mut result: u64 = 0;
        let mut shift = 0;

        let mut success = false;
        for b in src {
            let msb_dropped = b & DROP_MSB;
            result |= u64::from(msb_dropped) << shift;
            shift += 7;

            if shift > (9 * 7) {
                // This check is required to ensure that we actually return `None` when `src` has a value that would overflow `u64`.
                success = *b < 2;
                break;
            } else if b & MSB == 0 {
                success = true;
                break;
            }
        }

        if success {
            Some((result, shift / 7))
        } else {
            None
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    #[inline]
    fn encode_var(self, dst: &mut [u8]) -> usize {
        assert!(dst.len() >= self.required_space());
        let mut n = self;
        let mut i = 0;

        while n >= 0x80 {
            dst[i] = MSB | (n as u8);
            i += 1;
            n >>= 7;
        }

        dst[i] = n as u8;
        i + 1
    }
}

impl VarInt for i64 {
    fn required_space(self) -> usize {
        zigzag_encode(self).required_space()
    }

    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        let (result, size) = u64::decode_var(src)?;
        Some((zigzag_decode(result), size))
    }

    #[inline]
    fn encode_var(self, dst: &mut [u8]) -> usize {
        zigzag_encode(self).encode_var(dst)
    }
}

impl VarInt for U256 {
    fn required_space(self) -> usize {
        u256_varint_size(self)
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        read_u256_varint(src)
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        write_u256_varint(dst, self)
    }
}

impl VarInt for u128 {
    fn required_space(self) -> usize {
        U256 { low: self, high: 0 }.required_space()
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        match U256::decode_var(src)? {
            (U256 { high: 0, low }, s) => Some((low, s)),
            _ => None,
        }
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        U256 { low: self, high: 0 }.encode_var(dst)
    }
}

// Adapted from integer-encoding-rs. See third_party/license/integer-encoding.LICENSE
#[expect(clippy::cast_sign_loss)]
#[inline]
fn zigzag_encode_i128(from: i128) -> u128 {
    ((from << 1) ^ (from >> 127)) as u128
}
// Adapted from integer-encoding-rs. See third_party/license/integer-encoding.LICENSE
// see: http://stackoverflow.com/a/2211086/56332
// casting required because operations like unary negation
// cannot be performed on unsigned integers
#[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
#[inline]
fn zigzag_decode_i128(from: u128) -> i128 {
    ((from >> 1) ^ (-((from & 1) as i128)) as u128) as i128
}
impl VarInt for i128 {
    fn required_space(self) -> usize {
        u128::required_space(zigzag_encode_i128(self))
    }

    #[inline]
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        u128::decode_var(src).map(|(v, s)| (zigzag_decode_i128(v), s))
    }

    #[inline]
    fn encode_var(self, dst: &mut [u8]) -> usize {
        zigzag_encode_i128(self).encode_var(dst)
    }
}

impl<T: MontConfig<4>> VarInt for MontScalar<T> {
    fn required_space(self) -> usize {
        scalar_varint_size(&self)
    }
    fn decode_var(src: &[u8]) -> Option<(Self, usize)> {
        read_scalar_varint(src)
    }
    fn encode_var(self, dst: &mut [u8]) -> usize {
        write_scalar_varint(dst, &self)
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use alloc::{vec, vec::Vec};

    #[test]
    fn u64_zero_requires_one_byte() {
        assert_eq!(0u64.required_space(), 1);
    }

    #[test]
    fn u64_127_requires_one_byte() {
        assert_eq!(127u64.required_space(), 1);
    }

    #[test]
    fn u64_128_requires_two_bytes() {
        assert_eq!(128u64.required_space(), 2);
    }

    #[test]
    fn u64_max_requires_ten_bytes() {
        assert_eq!(u64::MAX.required_space(), 10);
    }

    #[test]
    fn u64_zero_encode_decode_roundtrip() {
        let encoded = 0u64.encode_var_vec();
        let (decoded, bytes_read) = u64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, 0u64);
        assert_eq!(bytes_read, 1);
    }

    #[test]
    fn u64_small_value_roundtrip() {
        let val = 42u64;
        let encoded = val.encode_var_vec();
        let (decoded, _) = u64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u64_max_roundtrip() {
        let val = u64::MAX;
        let encoded = val.encode_var_vec();
        let (decoded, bytes) = u64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(bytes, encoded.len());
    }

    #[test]
    fn u64_256_requires_two_bytes() {
        assert_eq!(256u64.required_space(), 2);
    }

    #[test]
    fn u64_decode_returns_none_on_empty_slice() {
        assert_eq!(u64::decode_var(&[]), None);
    }

    #[test]
    fn i64_zero_roundtrip() {
        let val = 0i64;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i64_positive_roundtrip() {
        let val = 100i64;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i64_negative_one_roundtrip() {
        let val = -1i64;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i64_negative_large_roundtrip() {
        let val = -1000000i64;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i64_min_roundtrip() {
        let val = i64::MIN;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i64_max_roundtrip() {
        let val = i64::MAX;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i64::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u32_roundtrip() {
        let val = u32::MAX;
        let encoded = val.encode_var_vec();
        let (decoded, _) = u32::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u32_overflow_returns_none() {
        // Encode a value that overflows u32
        let large: u64 = u64::from(u32::MAX) + 1;
        let encoded = large.encode_var_vec();
        assert_eq!(u32::decode_var(&encoded), None);
    }

    #[test]
    fn u16_roundtrip() {
        let val = u16::MAX;
        let encoded = val.encode_var_vec();
        let (decoded, _) = u16::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u8_roundtrip() {
        let val = 255u8;
        let encoded = val.encode_var_vec();
        let (decoded, _) = u8::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i32_negative_roundtrip() {
        let val = -42i32;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i32::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i32_min_roundtrip() {
        let val = i32::MIN;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i32::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i16_roundtrip() {
        let val = i16::MIN;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i16::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i8_roundtrip() {
        let val = -127i8;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i8::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn bool_true_roundtrip() {
        let encoded = true.encode_var_vec();
        let (decoded, _) = bool::decode_var(&encoded).unwrap();
        assert!(decoded);
    }

    #[test]
    fn bool_false_roundtrip() {
        let encoded = false.encode_var_vec();
        let (decoded, _) = bool::decode_var(&encoded).unwrap();
        assert!(!decoded);
    }

    #[test]
    fn u128_zero_roundtrip() {
        let val = 0u128;
        let encoded = val.encode_var_vec();
        let (decoded, _) = u128::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u128_max_roundtrip() {
        let val = u128::MAX;
        let encoded = val.encode_var_vec();
        let (decoded, _) = u128::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i128_negative_roundtrip() {
        let val = -1i128;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i128::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i128_min_roundtrip() {
        let val = i128::MIN;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i128::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn i128_max_roundtrip() {
        let val = i128::MAX;
        let encoded = val.encode_var_vec();
        let (decoded, _) = i128::decode_var(&encoded).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u64_encode_length_matches_required_space() {
        for val in [0u64, 1, 127, 128, 255, 256, 65535, 65536, u64::MAX] {
            assert_eq!(val.encode_var_vec().len(), val.required_space());
        }
    }

    #[test]
    fn consecutive_u64_values_can_be_decoded_sequentially() {
        let mut buf = vec![0u8; 20];
        let n1 = 42u64.encode_var(&mut buf);
        let n2 = 1000u64.encode_var(&mut buf[n1..]);
        let (v1, s1) = u64::decode_var(&buf).unwrap();
        let (v2, _) = u64::decode_var(&buf[s1..]).unwrap();
        assert_eq!(v1, 42);
        assert_eq!(v2, 1000);
        assert_eq!(s1, n1);
        let _ = n2; // suppress unused warning
    }

    #[test]
    fn i64_required_space_negative_one_equals_positive_one() {
        assert_eq!((-1i64).required_space(), 1i64.required_space());
    }
}
