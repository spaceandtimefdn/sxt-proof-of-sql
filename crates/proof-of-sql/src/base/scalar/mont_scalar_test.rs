use crate::base::scalar::{
    test_scalar::{TestMontConfig, TestScalar},
    Scalar, ScalarConversionError,
};
use ark_ff::MontConfig;
use bnum::types::U256;
use core::hash::{Hash, Hasher};
use num_bigint::BigInt;

fn assert_overflow<T>(result: Result<T, ScalarConversionError>) {
    assert!(matches!(
        result,
        Err(ScalarConversionError::Overflow { .. })
    ));
}

#[test]
fn we_can_use_reimplemented_scalar_arithmetic_traits() {
    let one = TestScalar::ONE;
    let two = TestScalar::TWO;
    let three = TestScalar::from(3u8);
    let six = TestScalar::from(6u8);

    assert_eq!(one + two, three);
    assert_eq!(three - one, two);
    assert_eq!(two * three, six);
    assert_eq!(-one + one, TestScalar::ZERO);

    let mut assigned = one;
    assigned += two;
    assert_eq!(assigned, three);
    assigned -= one;
    assert_eq!(assigned, two);
    assigned *= three;
    assert_eq!(assigned, six);

    assert_eq!([one, two, three].into_iter().sum::<TestScalar>(), six);
    assert_eq!([one, two, three].into_iter().product::<TestScalar>(), six);
    assert_eq!([one, two, three].iter().sum::<TestScalar>(), six);
    assert_eq!(TestScalar::default(), TestScalar::ZERO);
    assert_eq!(one.clone(), one);
    assert!(two > one);

    let mut left = std::collections::hash_map::DefaultHasher::new();
    let mut right = std::collections::hash_map::DefaultHasher::new();
    one.hash(&mut left);
    TestScalar::ONE.hash(&mut right);
    assert_eq!(left.finish(), right.finish());
}

#[test]
fn we_can_construct_scalar_from_references() {
    let value = 123u16;

    assert_eq!(TestScalar::from(&value), TestScalar::from(value));
}

#[test]
fn test_bigint_to_scalar_overflow() {
    assert_eq!(
        TestScalar::try_from(BigInt::from(0)).unwrap(),
        TestScalar::ZERO
    );
    assert_eq!(
        TestScalar::try_from(
            "3618502788666131106986593281521497120428558179689953803000975469142727125494"
                .parse::<BigInt>()
                .unwrap()
        )
        .unwrap(),
        TestScalar::MAX_SIGNED
    );
    assert_eq!(
        TestScalar::try_from(
            "-3618502788666131106986593281521497120428558179689953803000975469142727125494"
                .parse::<BigInt>()
                .unwrap()
        )
        .unwrap(),
        -TestScalar::MAX_SIGNED
    );

    assert!(matches!(
        TestScalar::try_from(
            "3618502788666131106986593281521497120428558179689953803000975469142727125495"
                .parse::<BigInt>()
                .unwrap()
        ),
        Err(ScalarConversionError::Overflow { .. })
    ));
    assert!(matches!(
        TestScalar::try_from(
            "-3618502788666131106986593281521497120428558179689953803000975469142727125495"
                .parse::<BigInt>()
                .unwrap()
        ),
        Err(ScalarConversionError::Overflow { .. })
    ));
}

#[test]
fn we_can_convert_scalar_to_bool_with_strict_bounds() {
    assert!(!bool::try_from(TestScalar::ZERO).unwrap());
    assert!(bool::try_from(TestScalar::ONE).unwrap());

    assert_overflow(bool::try_from(TestScalar::TWO));
    assert_overflow(bool::try_from(-TestScalar::ONE));
}

#[test]
fn we_can_convert_scalar_to_u8_with_strict_bounds() {
    assert_eq!(u8::try_from(TestScalar::from(255u16)).unwrap(), u8::MAX);

    assert_overflow(u8::try_from(TestScalar::from(256u16)));
    assert_overflow(u8::try_from(-TestScalar::ONE));
}

#[test]
fn we_can_convert_scalar_to_signed_integer_primitives_with_strict_bounds() {
    assert_eq!(i8::try_from(TestScalar::from(i8::MAX)).unwrap(), i8::MAX);
    assert_eq!(i8::try_from(TestScalar::from(i8::MIN)).unwrap(), i8::MIN);
    assert_overflow(i8::try_from(TestScalar::from(i16::from(i8::MAX) + 1)));
    assert_overflow(i8::try_from(TestScalar::from(i16::from(i8::MIN) - 1)));

    assert_eq!(i16::try_from(TestScalar::from(i16::MAX)).unwrap(), i16::MAX);
    assert_eq!(i16::try_from(TestScalar::from(i16::MIN)).unwrap(), i16::MIN);
    assert_overflow(i16::try_from(TestScalar::from(i32::from(i16::MAX) + 1)));
    assert_overflow(i16::try_from(TestScalar::from(i32::from(i16::MIN) - 1)));

    assert_eq!(i32::try_from(TestScalar::from(i32::MAX)).unwrap(), i32::MAX);
    assert_eq!(i32::try_from(TestScalar::from(i32::MIN)).unwrap(), i32::MIN);
    assert_overflow(i32::try_from(TestScalar::from(i64::from(i32::MAX) + 1)));
    assert_overflow(i32::try_from(TestScalar::from(i64::from(i32::MIN) - 1)));

    assert_eq!(i64::try_from(TestScalar::from(i64::MAX)).unwrap(), i64::MAX);
    assert_eq!(
        i64::try_from(-TestScalar::from(i64::MAX as u64 + 1)).unwrap(),
        i64::MIN
    );
    assert_overflow(i64::try_from(TestScalar::from(i64::MAX as u64 + 1)));
    assert_overflow(i64::try_from(-TestScalar::from(i64::MAX as u64 + 2)));

    assert_eq!(
        i128::try_from(TestScalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        i128::try_from(-TestScalar::from(i128::MAX as u128 + 1)).unwrap(),
        i128::MIN
    );
    assert_overflow(i128::try_from(TestScalar::from(i128::MAX as u128 + 1)));
    assert_overflow(i128::try_from(-TestScalar::from(i128::MAX as u128 + 2)));
}

#[test]
fn we_can_convert_scalar_to_bigint_with_signed_field_interpretation() {
    assert_eq!(BigInt::from(TestScalar::ZERO), BigInt::from(0));
    assert_eq!(BigInt::from(TestScalar::from(7u8)), BigInt::from(7));
    assert_eq!(BigInt::from(-TestScalar::from(7u8)), BigInt::from(-7));
}

#[test]
fn we_can_bound_modulus_using_max_bits() {
    let modulus_of_i_max_bits = U256::ONE << TestScalar::MAX_BITS;
    let modulus_of_i_max_bits_plus_1 = U256::ONE << (TestScalar::MAX_BITS + 1);
    let modulus_of_test_scalar = U256::from(TestMontConfig::MODULUS.0);
    assert!(modulus_of_i_max_bits <= modulus_of_test_scalar);
    assert!(modulus_of_i_max_bits_plus_1 > modulus_of_test_scalar);
}
