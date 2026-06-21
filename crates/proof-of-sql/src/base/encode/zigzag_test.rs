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

#[test]
fn even_zigzag_values_decode_to_positive_scalars() {
    let zero: TestScalar = U256::from_words(0, 0).zigzag();
    let one: TestScalar = U256::from_words(2, 0).zigzag();
    let two: TestScalar = U256::from_words(4, 0).zigzag();

    assert_eq!(zero, TestScalar::from(0_u64));
    assert_eq!(one, TestScalar::from(1_u64));
    assert_eq!(two, TestScalar::from(2_u64));

    let encoded = U256::from_words(0, 1);
    let decoded: TestScalar = encoded.zigzag();
    let expected: TestScalar = (&U256::from_words(1_u128 << 127, 0)).into();

    assert_eq!(decoded, expected);
}

#[test]
fn odd_zigzag_values_decode_to_negative_scalars() {
    let minus_one: TestScalar = U256::from_words(1, 0).zigzag();
    let minus_two: TestScalar = U256::from_words(3, 0).zigzag();
    let minus_three: TestScalar = U256::from_words(5, 0).zigzag();

    assert_eq!(minus_one, -TestScalar::from(1_u64));
    assert_eq!(minus_two, -TestScalar::from(2_u64));
    assert_eq!(minus_three, -TestScalar::from(3_u64));
}

#[test]
fn odd_zigzag_decode_carries_from_low_to_high_word() {
    let decoded: TestScalar = U256::from_words(u128::MAX, 1).zigzag();
    let expected_magnitude: TestScalar = (&U256::from_words(0, 1)).into();

    assert_eq!(decoded, -expected_magnitude);
}

#[test]
fn zigzag_round_trips_positive_and_negative_scalars() {
    for value in 0..1000_u128 {
        let scalar = TestScalar::from(value);
        let decoded: TestScalar = scalar.zigzag().zigzag();

        assert_eq!(decoded, scalar);
    }

    for value in 1..1000_u128 {
        let scalar = -TestScalar::from(value);
        let decoded: TestScalar = scalar.zigzag().zigzag();

        assert_eq!(decoded, scalar);
    }
}
