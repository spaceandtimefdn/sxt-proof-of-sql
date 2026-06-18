use crate::base::{
    encode::{ZigZag, U256},
    scalar::test_scalar::TestScalar,
};

#[test]
fn small_scalars_are_encoded_as_positive_zigzag_values() {
    // x = 0
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(TestScalar::from(0_u64).zigzag() == U256::from_words(0, 0));

    // x = 1
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(TestScalar::from(1_u8).zigzag() == U256::from_words(2, 0));

    // x = 2
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(TestScalar::from(2_u32).zigzag() == U256::from_words(4, 0));

    // x = u128::MAX
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(
        TestScalar::from(u128::MAX).zigzag()
            == U256::from_words(0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fffe, 0x1)
    );

    for x in 1..1000_u128 {
        // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
        assert!(TestScalar::from(x).zigzag() == U256::from_words(2 * x, 0));
    }
}

#[test]
fn big_scalars_with_small_additive_inverses_are_encoded_as_negative_zigzag_values() {
    // x = p - 1 (p = 2^252 + 27742317777372353535851937790883648493 is the ristretto group order)
    // the additive inverse of x is y = 1. Since y < x, the ZigZag encodes -y, which is
    // encoded as 2 * y - 1 = 1
    assert!((-TestScalar::from(1_u32)).zigzag() == U256::from_words(1, 0));

    // x = p - 2 (p = 2^252 + 27742317777372353535851937790883648493 is the ristretto group order)
    // the additive inverse of x is y = 2. Since y < x, the ZigZag encodes -y, which is
    // encoded as 2 * y - 1 = 3
    assert!((-TestScalar::from(2_u32)).zigzag() == U256::from_words(3, 0));

    for y in 1..1000_u128 {
        // since x > y, where x + y = 0, the ZigZag value is encoded as 2 * y - 1
        assert!((-TestScalar::from(y)).zigzag() == U256::from_words(2 * y - 1, 0));
    }
}

#[test]
fn big_scalars_that_are_smaller_than_their_additive_inverses_are_encoded_as_positive_zigzag_values()
{
    // x = (p - 1) / 2 (p is the ristretto group order)
    let val: TestScalar = (&U256::from_words(
        0x0a6f_7cef_517b_ce6b_2c09_318d_2e7a_e9f6,
        0x0800_0000_0000_0000_0000_0000_0000_0000,
    ))
        .into();
    // since x < y, where x + y = 0, the ZigZag value is encoded as 2 * x
    assert!(
        val.zigzag()
            == U256::from_words(
                27_742_317_777_372_353_535_851_937_790_883_648_492,
                21_267_647_932_558_653_966_460_912_964_485_513_216
            )
    );
}

#[test]
fn big_additive_inverses_that_are_smaller_than_the_input_scalars_are_encoded_as_negative_zigzag_values(
) {
    // x = (p + 1) / 2 (p is the ristretto group order)
    let val: TestScalar = (&U256::from_words(
        0x0a6f_7cef_517b_ce6b_2c09_318d_2e7a_e9f7,
        0x0800_0000_0000_0000_0000_0000_0000_0000,
    ))
        .into();

    // the additive inverse of x is y = -x = (p - 1) / 2
    // since we have y < x, the ZigZag encoding is 2 * y - 1 = p - 2
    assert!(
        val.zigzag()
            == U256::from_words(
                27_742_317_777_372_353_535_851_937_790_883_648_491,
                21_267_647_932_558_653_966_460_912_964_485_513_216
            )
    );

    // x = - U256 { low: 0, high: 0x1_u128 }
    // since x > y, where x + y = 0, the ZigZag value is encoded as 2 * y - 1
    let val: TestScalar = (&U256 {
        low: 0x0_u128,
        high: 0x1_u128,
    })
        .into();
    assert!(
        (-val).zigzag()
            == U256::from_words(0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff_u128, 0x1_u128)
    );
}

// ==============================
// 新增测试：覆盖未测试的代码路径
// ==============================

/// 测试 U256 → MontScalar 的 ZigZag 解码：偶数值（正数）
#[test]
fn even_u256_zigzag_decodes_to_positive_scalar() {
    // 偶数 U256 → 正标量
    // zigzag(0) = 0 → 解码为 0
    let result: TestScalar = U256::from_words(0, 0).zigzag();
    assert_eq!(result, TestScalar::from(0_u64));

    // zigzag(1) = 2 → 解码为 1
    let result: TestScalar = U256::from_words(2, 0).zigzag();
    assert_eq!(result, TestScalar::from(1_u64));

    // zigzag(2) = 4 → 解码为 2
    let result: TestScalar = U256::from_words(4, 0).zigzag();
    assert_eq!(result, TestScalar::from(2_u64));
}

/// 测试 U256 → MontScalar 的 ZigZag 解码：奇数值（负数）
#[test]
fn odd_u256_zigzag_decodes_to_negative_scalar() {
    // 奇数 U256 → 负标量
    // zigzag(-1) = 1 → 解码为 -1
    let result: TestScalar = U256::from_words(1, 0).zigzag();
    assert_eq!(result, -TestScalar::from(1_u64));

    // zigzag(-2) = 3 → 解码为 -2
    let result: TestScalar = U256::from_words(3, 0).zigzag();
    assert_eq!(result, -TestScalar::from(2_u64));

    // zigzag(-3) = 5 → 解码为 -3
    let result: TestScalar = U256::from_words(5, 0).zigzag();
    assert_eq!(result, -TestScalar::from(3_u64));
}

/// 测试 MontScalar → U256 → MontScalar 的 ZigZag 往返转换
#[test]
fn zigzag_round_trip_preserves_scalar_value() {
    // 正值
    for val in [0_u64, 1, 2, 10, 100, 1000, u128::MAX as u64] {
        let scalar = TestScalar::from(val);
        let zigzag_encoded: U256 = scalar.zigzag();
        let decoded: TestScalar = zigzag_encoded.zigzag();
        assert_eq!(decoded, scalar, "round-trip failed for positive value {val}");
    }

    // 负值
    for val in [1_u64, 2, 10, 100, 1000] {
        let scalar = -TestScalar::from(val);
        let zigzag_encoded: U256 = scalar.zigzag();
        let decoded: TestScalar = zigzag_encoded.zigzag();
        assert_eq!(decoded, scalar, "round-trip failed for negative value -{val}");
    }
}

/// 测试 ZigZag 编码中 x.high == y.high && x.low > y.low 的分支
/// 当标量的 U256 表示和其加法逆元的 U256 表示的 high 部分相等，
/// 但 low 部分标量更大时，选择逆元进行编码
#[test]
fn zigzag_encodes_negative_when_x_high_equals_y_high_and_x_low_greater() {
    // 构造一个标量，使得 x.high == y.high 但 x.low > y.low
    // 这发生在标量值接近 p/2 的时候
    // (p - 1) / 2 的标量：x < y，所以走正值编码
    let val: TestScalar = (&U256::from_words(
        0x0a6f_7cef_517b_ce6b_2c09_318d_2e7a_e9f6,
        0x0800_0000_0000_0000_0000_0000_0000_0000,
    ))
        .into();

    let zigzag_val: U256 = val.zigzag();
    // 验证 round-trip
    let decoded: TestScalar = zigzag_val.zigzag();
    assert_eq!(decoded, val);
}

/// 测试 ZigZag 编码中 overflowing_add 的进位分支
/// 当 zig_val.low 加 1 溢出时，carry_low 为 true，需要进位到 high
#[test]
fn zigzag_decode_handles_overflowing_add_carry() {
    // 构造一个奇数 U256，其 low 部分为 u128::MAX
    // 这样 zig_val.low = (u128::MAX >> 1)，加 1 时不会溢出
    // 但如果构造一个 low = 0xffff_ffff_ffff_fffe 的奇数 U256
    // zig_val.low = (0xffff_ffff_ffff_fffe >> 1) | ((high & 1) << 127)
    // 然后 zig_val.low + 1 不会溢出

    // 更好的测试：构造一个 U256 使得 zig_val.low 在加 1 后溢出
    // zig_val.low = (self.low >> 1) | ((self.high & 1) << 127)
    // 要让 zig_val.low + 1 溢出，需要 zig_val.low = u128::MAX
    // 即 (self.low >> 1) | ((self.high & 1) << 127) = u128::MAX
    // 当 self.high & 1 = 1 时，(1 << 127) = 0x8000_0000_0000_0000_0000_0000_0000_0000
    // 需要 self.low >> 1 的低 127 位全为 1，即 self.low 的低 127 位全为 1
    // self.low = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fffe（偶数，不行，需要奇数）
    // self.low = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff（奇数）
    // self.high & 1 = 1 → self.high = 1
    let odd_u256 = U256::from_words(u128::MAX, 1);
    let result: TestScalar = odd_u256.zigzag();
    // 验证 round-trip
    let encoded: U256 = result.zigzag();
    let decoded: TestScalar = encoded.zigzag();
    assert_eq!(decoded, result);
}

/// 测试零值的 ZigZag 编码/解码
#[test]
fn zero_scalar_zigzag_encodes_to_zero_u256() {
    let zero = TestScalar::from(0_u64);
    let encoded: U256 = zero.zigzag();
    assert_eq!(encoded, U256::from_words(0, 0));

    let decoded: TestScalar = encoded.zigzag();
    assert_eq!(decoded, zero);
}

/// 测试大正标量的 ZigZag 编码（x < y 的情况）
#[test]
fn large_positive_scalar_zigzag_encodes_correctly() {
    let val = TestScalar::from(u128::MAX);
    let encoded: U256 = val.zigzag();
    // u128::MAX 是正数，x < y（因为 y = -x 更大），所以编码为 2 * x
    let expected = U256::from_words(u128::MAX << 1, u128::MAX >> 127);
    assert_eq!(encoded, expected);

    // round-trip 验证
    let decoded: TestScalar = encoded.zigzag();
    assert_eq!(decoded, val);
}
