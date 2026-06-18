use super::VarInt;
use crate::base::scalar::{test_scalar::TestScalar, Scalar};
use alloc::{vec, vec::Vec};
use core::{
    fmt::Debug,
    ops::{Add, Neg},
};
use num_traits::{One, Zero};
use rand::Rng;

/**
 * Adapted from integer-encoding-rs
 *
 * See third_party/license/integer-encoding.LICENSE
 */

// -----------------------------------------------------------------------------------------------------
// The following tests are taken directly from integer-encoding-rs with minimal modification
// -----------------------------------------------------------------------------------------------------

#[test]
fn test_required_space() {
    assert_eq!(0_u32.required_space(), 1);
    assert_eq!(1_u32.required_space(), 1);
    assert_eq!(128_u32.required_space(), 2);
    assert_eq!(16384_u32.required_space(), 3);
    assert_eq!(2_097_151_u32.required_space(), 3);
    assert_eq!(2_097_152_u32.required_space(), 4);
}

#[test]
fn test_encode_u64() {
    assert_eq!(0_u32.encode_var_vec(), vec![0b0000_0000]);
    assert_eq!(300_u32.encode_var_vec(), vec![0b1010_1100, 0b0000_0010]);
}

#[test]
fn test_identity_u64() {
    for i in 1_u64..100 {
        assert_eq!(
            u64::decode_var(i.encode_var_vec().as_slice()).unwrap(),
            (i, 1)
        );
    }
    for i in 16400_u64..16500 {
        assert_eq!(
            u64::decode_var(i.encode_var_vec().as_slice()).unwrap(),
            (i, 3)
        );
    }
}

#[test]
fn test_decode_max_u64() {
    let max_vec_encoded = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        u64::decode_var(max_vec_encoded.as_slice()).unwrap().0,
        u64::MAX
    );
}

#[test]
fn test_encode_i64() {
    assert_eq!(0_i64.encode_var_vec(), 0_u32.encode_var_vec());
    assert_eq!(150_i64.encode_var_vec(), 300_u32.encode_var_vec());
    assert_eq!((-150_i64).encode_var_vec(), 299_u32.encode_var_vec());
    assert_eq!(
        (-2_147_483_648_i64).encode_var_vec(),
        4_294_967_295_u64.encode_var_vec()
    );
    assert_eq!(
        i64::MAX.encode_var_vec(),
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
    assert_eq!(
        i64::MIN.encode_var_vec(),
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
}

#[test]
fn test_decode_min_i64() {
    let min_vec_encoded = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        i64::decode_var(min_vec_encoded.as_slice()).unwrap().0,
        i64::MIN
    );
}

#[test]
fn test_decode_max_i64() {
    let max_vec_encoded = vec![0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        i64::decode_var(max_vec_encoded.as_slice()).unwrap().0,
        i64::MAX
    );
}

#[test]
fn test_encode_i16() {
    assert_eq!(150_i16.encode_var_vec(), 300_u32.encode_var_vec());
    assert_eq!((-150_i16).encode_var_vec(), 299_u32.encode_var_vec());
}

#[test]
fn test_unterminated_varint() {
    let buf = vec![0xff_u8; 12];
    assert!(u64::decode_var(&buf).is_none());
}

#[test]
fn test_unterminated_varint_2() {
    let buf = [0xff, 0xff];
    assert!(u64::decode_var(&buf).is_none());
}

#[test]
fn test_decode_extra_bytes_u64() {
    let mut encoded = 0x12345u64.encode_var_vec();
    assert_eq!(u64::decode_var(&encoded[..]), Some((0x12345, 3)));

    encoded.push(0x99);
    assert_eq!(u64::decode_var(&encoded[..]), Some((0x12345, 3)));

    let encoded = [0xFF, 0xFF, 0xFF];
    assert_eq!(u64::decode_var(&encoded[..]), None);

    // Overflow
    let mut encoded = vec![0xFF; 64];
    encoded.push(0x00);
    assert_eq!(u64::decode_var(&encoded[..]), None);
}

#[test]
fn test_decode_extra_bytes_i64() {
    let mut encoded = (-0x12345i64).encode_var_vec();
    assert_eq!(i64::decode_var(&encoded[..]), Some((-0x12345, 3)));

    encoded.push(0x99);
    assert_eq!(i64::decode_var(&encoded[..]), Some((-0x12345, 3)));

    let encoded = [0xFF, 0xFF, 0xFF];
    assert_eq!(i64::decode_var(&encoded[..]), None);

    // Overflow
    let mut encoded = vec![0xFF; 64];
    encoded.push(0x00);
    assert_eq!(i64::decode_var(&encoded[..]), None);
}

#[test]
fn test_regression_22() {
    let encoded: Vec<u8> = 0x0011_2233_u64.encode_var_vec();
    assert!(i8::decode_var(&encoded).is_none());
}

// ------------------------------------------------------------------------------
// End of tests taken directly from integer-encoding-rs with minimal modification
// ------------------------------------------------------------------------------

// ------------------
// VarInt trait tests
// ------------------

pub(super) fn test_encode_decode<T: VarInt + PartialEq + Debug, const N: usize>(
    val: T,
    encoded: [u8; N],
) {
    let result: &mut [u8] = &mut [0; N];
    assert_eq!(val.required_space(), N);
    assert_eq!(val.encode_var(result), N);
    assert_eq!(result, encoded);
    assert_eq!((val, N), T::decode_var(result).unwrap());
}

fn test_small_unsigned_values_encode_and_decode_properly<
    T: VarInt + Zero + One + Add + PartialEq + Debug,
>() {
    test_encode_decode(T::zero(), [0]);
    test_encode_decode(T::one(), [1]);
    test_encode_decode(T::one() + T::one(), [2]);
    test_encode_decode(T::one() + T::one() + T::one(), [3]);
}

pub(super) fn test_small_signed_values_encode_and_decode_properly<T>(one: T)
where
    T: VarInt + Add<Output = T> + PartialEq + Debug + Neg<Output = T>,
{
    test_encode_decode(one + (-one), [0]);
    test_encode_decode(-one, [1]);
    test_encode_decode(one, [2]);
    test_encode_decode(-(one + one), [3]);
    test_encode_decode(one + one, [4]);
    test_encode_decode(-(one + one + one), [5]);
    test_encode_decode(one + one + one, [6]);
}

pub(super) fn test_encode_and_decode_types_align<Small, Large>(
    align_tests: &[Small],
    too_large_tests: &[Large],
    buffer_size: usize,
) where
    Small: VarInt + Into<Large>,
    Large: VarInt + PartialEq + Debug,
{
    for &val_small in align_tests {
        let val_large: Large = val_small.into();
        let mut result_small = vec![0u8; buffer_size];
        let mut result_large = vec![0u8; buffer_size];
        assert_eq!(
            val_small.encode_var(&mut result_small),
            val_large.encode_var(&mut result_large)
        );
        assert_eq!(result_small, result_large);
        let decode_small = Small::decode_var(&result_small);
        let decode_large = Large::decode_var(&result_small);
        assert_eq!(decode_small.map(|(v, s)| (v.into(), s)), decode_large);
    }

    for too_large in too_large_tests {
        let mut buffer = vec![0u8; buffer_size];
        too_large.encode_var(&mut buffer);
        let decode_small = Small::decode_var(&buffer);
        let decode_large = Large::decode_var(&buffer);
        assert!(decode_small.is_none());
        assert!(decode_large.is_some());
    }
}

#[test]
fn we_can_encode_and_decode_small_i64_values() {
    test_small_signed_values_encode_and_decode_properly::<i64>(1);
}

#[test]
fn we_can_encode_and_decode_small_u64_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u64>();
}

#[test]
fn we_can_encode_and_decode_large_u64_values() {
    test_encode_decode(
        u64::MAX,
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
    assert!(
        u64::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]).is_none()
    );
}
#[test]
fn we_can_encode_and_decode_large_i64_values() {
    test_encode_decode(
        i64::MAX,
        [0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
    test_encode_decode(
        i64::MIN,
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
    assert!(
        i64::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]).is_none()
    );
    assert!(
        i64::decode_var(&[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]).is_none()
    );
}

#[test]
fn we_can_encode_and_decode_small_i32_values() {
    test_small_signed_values_encode_and_decode_properly::<i32>(1);
}

#[test]
fn we_can_encode_and_decode_small_u32_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u32>();
}

#[test]
fn we_can_encode_and_decode_large_u32_values() {
    test_encode_decode(u32::MAX, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    assert!(u32::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x20]).is_none());
}
#[test]
fn we_can_encode_and_decode_large_i32_values() {
    test_encode_decode(i32::MAX, [0xFE, 0xFF, 0xFF, 0xFF, 0x0F]);
    test_encode_decode(i32::MIN, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    assert!(i32::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x10]).is_none());
    assert!(i32::decode_var(&[0x81, 0x80, 0x80, 0x80, 0x10]).is_none());
}

#[test]
fn we_can_encode_and_decode_i32_and_i64_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<i32, i64>(
        &rng.gen::<[_; 32]>(),
        &[
            i64::from(i32::MAX) + 1,
            i64::from(i32::MIN) - 1,
            i64::from(i32::MAX) * 1000,
            i64::from(i32::MIN) * 1000,
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_u32_and_u64_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<u32, u64>(
        &rng.gen::<[_; 32]>(),
        &[u64::from(u32::MAX) + 1, u64::from(u32::MAX) * 1000],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_large_positive_u128() {
    let value: u128 =
        0b110_0010101_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_0011100;
    let expected_result: &[u8] = &[
        0b1001_1100,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1001_0101,
        0b0000_0110,
    ];
    let result: &mut [u8] = &mut [0; 11];
    assert_eq!(value.required_space(), 11);
    value.encode_var(result);
    assert_eq!(expected_result, result);
    assert_eq!((value, 11), u128::decode_var(result).unwrap());
}

#[test]
fn we_can_encode_and_decode_large_positive_i128() {
    #[expect(clippy::unusual_byte_groupings)]
    let value: i128 =
        0b110_0010101_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_001110;
    let expected_result: &[u8] = &[
        0b1001_1100,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1001_0101,
        0b0000_0110,
    ];
    let result: &mut [u8] = &mut [0; 11];
    assert_eq!(value.required_space(), 11);
    value.encode_var(result);
    assert_eq!(expected_result, result);
    assert_eq!((value, 11), i128::decode_var(result).unwrap());
}

#[test]
fn we_can_encode_and_decode_large_negative_i128() {
    #[expect(clippy::unusual_byte_groupings)]
    let value: i128 =
        -1 - 0b110_0010101_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_001110;
    let expected_result: &[u8] = &[
        0b1001_1101,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1111_1111,
        0b1001_0101,
        0b0000_0110,
    ];
    let result: &mut [u8] = &mut [0; 11];
    assert_eq!(value.required_space(), 11);
    value.encode_var(result);
    assert_eq!(expected_result, result);
    assert_eq!((value, 11), i128::decode_var(result).unwrap());
}
#[test]
fn we_can_encode_and_decode_small_i128_values() {
    test_small_signed_values_encode_and_decode_properly::<i128>(1);
}

#[test]
fn we_can_encode_and_decode_small_u128_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u128>();
}

#[test]
fn we_can_encode_and_decode_small_test_scalar_values() {
    test_small_signed_values_encode_and_decode_properly::<TestScalar>(TestScalar::ONE);
}

#[test]
fn we_can_encode_and_decode_i128_and_test_scalar_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<i128, TestScalar>(
        &rng.gen::<[_; 32]>(),
        &[
            TestScalar::from(i128::MAX) + TestScalar::one(),
            TestScalar::from(i128::MIN) - TestScalar::one(),
            TestScalar::from(i128::MAX) * TestScalar::from(1000),
            TestScalar::from(i128::MIN) * TestScalar::from(1000),
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_i64_and_i128_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<i64, i128>(
        &rng.gen::<[_; 32]>(),
        &[
            i128::from(i64::MAX) + 1,
            i128::from(i64::MIN) - 1,
            i128::from(i64::MAX) * 1000,
            i128::from(i64::MIN) * 1000,
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_u64_and_u128_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<u64, u128>(
        &rng.gen::<[_; 32]>(),
        &[u128::from(u64::MAX) + 1, u128::from(u64::MAX) * 1000],
        100,
    );
}

// ----------------------
// End VarInt trait tests
// ----------------------

// ==============================
// 新增测试：覆盖未测试的代码路径
// ==============================

/// 测试 bool::decode_var 对非法值（n 不是 0 或 1）返回 None
#[test]
fn bool_decode_var_returns_none_for_invalid_values() {
    // n = 2 应该返回 None
    let encoded = 2_u64.encode_var_vec();
    assert!(bool::decode_var(&encoded).is_none());

    // n = 100 应该返回 None
    let encoded = 100_u64.encode_var_vec();
    assert!(bool::decode_var(&encoded).is_none());

    // n = u64::MAX 应该返回 None
    let encoded = u64::MAX.encode_var_vec();
    assert!(bool::decode_var(&encoded).is_none());
}

/// 测试 bool::decode_var 对合法值（0 和 1）返回正确结果
#[test]
fn bool_decode_var_returns_correct_value_for_valid_inputs() {
    let encoded = 0_u64.encode_var_vec();
    assert_eq!(bool::decode_var(&encoded), Some((false, 1)));

    let encoded = 1_u64.encode_var_vec();
    assert_eq!(bool::decode_var(&encoded), Some((true, 1)));
}

/// 测试 bool::encode_var 编码
#[test]
fn bool_encode_var_works_correctly() {
    let mut buf = [0_u8; 1];
    let written = false.encode_var(&mut buf);
    assert_eq!(written, 1);
    assert_eq!(buf[0], 0);

    let mut buf = [0_u8; 1];
    let written = true.encode_var(&mut buf);
    assert_eq!(written, 1);
    assert_eq!(buf[0], 1);
}

/// 测试 bool::required_space
#[test]
fn bool_required_space_is_always_one() {
    assert_eq!(false.required_space(), 1);
    assert_eq!(true.required_space(), 1);
}

/// 测试 u128::decode_var 对 high != 0 的 U256 返回 None
#[test]
fn u128_decode_var_returns_none_when_high_is_nonzero() {
    // 编码一个 high != 0 的 U256 值
    let val = U256::from_words(0, 1);
    let mut buf = [0_u8; 20];
    let written = val.encode_var(&mut buf);
    // u128::decode_var 应该返回 None，因为 high != 0
    assert!(u128::decode_var(&buf[..written]).is_none());

    // 编码一个更大的值
    let val = U256::from_words(u128::MAX, u128::MAX);
    let mut buf = [0_u8; 40];
    let written = val.encode_var(&mut buf);
    assert!(u128::decode_var(&buf[..written]).is_none());
}

/// 测试 u128::decode_var 对合法值（high == 0）返回正确结果
#[test]
fn u128_decode_var_returns_correct_value_when_high_is_zero() {
    for val in [0_u128, 1, 127, 128, 255, 256, u64::MAX as u128, u128::MAX] {
        let mut buf = [0_u8; 20];
        let written = val.encode_var(&mut buf);
        let (decoded, read) = u128::decode_var(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(read, written);
    }
}

/// 测试 U256 VarInt 实现的 encode_var / decode_var
#[test]
fn u256_varint_encode_decode_round_trip() {
    let test_values = [
        U256::from_words(0, 0),
        U256::from_words(1, 0),
        U256::from_words(127, 0),
        U256::from_words(128, 0),
        U256::from_words(u128::MAX, 0),
        U256::from_words(0, 1),
        U256::from_words(1, 1),
        U256::from_words(u128::MAX, u128::MAX),
    ];

    for val in test_values {
        let mut buf = [0_u8; 40];
        let written = val.encode_var(&mut buf);
        let (decoded, read) = U256::decode_var(&buf[..written]).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(read, written);
    }
}

/// 测试 U256::required_space
#[test]
fn u256_required_space_matches_encode_var_output() {
    let test_values = [
        U256::from_words(0, 0),
        U256::from_words(1, 0),
        U256::from_words(128, 0),
        U256::from_words(u128::MAX, 0),
        U256::from_words(0, 1),
        U256::from_words(u128::MAX, u128::MAX),
    ];

    for val in test_values {
        let mut buf = [0_u8; 40];
        let written = val.encode_var(&mut buf);
        assert_eq!(val.required_space(), written);
    }
}

/// 测试 u8 的 VarInt 实现
#[test]
fn we_can_encode_and_decode_small_u8_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u8>();
}

#[test]
fn we_can_encode_and_decode_large_u8_values() {
    test_encode_decode(u8::MAX, [0xFF, 0x01]);
    // u8 溢出：编码一个大于 u8::MAX 的值，u8::decode_var 应返回 None
    let encoded = (u8::MAX as u64 + 1).encode_var_vec();
    assert!(u8::decode_var(&encoded).is_none());
}

/// 测试 u16 的 VarInt 实现
#[test]
fn we_can_encode_and_decode_small_u16_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u16>();
}

#[test]
fn we_can_encode_and_decode_large_u16_values() {
    test_encode_decode(u16::MAX, [0xFF, 0xFF, 0x03]);
    // u16 溢出
    let encoded = (u16::MAX as u64 + 1).encode_var_vec();
    assert!(u16::decode_var(&encoded).is_none());
}

/// 测试 i8 的 VarInt 实现
#[test]
fn we_can_encode_and_decode_small_i8_values() {
    test_small_signed_values_encode_and_decode_properly::<i8>(1);
}

#[test]
fn we_can_encode_and_decode_large_i8_values() {
    test_encode_decode(i8::MAX, [0xFE, 0x01]);
    test_encode_decode(i8::MIN, [0xFF, 0x01]);
    // i8 溢出
    let encoded = ((i8::MAX as i64) + 1).encode_var_vec();
    assert!(i8::decode_var(&encoded).is_none());
    let encoded = ((i8::MIN as i64) - 1).encode_var_vec();
    assert!(i8::decode_var(&encoded).is_none());
}

/// 测试 i16 的 VarInt 实现（扩展边界条件）
#[test]
fn we_can_encode_and_decode_large_i16_values() {
    test_encode_decode(i16::MAX, [0xFE, 0xFF, 0x03]);
    test_encode_decode(i16::MIN, [0xFF, 0xFF, 0x03]);
    // i16 溢出
    let encoded = ((i16::MAX as i64) + 1).encode_var_vec();
    assert!(i16::decode_var(&encoded).is_none());
    let encoded = ((i16::MIN as i64) - 1).encode_var_vec();
    assert!(i16::decode_var(&encoded).is_none());
}

/// 测试 isize 的 VarInt 实现
#[test]
fn we_can_encode_and_decode_small_isize_values() {
    test_small_signed_values_encode_and_decode_properly::<isize>(1);
}

/// 测试 u64::decode_var 对空 buffer 返回 None
#[test]
fn u64_decode_var_returns_none_for_empty_buffer() {
    let buf: [u8; 0] = [];
    assert!(u64::decode_var(&buf).is_none());
}

/// 测试 u64::decode_var 对只有 MSB=1 的单字节返回 None
#[test]
fn u64_decode_var_returns_none_for_single_byte_with_msb_set() {
    let buf = [0x80_u8];
    assert!(u64::decode_var(&buf).is_none());
}

/// 测试 i128 的 encode/decode 与 i64 的对齐性
#[test]
fn we_can_encode_and_decode_i64_and_i128_the_same_for_small_values() {
    for val in [-100_i64, -1, 0, 1, 100] {
        let mut buf_small = vec![0u8; 10];
        let mut buf_large = vec![0u8; 10];
        let written_small = val.encode_var(&mut buf_small);
        let written_large = (val as i128).encode_var(&mut buf_large);
        assert_eq!(written_small, written_large);
        assert_eq!(&buf_small[..written_small], &buf_large[..written_large]);
    }
}

/// 测试 u128 的 encode/decode 与 u64 的对齐性
#[test]
fn we_can_encode_and_decode_u64_and_u128_the_same_for_small_values() {
    for val in [0_u64, 1, 127, 128, 255, u64::MAX] {
        let mut buf_small = vec![0u8; 10];
        let mut buf_large = vec![0u8; 10];
        let written_small = val.encode_var(&mut buf_small);
        let written_large = (val as u128).encode_var(&mut buf_large);
        assert_eq!(written_small, written_large);
        assert_eq!(&buf_small[..written_small], &buf_large[..written_large]);
    }
}
