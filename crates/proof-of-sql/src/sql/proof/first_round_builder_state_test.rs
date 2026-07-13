use super::FirstRoundBuilder;
use crate::proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar;

#[test]
fn range_length_only_grows() {
    let mut builder = FirstRoundBuilder::<Curve25519Scalar>::new(4);
    assert_eq!(builder.range_length(), 4);

    builder.update_range_length(2);
    assert_eq!(builder.range_length(), 4);

    builder.update_range_length(4);
    assert_eq!(builder.range_length(), 4);

    builder.update_range_length(8);
    assert_eq!(builder.range_length(), 8);
}

#[test]
fn chi_and_rho_evaluation_lengths_are_tracked() {
    let mut builder = FirstRoundBuilder::<Curve25519Scalar>::new(2);

    builder.produce_chi_evaluation_length(5);
    builder.produce_chi_evaluation_length(3);
    builder.produce_rho_evaluation_length(7);
    builder.produce_rho_evaluation_length(11);

    assert_eq!(builder.chi_evaluation_lengths(), &[5, 3]);
    assert_eq!(builder.rho_evaluation_lengths(), &[7, 11]);
    assert_eq!(builder.range_length(), 5);
}
