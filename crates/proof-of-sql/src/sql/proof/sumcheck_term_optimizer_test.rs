use super::{
    sumcheck_term_optimizer::{OptimizedSumcheckTerms, SumcheckTermOptimizer},
    SumcheckSubpolynomialType,
};
use crate::base::{polynomial::MultilinearExtension, scalar::{test_scalar::TestScalar, Scalar}};
use alloc::{boxed::Box, vec, vec::Vec};

fn collect_term_info<'a>(
    optimized: &'a OptimizedSumcheckTerms<'a, TestScalar>,
) -> Vec<(SumcheckSubpolynomialType, TestScalar, usize)> {
    optimized
        .into_iter()
        .map(|(ty, coeff, term)| (ty, coeff, term.len()))
        .collect()
}

#[test]
fn we_can_optimize_with_no_terms() {
    let all_terms: Vec<(
        SumcheckSubpolynomialType,
        TestScalar,
        &Vec<Box<dyn MultilinearExtension<TestScalar>>>,
    )> = vec![];
    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);
    assert!(result.is_empty());
}

#[test]
fn we_can_optimize_with_only_constant_terms() {
    // Branch 1: Some(constant_sum) and None for linear
    // Two constant terms (0 MLEs) → merged into one with summed coefficient
    let const_term_a: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];
    let const_term_b: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];

    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(3u64),
            &const_term_a,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(5u64),
            &const_term_b,
        ),
    ];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    // Two constant ZeroSum terms should be merged into one with sum = 8, no MLEs
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(result[0].1, TestScalar::from(8u64));
    assert_eq!(result[0].2, 0);
}

#[test]
fn we_can_optimize_with_only_a_single_constant_term() {
    // Branch 1 with a single constant term → merged to a constant with 0 MLEs
    let const_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];

    let all_terms = vec![(
        SumcheckSubpolynomialType::Identity,
        TestScalar::from(7u64),
        &const_term,
    )];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 4);
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, SumcheckSubpolynomialType::Identity);
    assert_eq!(result[0].1, TestScalar::from(7u64));
    assert_eq!(result[0].2, 0);
}

#[test]
fn we_can_merge_constant_and_linear_terms() {
    // Branch 2: Some(constant_sum) and Some(linear_terms)
    // → combined into one term with 1 MLE and coeff = ONE
    let values = [
        TestScalar::from(1u64),
        TestScalar::from(2u64),
        TestScalar::from(3u64),
        TestScalar::from(4u64),
    ];

    let const_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];
    let linear_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(values.as_ref())];

    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(2u64),
            &const_term,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(3u64),
            &linear_term,
        ),
    ];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), values.len());
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    // Constant + linear → merged into one term with 1 MLE and coeff ONE
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(result[0].1, TestScalar::ONE);
    assert_eq!(result[0].2, 1);
}

#[test]
fn we_can_merge_multiple_linear_terms() {
    // Branch 2: None constant and 2+ linear terms
    // → combined into one term with 1 MLE and coeff = ONE
    let v1 = [
        TestScalar::from(1u64),
        TestScalar::from(2u64),
        TestScalar::from(3u64),
        TestScalar::from(4u64),
    ];
    let v2 = [
        TestScalar::from(5u64),
        TestScalar::from(6u64),
        TestScalar::from(7u64),
        TestScalar::from(8u64),
    ];

    let linear_term_1: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(v1.as_ref())];
    let linear_term_2: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(v2.as_ref())];

    let all_terms = vec![
        (
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(2u64),
            &linear_term_1,
        ),
        (
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(3u64),
            &linear_term_2,
        ),
    ];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), v1.len());
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    // 2 linear terms → merged into 1 MLE with coeff ONE
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, SumcheckSubpolynomialType::Identity);
    assert_eq!(result[0].1, TestScalar::ONE);
    assert_eq!(result[0].2, 1);
}

#[test]
fn we_can_passthrough_single_linear_term() {
    // Branch 3: no constant, single linear term → passthrough unchanged
    let values = [TestScalar::from(10u64), TestScalar::from(20u64)];
    let linear_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(values.as_ref())];

    let all_terms = vec![(
        SumcheckSubpolynomialType::ZeroSum,
        TestScalar::from(7u64),
        &linear_term,
    )];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), values.len());
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    // Single linear term passes through unchanged
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, SumcheckSubpolynomialType::ZeroSum);
    assert_eq!(result[0].1, TestScalar::from(7u64));
    assert_eq!(result[0].2, 1);
}

#[test]
fn we_can_passthrough_superlinear_terms() {
    // Terms with 2+ MLEs always pass through regardless of other terms
    let v1 = [TestScalar::from(1u64), TestScalar::from(2u64)];
    let v2 = [TestScalar::from(3u64), TestScalar::from(4u64)];

    let superlinear_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(v1.as_ref()), Box::new(v2.as_ref())];

    let all_terms = vec![(
        SumcheckSubpolynomialType::Identity,
        TestScalar::from(5u64),
        &superlinear_term,
    )];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), v1.len());
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    // Superlinear term passes through unchanged with 2 MLEs
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, SumcheckSubpolynomialType::Identity);
    assert_eq!(result[0].1, TestScalar::from(5u64));
    assert_eq!(result[0].2, 2);
}

#[test]
fn we_can_handle_both_zerosum_and_identity_types() {
    // Constant terms of different types are merged separately
    let const_zerosum: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];
    let const_identity: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];

    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(4u64),
            &const_zerosum,
        ),
        (
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(6u64),
            &const_identity,
        ),
    ];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), 2);
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    assert_eq!(result.len(), 2);

    let zerosum = result
        .iter()
        .find(|(ty, _, _)| *ty == SumcheckSubpolynomialType::ZeroSum)
        .unwrap();
    let identity = result
        .iter()
        .find(|(ty, _, _)| *ty == SumcheckSubpolynomialType::Identity)
        .unwrap();

    assert_eq!(zerosum.1, TestScalar::from(4u64));
    assert_eq!(zerosum.2, 0);
    assert_eq!(identity.1, TestScalar::from(6u64));
    assert_eq!(identity.2, 0);
}

#[test]
fn we_can_mix_superlinear_and_subquadratic_terms() {
    // Superlinear passes through; subquadratic (constant) is merged
    let v1 = [TestScalar::from(1u64), TestScalar::from(2u64)];
    let v2 = [TestScalar::from(3u64), TestScalar::from(4u64)];
    let superlinear: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(v1.as_ref()), Box::new(v2.as_ref())];
    let constant: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];

    let all_terms = vec![
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(9u64),
            &superlinear,
        ),
        (
            SumcheckSubpolynomialType::ZeroSum,
            TestScalar::from(2u64),
            &constant,
        ),
    ];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), v1.len());
    let optimized = optimizer.terms();
    let result = collect_term_info(&optimized);

    // Should have 2 terms: 1 superlinear passthrough + 1 merged constant
    assert_eq!(result.len(), 2);

    let superlinear_result = result.iter().find(|(_, _, len)| *len == 2).unwrap();
    assert_eq!(superlinear_result.1, TestScalar::from(9u64));

    let constant_result = result.iter().find(|(_, _, len)| *len == 0).unwrap();
    assert_eq!(constant_result.1, TestScalar::from(2u64));
}

#[test]
fn we_can_iterate_optimized_terms_via_into_iter() {
    // Verify IntoIterator works correctly — terms are chained in expected order
    let v = [TestScalar::from(1u64), TestScalar::from(2u64)];
    let superlinear: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
        vec![Box::new(v.as_ref()), Box::new(v.as_ref())];
    let linear: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![Box::new(v.as_ref())];

    // superlinear goes to old_grouped_terms (passthrough)
    // linear with no constant → single passthrough also
    let all_terms = vec![
        (
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(3u64),
            &superlinear,
        ),
        (
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(5u64),
            &linear,
        ),
    ];

    let optimizer = SumcheckTermOptimizer::new(all_terms.into_iter(), v.len());
    let optimized = optimizer.terms();

    // Collect once to verify count
    let count = (&optimized).into_iter().count();
    assert_eq!(count, 2);

    // Collect again to verify contents
    let result = collect_term_info(&optimized);
    assert!(result
        .iter()
        .any(|(ty, coeff, len)| *ty == SumcheckSubpolynomialType::Identity
            && *coeff == TestScalar::from(3u64)
            && *len == 2));
    assert!(result
        .iter()
        .any(|(ty, coeff, len)| *ty == SumcheckSubpolynomialType::Identity
            && *coeff == TestScalar::from(5u64)
            && *len == 1));
}
