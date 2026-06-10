use super::{sumcheck_term_optimizer::SumcheckTermOptimizer, SumcheckSubpolynomialType};
use crate::base::{
    polynomial::MultilinearExtension,
    scalar::{test_scalar::TestScalar, Scalar},
};
use alloc::{boxed::Box, vec, vec::Vec};

type TestTerm<'a> = Vec<Box<dyn MultilinearExtension<TestScalar> + 'a>>;

/// Reads the `index`-th entry of an MLE by taking an inner product with a one-hot vector.
fn read_entry(
    mle: &dyn MultilinearExtension<TestScalar>,
    index: usize,
    length: usize,
) -> TestScalar {
    let mut one_hot = vec![TestScalar::ZERO; length];
    one_hot[index] = TestScalar::ONE;
    mle.inner_product(&one_hot)
}

#[test]
fn we_can_merge_multiple_constant_terms_into_a_single_constant_term() {
    let constant_a: TestTerm = vec![];
    let constant_b: TestTerm = vec![];
    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(3u64),
            &constant_a,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(5u64),
            &constant_b,
        ),
    ];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let output = optimizer.terms();
    let collected: Vec<_> = (&output).into_iter().collect();

    assert_eq!(collected.len(), 1);
    let (ty, coeff, term) = collected[0];
    assert_eq!(ty, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(coeff, TestScalar::from(8u64));
    assert!(term.is_empty());
}

#[test]
fn we_can_combine_multiple_linear_terms_into_a_single_term() {
    let data_a: Vec<TestScalar> = vec![1u64, 2, 3, 4]
        .into_iter()
        .map(TestScalar::from)
        .collect();
    let data_b: Vec<TestScalar> = vec![10u64, 20, 30, 40]
        .into_iter()
        .map(TestScalar::from)
        .collect();
    let linear_a: TestTerm = vec![Box::new(&data_a)];
    let linear_b: TestTerm = vec![Box::new(&data_b)];
    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(2u64),
            &linear_a,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(3u64),
            &linear_b,
        ),
    ];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let output = optimizer.terms();
    let collected: Vec<_> = (&output).into_iter().collect();

    assert_eq!(collected.len(), 1);
    let (ty, coeff, term) = collected[0];
    assert_eq!(ty, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(coeff, TestScalar::ONE);
    assert_eq!(term.len(), 1);
    // combined[i] = 2 * data_a[i] + 3 * data_b[i]
    for (i, expected) in [32u64, 64, 96, 128].into_iter().enumerate() {
        assert_eq!(
            read_entry(term[0].as_ref(), i, 4),
            TestScalar::from(expected)
        );
    }
}

#[test]
fn a_single_linear_term_without_constant_terms_passes_through_unchanged() {
    let data: Vec<TestScalar> = vec![1u64, 2, 3, 4]
        .into_iter()
        .map(TestScalar::from)
        .collect();
    let linear: TestTerm = vec![Box::new(&data)];
    let all_terms = vec![(
        SumcheckSubpolynomialType::Identity,
        TestScalar::from(7u64),
        &linear,
    )];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let output = optimizer.terms();
    let collected: Vec<_> = (&output).into_iter().collect();

    assert_eq!(collected.len(), 1);
    let (ty, coeff, term) = collected[0];
    assert_eq!(ty, SumcheckSubpolynomialType::Identity);
    assert_eq!(coeff, TestScalar::from(7u64));
    // Passthrough must reference the original term, not a merged copy.
    assert!(core::ptr::eq(term, &raw const linear));
}

#[test]
fn a_constant_term_combines_with_a_single_linear_term_and_pads_with_the_constant() {
    let constant: TestTerm = vec![];
    let data: Vec<TestScalar> = vec![1u64, 2].into_iter().map(TestScalar::from).collect();
    let linear: TestTerm = vec![Box::new(&data)];
    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(5u64),
            &constant,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(2u64),
            &linear,
        ),
    ];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let output = optimizer.terms();
    let collected: Vec<_> = (&output).into_iter().collect();

    assert_eq!(collected.len(), 1);
    let (ty, coeff, term) = collected[0];
    assert_eq!(ty, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(coeff, TestScalar::ONE);
    assert_eq!(term.len(), 1);
    // combined[i] = 5 + 2 * data[i] for i < data.len(); entries past the
    // linear term's length are padded with the constant sum alone.
    for (i, expected) in [7u64, 9, 5, 5].into_iter().enumerate() {
        assert_eq!(
            read_entry(term[0].as_ref(), i, 4),
            TestScalar::from(expected)
        );
    }
}

#[test]
fn superlinear_terms_pass_through_while_constant_terms_merge() {
    let data_a: Vec<TestScalar> = vec![1u64, 2, 3, 4]
        .into_iter()
        .map(TestScalar::from)
        .collect();
    let data_b: Vec<TestScalar> = vec![5u64, 6, 7, 8]
        .into_iter()
        .map(TestScalar::from)
        .collect();
    let data_c: Vec<TestScalar> = vec![9u64, 10, 11, 12]
        .into_iter()
        .map(TestScalar::from)
        .collect();
    let quadratic: TestTerm = vec![Box::new(&data_a), Box::new(&data_b)];
    let cubic: TestTerm = vec![Box::new(&data_a), Box::new(&data_b), Box::new(&data_c)];
    let constant: TestTerm = vec![];
    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(11u64),
            &quadratic,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(13u64),
            &cubic,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(4u64),
            &constant,
        ),
    ];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let output = optimizer.terms();
    let collected: Vec<_> = (&output).into_iter().collect();

    assert_eq!(collected.len(), 3);
    // Passthrough (superlinear) terms come first, preserving identity and order.
    assert!(core::ptr::eq(collected[0].2, &raw const quadratic));
    assert_eq!(collected[0].1, TestScalar::from(11u64));
    assert!(core::ptr::eq(collected[1].2, &raw const cubic));
    assert_eq!(collected[1].1, TestScalar::from(13u64));
    // The lone constant term is merged into a multiplicand-free term.
    assert_eq!(collected[2].0, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(collected[2].1, TestScalar::from(4u64));
    assert!(collected[2].2.is_empty());
}

#[test]
fn zero_sum_and_identity_terms_merge_independently() {
    let constant_a: TestTerm = vec![];
    let constant_b: TestTerm = vec![];
    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(3u64),
            &constant_a,
        ),
        (
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(4u64),
            &constant_b,
        ),
    ];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 2);
    let output = optimizer.terms();
    let collected: Vec<_> = (&output).into_iter().collect();

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].0, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(collected[0].1, TestScalar::from(3u64));
    assert!(collected[0].2.is_empty());
    assert_eq!(collected[1].0, SumcheckSubpolynomialType::Identity);
    assert_eq!(collected[1].1, TestScalar::from(4u64));
    assert!(collected[1].2.is_empty());
}

#[test]
fn an_empty_term_list_optimizes_to_an_empty_term_list() {
    let all_terms: Vec<(SumcheckSubpolynomialType, TestScalar, &TestTerm)> = Vec::new();
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 8);
    let output = optimizer.terms();
    assert_eq!((&output).into_iter().count(), 0);
}
