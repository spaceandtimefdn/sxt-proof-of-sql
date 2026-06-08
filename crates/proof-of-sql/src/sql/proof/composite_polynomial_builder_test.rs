use super::CompositePolynomialBuilder;
use crate::{
    base::{polynomial::compute_evaluation_vector, scalar::Scalar},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
};
use num_traits::One;

#[test]
fn we_combine_single_degree_fr_multiplicands() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1)]);
    builder.produce_fr_multiplicand(&-Curve25519Scalar::one(), &[Box::new(&mle2)]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1);
    assert_eq!(p.flattened_ml_extensions.len(), 2);
    let pt = [Curve25519Scalar::from(9_268_764_u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 - eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_dont_duplicate_repeated_mles() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle1)]);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle2)]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3);
    assert_eq!(p.flattened_ml_extensions.len(), 5);
    let pt = [Curve25519Scalar::from(9_268_764_u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval1 + eval1 * eval2);
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_combine_identity_with_zero_sum_polynomials() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mle3 = [12, 22];
    let mle4 = [13, 23];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle2)]);
    builder.produce_zerosum_multiplicand(
        &-Curve25519Scalar::one(),
        &[Box::new(&mle3), Box::new(&mle4)],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3); //1 for the linear term, 1 for the fr multiplicand, 1 for the zerosum multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 6); //1 for fr, 1 for the linear term, and 4 for mle1-4
    let pt = [Curve25519Scalar::from(9_268_764_u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval3 = Curve25519Scalar::from(mle3[0]) * m0 + Curve25519Scalar::from(mle3[1]) * m1;
    let eval4 = Curve25519Scalar::from(mle4[0]) * m0 + Curve25519Scalar::from(mle4[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * eval1 * eval2 - eval3 * eval4;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_only_an_empty_fr_multiplicand() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&Curve25519Scalar::from(17), &[]);
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 1); //1 for the fr multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 2); //1 for fr, 1 for the linear term
    let pt = [Curve25519Scalar::from(9_268_764_u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = (m0 + m1) * Curve25519Scalar::from(17);
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * eval1;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_empty_terms_with_other_terms() {
    let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let mle1 = [10, 20];
    let mle2 = [11, 21];
    let mle3 = [12, 22];
    let mle4 = [13, 23];
    let mut builder = CompositePolynomialBuilder::new(1, &fr);
    builder.produce_fr_multiplicand(&One::one(), &[Box::new(&mle1), Box::new(&mle2)]);
    builder.produce_fr_multiplicand(&Curve25519Scalar::from(17), &[]);
    builder.produce_zerosum_multiplicand(
        &-Curve25519Scalar::one(),
        &[Box::new(&mle3), Box::new(&mle4)],
    );
    let p = builder.make_composite_polynomial();
    assert_eq!(p.products.len(), 3); //1 for the linear term, 1 for the fr multiplicand, 1 for the zerosum multiplicand
    assert_eq!(p.flattened_ml_extensions.len(), 6); //1 for fr, 1 for the linear term, and 4 for mle1-4
    let pt = [Curve25519Scalar::from(9_268_764_u64)];
    let m0 = Curve25519Scalar::one() - pt[0];
    let m1 = pt[0];
    let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
    let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
    let eval3 = Curve25519Scalar::from(mle3[0]) * m0 + Curve25519Scalar::from(mle3[1]) * m1;
    let eval4 = Curve25519Scalar::from(mle4[0]) * m0 + Curve25519Scalar::from(mle4[1]) * m1;
    let eval_fr = fr[0] * m0 + fr[1] * m1;
    let expected = eval_fr * (eval1 * eval2 + Curve25519Scalar::from(17)) - eval3 * eval4;
    assert_eq!(p.evaluate(&pt), expected);
}

#[test]
fn we_can_handle_padded_terms_and_single_term_zerosum_products() {
    let fr = [
        Curve25519Scalar::from(1u64),
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(3u64),
    ];
    let mle1 = [4, 5, 6];
    let mle2 = [7, 8, 9];
    let fr_multiplier = Curve25519Scalar::from(2u64);
    let zerosum_multiplier = Curve25519Scalar::from(3u64);
    let mut builder = CompositePolynomialBuilder::new(2, &fr);
    builder.produce_fr_multiplicand(&fr_multiplier, &[Box::new(&mle1)]);
    builder.produce_zerosum_multiplicand(&zerosum_multiplier, &[Box::new(&mle2)]);

    let p = builder.make_composite_polynomial();

    assert_eq!(p.products.len(), 2);
    assert_eq!(p.flattened_ml_extensions.len(), 3);
    let pt = [
        Curve25519Scalar::from(9_268_764_u64),
        Curve25519Scalar::from(21_533_u64),
    ];
    let mut evaluation_vec = vec![Curve25519Scalar::ZERO; 1 << pt.len()];
    compute_evaluation_vector(&mut evaluation_vec, &pt);

    let eval_fr = fr
        .iter()
        .zip(evaluation_vec.iter())
        .map(|(x, eval)| *x * *eval)
        .sum::<Curve25519Scalar>();
    let eval1 = mle1
        .iter()
        .zip(evaluation_vec.iter())
        .map(|(x, eval)| Curve25519Scalar::from(*x) * *eval)
        .sum::<Curve25519Scalar>();
    let eval2 = mle2
        .iter()
        .zip(evaluation_vec.iter())
        .map(|(x, eval)| Curve25519Scalar::from(*x) * *eval)
        .sum::<Curve25519Scalar>();
    let expected = eval_fr * fr_multiplier * eval1 + zerosum_multiplier * eval2;

    assert_eq!(p.evaluate(&pt), expected);
}
