use super::CompositePolynomial;
use crate::base::scalar::test_scalar::TestScalar;
use alloc::rc::Rc;

#[test]
fn test_composite_polynomial_evaluation() {
    let a: Vec<TestScalar> = vec![
        -TestScalar::from(7u32),
        TestScalar::from(2u32),
        -TestScalar::from(6u32),
        TestScalar::from(17u32),
    ];
    let b: Vec<TestScalar> = vec![
        TestScalar::from(2u32),
        -TestScalar::from(8u32),
        TestScalar::from(4u32),
        TestScalar::from(1u32),
    ];
    let c: Vec<TestScalar> = vec![
        TestScalar::from(1u32),
        TestScalar::from(3u32),
        -TestScalar::from(5u32),
        -TestScalar::from(9u32),
    ];
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(a), Rc::new(b)], TestScalar::from(3u32));
    prod.add_product([Rc::new(c)], TestScalar::from(2u32));
    let prod00 = prod.evaluate(&[TestScalar::from(0u32), TestScalar::from(0u32)]);
    let prod10 = prod.evaluate(&[TestScalar::from(1u32), TestScalar::from(0u32)]);
    let prod01 = prod.evaluate(&[TestScalar::from(0u32), TestScalar::from(1u32)]);
    let prod11 = prod.evaluate(&[TestScalar::from(1u32), TestScalar::from(1u32)]);
    let calc00 = -TestScalar::from(40u32);
    let calc10 = -TestScalar::from(42u32);
    let calc01 = -TestScalar::from(82u32);
    let calc11 = TestScalar::from(33u32);
    assert_eq!(prod00, calc00);
    assert_eq!(prod10, calc10);
    assert_eq!(prod01, calc01);
    assert_eq!(prod11, calc11);
}

#[test]
fn add_product_reuses_existing_multilinear_extensions() {
    let values = Rc::new(vec![TestScalar::from(2), TestScalar::from(3)]);
    let mut prod = CompositePolynomial::new(1);

    prod.add_product([values.clone(), values.clone()], TestScalar::from(5));
    prod.add_product([values], TestScalar::from(7));

    assert_eq!(prod.max_multiplicands, 2);
    assert_eq!(prod.flattened_ml_extensions.len(), 1);
    assert_eq!(prod.products[0].1, vec![0, 0]);
    assert_eq!(prod.products[1].1, vec![0]);
}

#[test]
#[should_panic(expected = "assertion failed: !product.is_empty()")]
fn add_product_rejects_empty_products() {
    let mut prod = CompositePolynomial::<TestScalar>::new(1);

    prod.add_product([], TestScalar::from(5));
}

#[expect(clippy::identity_op)]
#[test]
fn test_composite_polynomial_hypercube_sum() {
    let a: Vec<TestScalar> = vec![
        -TestScalar::from(7u32),
        TestScalar::from(2u32),
        -TestScalar::from(6u32),
        TestScalar::from(17u32),
    ];
    let b: Vec<TestScalar> = vec![
        TestScalar::from(2u32),
        -TestScalar::from(8u32),
        TestScalar::from(4u32),
        TestScalar::from(1u32),
    ];
    let c: Vec<TestScalar> = vec![
        TestScalar::from(1u32),
        TestScalar::from(3u32),
        -TestScalar::from(5u32),
        -TestScalar::from(9u32),
    ];
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(a), Rc::new(b)], TestScalar::from(3u32));
    prod.add_product([Rc::new(c)], TestScalar::from(2u32));
    let sum = prod.hypercube_sum(4);
    assert_eq!(
        sum,
        TestScalar::from(3 * ((-7) * 2 + 2 * (-8) + (-6) * 4 + 17 * 1) + 2 * (1 + 3 + (-5) + (-9)))
    );
}
