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
fn we_can_format_test_scalar_values() {
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{}", TestScalar::from(0x00AB_C123_u64))
    );
    assert_eq!(
        "1000000000000000000000000000000014DEF9DEA2F79CD65812631A5C4A12CA",
        format!("{}", TestScalar::from(-0x00AB_C123_i64))
    );
    assert_eq!(
        "+0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", TestScalar::from(0x00AB_C123_u64))
    );
    assert_eq!(
        "-0x0000...C123",
        format!("{:+#}", TestScalar::from(-0x00AB_C123_i64))
    );
}

#[test]
fn we_can_serialize_test_scalar_limbs() {
    let scalar = TestScalar::from(0x00AB_C123_u64);
    let serialized = serde_json::to_string(&scalar).unwrap();

    assert_eq!(serialized, "[0,0,0,11256099]");
    assert_eq!(
        serde_json::from_str::<TestScalar>(&serialized).unwrap(),
        scalar
    );
}

#[test]
fn we_can_convert_test_scalar_with_little_endian_bytes() {
    let bytes = [0x23, 0xC1, 0xAB, 0x00, 0x34, 0x12];
    let scalar = TestScalar::from_le_bytes_mod_order(&bytes);

    assert_eq!(&scalar.to_bytes_le()[..bytes.len()], bytes);
    assert_eq!(TestScalar::from_bigint([0x1234_00AB_C123, 0, 0, 0]), scalar);
}
