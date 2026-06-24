use crate::base::{
    encode::{ZigZag, U256},
    scalar::MontScalar,
};
use ark_ff::MontConfig;
use core::cmp::{max, Ordering};

/// This function writes the input scalar x as a varint encoding to buf slice
///
/// See `<https://developers.google.com/protocol-buffers/docs/encoding#varints>` as reference.
///
/// return:
/// - the total number of bytes N written to buf
///
/// crash:
/// - in case N is bigger than `buf.len()`
pub fn write_scalar_varint<T: MontConfig<4>>(buf: &mut [u8], x: &MontScalar<T>) -> usize {
    write_u256_varint(buf, x.zigzag())
}

#[expect(clippy::cast_possible_truncation)]
pub fn write_u256_varint(buf: &mut [u8], mut zig_x: U256) -> usize {
    let mut pos = 0;

    // we keep writing until we get a value that has the MSB not set.
    // a MSB not set implies that we have reached the end of the number.
    while zig_x.high != 0 || zig_x.low >= 0b1000_0000 {
        // we read the next 7 bits from `zig_x` casting to u8 and setting
        // the 8-th bit to 1 to indicate that we still need to write more bytes to buf
        buf[pos] = (zig_x.low as u8) | 0b1000_0000;
        pos += 1;

        // we shift the whole `zig_x` number 7 bits to right
        zig_x.low = (zig_x.low >> 7) | ((zig_x.high & 0b0111_1111) << 121);
        zig_x.high >>= 7;
    }

    // we write the last byte to buf with the MSB not set.
    // that indicates that the number has no continuation.
    buf[pos] = (zig_x.low & 0b0111_1111) as u8;

    pos + 1
}

/// This function consumes the N first byte elements from buf slice
/// that have their MSB set plus 1 more byte that does not have the MSB set.
/// These consumed bytes must represent a varint encoded number. Effectively,
/// each byte can have up to 7-bit set associated with the encoded number,
/// besides MSB 1-bit to represent in which byte the encoding ends.
///
/// return `Some((value, read_bytes))`:
/// - `value` = the dalek scalar generated out of the consumed bytes
/// - `read_bytes` = the total number of bytes N consumed
///
/// return None:
/// - in case of more than 37 bytes are read
/// - in case of more bytes read than the buffer length
///
/// Note: because this function can read up to 37 bytes,
///  buf can represent a number with up to 37 * 7 bits = 259 bits.
///  Since read-scalar stores the buf into a U256 type, which can only
///  hold up to 256 bit numbers, the non-continuation bits
///  257 up to 259 from buf are ignored.
pub fn read_scalar_varint<T: MontConfig<4>>(buf: &[u8]) -> Option<(MontScalar<T>, usize)> {
    read_u256_varint(buf).map(|(val, s)| (val.zigzag(), s))
}
pub fn read_u256_varint(buf: &[u8]) -> Option<(U256, usize)> {
    // The decoded value representing a u256 integer
    let mut val = U256::from_words(0, 0);

    // The number of bits to shift by (<<0, <<7, <<14, etc)
    let mut shift_amount: u32 = 0;

    // we keep reading until we find a byte with the MSB equal to zero,
    // which implies that we have read the whole varint number
    for next_byte in buf {
        // we write the `next 7 bits` at the [shift_amount..shift_amount + 7)
        // bit positions of val u256 number
        match shift_amount.cmp(&126_u32) {
            Ordering::Less => val.low |= (u128::from(*next_byte & 0b0111_1111)) << shift_amount,
            Ordering::Equal => {
                val.low |= (u128::from(*next_byte & 0b0000_0011)) << shift_amount;
                val.high |= (u128::from(*next_byte & 0b0111_1100)) >> 2;
            }
            Ordering::Greater => {
                val.high |= (u128::from(*next_byte & 0b0111_1111)) << (shift_amount - 128);
            }
        }

        shift_amount += 7;

        if (*next_byte >> 7) == 0 {
            // check if we have reached the end of the encoding (MSB not set)
            return Some((val, (shift_amount / 7) as usize));
        }

        if shift_amount > 256 {
            // the dalek scalar can only support 256 bits
            return None;
        }
    }

    // we read all the bytes in buf, but couldn't reach the end of the varint encoding
    None
}

/// This function writes all the input scalars `scals` to the input buffer `buf`.
/// For that, the Varint together with the [`ZigZag`] encoding is used.
///
/// return:
/// - the total number of bytes written to buf
///
/// error:
/// - in case buf has not enough space to hold all the scalars encoding.
#[cfg(test)]
pub fn write_scalar_varints<T: MontConfig<4>>(buf: &mut [u8], scals: &[MontScalar<T>]) -> usize {
    let mut total_bytes_written = 0;

    for scal in scals {
        let bytes_written = write_scalar_varint(&mut buf[total_bytes_written..], scal);

        total_bytes_written += bytes_written;
    }

    total_bytes_written
}

/// This function read all the specified scalars from `input_buf` to `scals_buf`.
/// For that, it converts the input buffer from a Varint and [`ZigZag`] encoding to a Dalek Scalar
///
/// See `<https://developers.google.com/protocol-buffers/docs/encoding#varints>` as reference.
///
/// error:
/// - in case it's not possible to read all specified scalars from `input_buf`
#[cfg(test)]
pub fn read_scalar_varints<T: MontConfig<4>>(
    scals_buf: &mut [MontScalar<T>],
    input_buf: &[u8],
) -> Option<()> {
    let mut buf = input_buf;

    for scal_buf in scals_buf.iter_mut() {
        let (scal, bytes_read) = read_scalar_varint(buf)?;

        *scal_buf = scal;
        buf = &buf[bytes_read..];
    }

    Some(())
}

/// This function returns the varint encoding size for the given scalar
///
/// This function should be used to get an upper bound on the buffer size
/// used by the `write_scalar_varint` function.
pub fn scalar_varint_size<T: MontConfig<4>>(x: &MontScalar<T>) -> usize {
    u256_varint_size(x.zigzag())
}
pub fn u256_varint_size(zig_x: U256) -> usize {
    let zigzag_size = if zig_x.high == 0 {
        128 - zig_x.low.leading_zeros()
    } else {
        256 - zig_x.high.leading_zeros()
    };

    // we must at least return 1. because even for
    // the 0 scalar case, we need one byte for the encoding
    max(1, (zigzag_size as usize).div_ceil(7))
}

/// This function returns the varint encoding size for the given scalar slice
///
/// This function should be used to get an upper bound on the buffer size
/// used by the `write_scalar_varints` function.
#[cfg(test)]
pub fn scalar_varints_size<T: MontConfig<4>>(scals: &[MontScalar<T>]) -> usize {
    let mut all_size: usize = 0;

    for x in scals {
        all_size += scalar_varint_size(x);
    }

    all_size
}

#[cfg(test)]
mod tests {
    use super::{
        read_scalar_varint, read_scalar_varints, read_u256_varint, scalar_varint_size,
        scalar_varints_size, u256_varint_size, write_scalar_varint, write_scalar_varints,
        write_u256_varint,
    };
    use crate::base::{encode::U256, scalar::test_scalar::TestScalar};
    use alloc::vec;

    #[test]
    fn zero_scalar_roundtrip() {
        let scalar = TestScalar::from(0u64);
        let mut buf = vec![0u8; 40];
        let written = write_scalar_varint(&mut buf, &scalar);
        let (decoded, read) = read_scalar_varint::<_>(&buf[..written]).unwrap();
        assert_eq!(decoded, scalar);
        assert_eq!(read, written);
    }

    #[test]
    fn positive_scalar_roundtrip() {
        let scalar = TestScalar::from(42u64);
        let mut buf = vec![0u8; 40];
        let written = write_scalar_varint(&mut buf, &scalar);
        let (decoded, _) = read_scalar_varint::<_>(&buf[..written]).unwrap();
        assert_eq!(decoded, scalar);
    }

    #[test]
    fn large_scalar_roundtrip() {
        let scalar = TestScalar::from(1000000u64);
        let mut buf = vec![0u8; 40];
        let written = write_scalar_varint(&mut buf, &scalar);
        let (decoded, _) = read_scalar_varint::<_>(&buf[..written]).unwrap();
        assert_eq!(decoded, scalar);
    }

    #[test]
    fn negative_scalar_roundtrip() {
        let scalar = -TestScalar::from(1u64);
        let mut buf = vec![0u8; 40];
        let written = write_scalar_varint(&mut buf, &scalar);
        let (decoded, _) = read_scalar_varint::<_>(&buf[..written]).unwrap();
        assert_eq!(decoded, scalar);
    }

    #[test]
    fn negative_large_scalar_roundtrip() {
        let scalar = -TestScalar::from(99999u64);
        let mut buf = vec![0u8; 40];
        let written = write_scalar_varint(&mut buf, &scalar);
        let (decoded, _) = read_scalar_varint::<_>(&buf[..written]).unwrap();
        assert_eq!(decoded, scalar);
    }

    #[test]
    fn scalar_varint_size_for_zero_is_one() {
        let scalar = TestScalar::from(0u64);
        assert_eq!(scalar_varint_size(&scalar), 1);
    }

    #[test]
    fn scalar_varint_size_for_one_is_one() {
        let scalar = TestScalar::from(1u64);
        assert_eq!(scalar_varint_size(&scalar), 1);
    }

    #[test]
    fn scalar_varint_size_for_negative_one_is_one() {
        let scalar = -TestScalar::from(1u64);
        assert_eq!(scalar_varint_size(&scalar), 1);
    }

    #[test]
    fn scalar_varint_size_increases_with_magnitude() {
        let small = TestScalar::from(1u64);
        let large = TestScalar::from(1000000u64);
        assert!(scalar_varint_size(&large) >= scalar_varint_size(&small));
    }

    #[test]
    fn write_bytes_equals_varint_size() {
        let scalar = TestScalar::from(12345u64);
        let mut buf = vec![0u8; 40];
        let written = write_scalar_varint(&mut buf, &scalar);
        assert_eq!(written, scalar_varint_size(&scalar));
    }

    #[test]
    fn u256_zero_roundtrip() {
        let val = U256::from_words(0, 0);
        let mut buf = vec![0u8; 40];
        let written = write_u256_varint(&mut buf, val);
        let (decoded, read) = read_u256_varint(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(read, written);
    }

    #[test]
    fn u256_small_value_roundtrip() {
        let val = U256::from_words(42, 0);
        let mut buf = vec![0u8; 40];
        let written = write_u256_varint(&mut buf, val);
        let (decoded, _) = read_u256_varint(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u256_large_low_word_roundtrip() {
        let val = U256::from_words(u128::MAX, 0);
        let mut buf = vec![0u8; 40];
        let written = write_u256_varint(&mut buf, val);
        let (decoded, _) = read_u256_varint(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u256_high_word_roundtrip() {
        let val = U256::from_words(0, 12345);
        let mut buf = vec![0u8; 40];
        let written = write_u256_varint(&mut buf, val);
        let (decoded, _) = read_u256_varint(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
    }

    #[test]
    fn u256_varint_size_for_zero_is_one() {
        assert_eq!(u256_varint_size(U256::from_words(0, 0)), 1);
    }

    #[test]
    fn u256_varint_size_for_128_is_two() {
        // 128 requires 8 bits, ceil(8/7) = 2 bytes
        assert_eq!(u256_varint_size(U256::from_words(128, 0)), 2);
    }

    #[test]
    fn u256_varint_size_matches_written_bytes() {
        let val = U256::from_words(9999, 0);
        let mut buf = vec![0u8; 40];
        let written = write_u256_varint(&mut buf, val);
        assert_eq!(written, u256_varint_size(val));
    }

    #[test]
    fn read_u256_varint_returns_none_on_empty_buf() {
        assert_eq!(read_u256_varint(&[]), None);
    }

    #[test]
    fn multiple_scalars_roundtrip() {
        let scalars = alloc::vec![
            TestScalar::from(1u64),
            TestScalar::from(100u64),
            -TestScalar::from(5u64),
            TestScalar::from(0u64),
        ];
        let total_size = scalar_varints_size(&scalars);
        let mut buf = vec![0u8; total_size];
        let written = write_scalar_varints(&mut buf, &scalars);
        assert_eq!(written, total_size);
        let mut decoded = vec![TestScalar::from(0u64); 4];
        read_scalar_varints(&mut decoded, &buf).unwrap();
        assert_eq!(decoded, scalars);
    }

    #[test]
    fn scalar_varints_size_empty_is_zero() {
        let empty: alloc::vec::Vec<TestScalar> = alloc::vec![];
        assert_eq!(scalar_varints_size(&empty), 0);
    }

    #[test]
    fn scalar_varints_size_single() {
        let s = TestScalar::from(42u64);
        let scalars = alloc::vec![s];
        assert_eq!(scalar_varints_size(&scalars), scalar_varint_size(&s));
    }

    #[test]
    fn scalar_varints_size_sum_of_individual() {
        let scalars = alloc::vec![TestScalar::from(1u64), TestScalar::from(1000u64)];
        let expected: usize = scalars.iter().map(scalar_varint_size).sum();
        assert_eq!(scalar_varints_size(&scalars), expected);
    }

    #[test]
    fn u256_roundtrip_full_128_bit_high() {
        let val = U256::from_words(1, u128::MAX);
        let mut buf = vec![0u8; 40];
        let written = write_u256_varint(&mut buf, val);
        let (decoded, _) = read_u256_varint(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
    }
}
