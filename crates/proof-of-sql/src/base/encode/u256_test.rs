use super::U256;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn from_words_keeps_low_and_high_halves() {
    let low = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210_u128;
    let high = 0x0fed_cba9_8765_4321_0011_2233_4455_6677_u128;

    let value = U256::from_words(low, high);

    assert_eq!(value.low, low);
    assert_eq!(value.high, high);
}

#[test]
fn mont_scalar_conversion_packs_limbs_little_endian() {
    let scalar = TestScalar::from_bigint([
        0x0123_4567_89ab_cdef,
        0xfedc_ba98_7654_3210,
        0x1020_3040_5060_7080,
        0x0000_0000_0000_0001,
    ]);

    let value = U256::from(&scalar);

    assert_eq!(value.low, 0xfedc_ba98_7654_3210_0123_4567_89ab_cdef);
    assert_eq!(value.high, 0x0000_0000_0000_0001_1020_3040_5060_7080);
}

#[test]
fn mont_scalar_conversion_reads_low_then_high_little_endian() {
    let value = U256::from_words(
        0x1111_2222_3333_4444_5555_6666_7777_8888,
        0x0000_0000_0000_0002_9999_aaaa_bbbb_cccc,
    );

    let scalar = TestScalar::from(&value);
    let round_trip_value = U256::from(&scalar);

    assert_eq!(round_trip_value.low, value.low);
    assert_eq!(round_trip_value.high, value.high);
}

#[test]
fn mont_scalar_conversion_round_trips_small_scalars() {
    for input in [
        TestScalar::from(0_u64),
        TestScalar::from(1_u64),
        TestScalar::from(u64::MAX),
        TestScalar::from(u128::MAX),
    ] {
        let encoded = U256::from(&input);
        let decoded = TestScalar::from(&encoded);

        assert_eq!(decoded, input);
    }
}
