use crate::base::{
    encode::U256,
    scalar::test_scalar::TestScalar,
};
use core::fmt::Debug;

/// 测试 U256::from_words 构造函数
#[test]
fn from_words_creates_u256_with_correct_low_and_high() {
    let val = U256::from_words(0, 0);
    assert_eq!(val.low, 0);
    assert_eq!(val.high, 0);

    let val = U256::from_words(1, 0);
    assert_eq!(val.low, 1);
    assert_eq!(val.high, 0);

    let val = U256::from_words(0, 1);
    assert_eq!(val.low, 0);
    assert_eq!(val.high, 1);

    let val = U256::from_words(u128::MAX, u128::MAX);
    assert_eq!(val.low, u128::MAX);
    assert_eq!(val.high, u128::MAX);
}

/// 测试 U256 的 PartialEq 实现
#[test]
fn u256_equality_works_correctly() {
    assert_eq!(U256::from_words(0, 0), U256::from_words(0, 0));
    assert_eq!(U256::from_words(1, 2), U256::from_words(1, 2));
    assert_ne!(U256::from_words(1, 0), U256::from_words(0, 0));
    assert_ne!(U256::from_words(0, 1), U256::from_words(0, 0));
    assert_ne!(U256::from_words(1, 0), U256::from_words(0, 1));
}

/// 测试 U256 的 Clone 实现
#[test]
fn u256_clone_works_correctly() {
    let original = U256::from_words(0x1234_5678_9abc_def0, 0xfedc_ba98_7654_3210);
    let cloned = original;
    assert_eq!(original, cloned);
}

/// 测试 U256 的 Copy 实现
#[test]
fn u256_copy_works_correctly() {
    let original = U256::from_words(0xaaaa_bbbb_cccc_dddd, 0x1111_2222_3333_4444);
    let copied = original;
    assert_eq!(original, copied);
}

/// 测试 From<&MontScalar<T>> for U256 — 零值转换
#[test]
fn u256_from_zero_scalar() {
    let scalar = TestScalar::from(0_u64);
    let u256_val: U256 = (&scalar).into();
    assert_eq!(u256_val.low, 0);
    assert_eq!(u256_val.high, 0);
}

/// 测试 From<&MontScalar<T>> for U256 — 小正整数转换
#[test]
fn u256_from_small_positive_scalar() {
    let scalar = TestScalar::from(1_u64);
    let u256_val: U256 = (&scalar).into();
    assert_eq!(u256_val.low, 1);
    assert_eq!(u256_val.high, 0);

    let scalar = TestScalar::from(255_u64);
    let u256_val: U256 = (&scalar).into();
    assert_eq!(u256_val.low, 255);
    assert_eq!(u256_val.high, 0);
}

/// 测试 From<&MontScalar<T>> for U256 — 大正整数转换（跨越u64范围）
#[test]
fn u256_from_large_positive_scalar() {
    let scalar = TestScalar::from(u128::MAX);
    let u256_val: U256 = (&scalar).into();
    assert_eq!(u256_val.low, u128::MAX);
    assert_eq!(u256_val.high, 0);
}

/// 测试 From<&U256> for MontScalar<T> — 零值转换
#[test]
fn scalar_from_zero_u256() {
    let u256_val = U256::from_words(0, 0);
    let scalar: TestScalar = (&u256_val).into();
    assert_eq!(scalar, TestScalar::from(0_u64));
}

/// 测试 From<&U256> for MontScalar<T> — 小正整数转换
#[test]
fn scalar_from_small_positive_u256() {
    let u256_val = U256::from_words(42, 0);
    let scalar: TestScalar = (&u256_val).into();
    assert_eq!(scalar, TestScalar::from(42_u64));
}

/// 测试 From<&U256> for MontScalar<T> — 大值转换（high非零）
#[test]
fn scalar_from_u256_with_nonzero_high() {
    let u256_val = U256::from_words(0, 1);
    let scalar: TestScalar = (&u256_val).into();
    // U256 { low: 0, high: 1 } = 2^128，通过 from_le_bytes_mod_order 转换
    // 验证 round-trip：转回 U256 应该得到相同的值
    let back_to_u256: U256 = (&scalar).into();
    assert_eq!(back_to_u256, u256_val);
}

/// 测试 MontScalar ↔ U256 往返转换的一致性
#[test]
fn round_trip_conversion_between_scalar_and_u256_preserves_value() {
    // 测试多个值
    for val in [0_u64, 1, 127, 128, 255, 256, 1000, u64::MAX] {
        let scalar = TestScalar::from(val);
        let u256_val: U256 = (&scalar).into();
        let back: TestScalar = (&u256_val).into();
        assert_eq!(back, scalar, "round-trip failed for value {val}");
    }

    // u128 范围的值
    let scalar = TestScalar::from(u128::MAX);
    let u256_val: U256 = (&scalar).into();
    let back: TestScalar = (&u256_val).into();
    assert_eq!(back, scalar);
}

/// 测试 U256 的 from_words 是 const fn
#[test]
fn from_words_can_be_used_in_const_context() {
    const ZERO: U256 = U256::from_words(0, 0);
    assert_eq!(ZERO.low, 0);
    assert_eq!(ZERO.high, 0);

    const MAX: U256 = U256::from_words(u128::MAX, u128::MAX);
    assert_eq!(MAX.low, u128::MAX);
    assert_eq!(MAX.high, u128::MAX);
}

/// 测试 From<&MontScalar<T>> for U256 — 负标量转换
#[test]
fn u256_from_negative_scalar() {
    // -1 在标量域中等于 p - 1
    let neg_one = -TestScalar::from(1_u64);
    let u256_val: U256 = (&neg_one).into();
    // p - 1 的 low 和 high 都应该非零
    assert_ne!(u256_val.low, 0);
    assert_ne!(u256_val.high, 0);
}

/// 测试 U256 公共字段的直接访问
#[test]
fn u256_public_fields_are_accessible() {
    let mut val = U256::from_words(0, 0);
    val.low = 100;
    val.high = 200;
    assert_eq!(val.low, 100);
    assert_eq!(val.high, 200);
}
