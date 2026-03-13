use crate::base::scalar::{
    test_scalar::{TestMontConfig, TestScalar},
    Scalar, ScalarConversionError,
};
use ark_ff::{BigInt as ArkBigInt, Fp, MontConfig};
use bnum::types::U256;
use num_bigint::BigInt;
#[test]
fn test_bigint_to_scalar_overflow() {
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
fn we_can_bound_modulus_using_max_bits() {
    let modulus_of_i_max_bits = U256::ONE << TestScalar::MAX_BITS;
    let modulus_of_i_max_bits_plus_1 = U256::ONE << (TestScalar::MAX_BITS + 1);
    let modulus_of_test_scalar = U256::from(TestMontConfig::MODULUS.0);
    assert!(modulus_of_i_max_bits <= modulus_of_test_scalar);
    assert!(modulus_of_i_max_bits_plus_1 > modulus_of_test_scalar);
}

#[test]
fn we_can_wrap_unwrap_and_sum_test_scalars() {
    let base_scalars = [
        Fp::new(ArkBigInt([1, 0, 0, 0])),
        Fp::new(ArkBigInt([2, 0, 0, 0])),
        Fp::new(ArkBigInt([3, 0, 0, 0])),
    ];
    let wrapped = TestScalar::wrap_slice(&base_scalars);

    assert_eq!(wrapped, [1_i64, 2, 3].map(TestScalar::from));
    assert_eq!(TestScalar::unwrap_slice(&wrapped), base_scalars);
    assert_eq!(wrapped.iter().sum::<TestScalar>(), TestScalar::from(6));
    let bytes = TestScalar::from_bigint([9, 0, 0, 0]).to_bytes_le();
    assert_eq!(bytes.len(), 32);
    assert_eq!(bytes[0], 9);
    assert!(bytes[1..].iter().all(|byte| *byte == 0));
}

#[test]
fn we_cannot_convert_negative_or_large_scalars_to_u8() {
    assert!(matches!(
        u8::try_from(-TestScalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    ));
    assert!(matches!(
        u8::try_from(TestScalar::from(300_u64)),
        Err(ScalarConversionError::Overflow { .. })
    ));
    assert!(matches!(
        u8::try_from(TestScalar::from([0, 1, 0, 0])),
        Err(ScalarConversionError::Overflow { .. })
    ));
}

#[test]
fn we_cannot_convert_high_limb_scalars_to_i128() {
    assert!(matches!(
        i128::try_from(TestScalar::from([0, 0, 1, 0])),
        Err(ScalarConversionError::Overflow { .. })
    ));
}
