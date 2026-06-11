use super::U256;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn from_words_preserves_low_and_high_halves() {
    let value = U256::from_words(
        0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
        0x0000_0000_0000_1234_abcd_0000_ffff_0000,
    );

    assert_eq!(value.low, 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
    assert_eq!(value.high, 0x0000_0000_0000_1234_abcd_0000_ffff_0000);
}

#[test]
fn u256_round_trips_through_test_scalar_when_value_is_in_field() {
    let value = U256::from_words(
        0xfedc_ba98_7654_3210_0123_4567_89ab_cdef,
        0x0000_0000_0000_0000_0000_0000_0000_1234,
    );

    let scalar: TestScalar = (&value).into();
    let round_trip = U256::from(&scalar);

    assert_eq!(round_trip.low, value.low);
    assert_eq!(round_trip.high, value.high);
}

#[test]
fn scalar_to_u256_uses_little_endian_word_ordering() {
    let scalar = TestScalar::from(0x1234_5678_90ab_cdef_u64);
    let encoded = U256::from(&scalar);

    assert_eq!(encoded.low, 0x1234_5678_90ab_cdef);
    assert_eq!(encoded.high, 0);
}
