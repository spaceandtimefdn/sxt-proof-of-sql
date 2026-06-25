use crate::base::{encode::U256, scalar::test_scalar::TestScalar};

#[test]
fn from_words_preserves_low_and_high_words() {
    let value = U256::from_words(
        0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
        0x0fed_cba9_8765_4321_1234_5678_9abc_def0,
    );

    assert!(value.low == 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);
    assert!(value.high == 0x0fed_cba9_8765_4321_1234_5678_9abc_def0);
}

#[test]
fn small_scalars_round_trip_through_u256_words() {
    for scalar in [
        TestScalar::from(0_u64),
        TestScalar::from(1_u64),
        TestScalar::from(u64::MAX),
        TestScalar::from(u128::MAX),
    ] {
        let words: U256 = (&scalar).into();
        let round_trip: TestScalar = (&words).into();

        assert!(round_trip == scalar);
    }
}

#[test]
fn u256_values_below_the_modulus_round_trip_through_scalars() {
    let value = U256::from_words(
        0xffff_ffff_ffff_ffff_0000_0000_0000_0001,
        0x0000_0000_0000_0000_0000_0000_0000_0001,
    );

    let scalar: TestScalar = (&value).into();
    let round_trip: U256 = (&scalar).into();

    assert!(round_trip == value);
}
