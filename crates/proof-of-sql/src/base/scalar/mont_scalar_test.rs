use crate::base::scalar::{
    test_scalar::{TestMontConfig, TestScalar},
    Scalar, ScalarConversionError,
};
use ark_ff::MontConfig;
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
fn we_can_roundtrip_mont_scalar_le_bytes() {
    let scalars = [
        TestScalar::ZERO,
        TestScalar::ONE,
        TestScalar::from(255_u64),
        TestScalar::MAX_SIGNED,
        -TestScalar::ONE,
    ];

    for scalar in scalars {
        assert_eq!(
            TestScalar::from_le_bytes_mod_order(&scalar.to_bytes_le()),
            scalar
        );
    }
}

#[test]
fn we_can_wrap_and_unwrap_mont_scalar_slices() {
    let scalars = vec![
        TestScalar::ZERO,
        TestScalar::from(17_u64),
        -TestScalar::from(42_u64),
        TestScalar::MAX_SIGNED,
    ];

    let raw_scalars = TestScalar::unwrap_slice(&scalars);
    let wrapped_scalars = TestScalar::wrap_slice(&raw_scalars);

    assert_eq!(wrapped_scalars, scalars);
    assert_eq!(TestScalar::unwrap_slice(&wrapped_scalars), raw_scalars);
}
