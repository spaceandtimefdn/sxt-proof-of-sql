//! Tests for SumcheckTermOptimizer.

#[cfg(test)]
mod sumcheck_term_optimizer_test {
    use crate::base::map::IndexMap;
    use crate::base::polynomial::MultilinearExtension;
    use crate::base::scalar::test_scalar::TestScalar;
    use crate::sql::proof::sumcheck_subpolynomial::SumcheckSubpolynomialType;
    use crate::sql::proof::SumcheckTermOptimizer;
    use alloc::vec;
    use alloc::vec::Vec;
    use bumpalo::Bump;

    #[test]
    fn test_sumcheck_term_optimizer_empty() {
        let terms: Vec<(SumcheckSubpolynomialType, TestScalar, &Vec<Box<dyn MultilinearExtension<TestScalar> + '_>>)> = vec![];
        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 0);
        let result = optimizer.terms();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_sumcheck_term_optimizer_debug() {
        let terms: Vec<(SumcheckSubpolynomialType, TestScalar, &Vec<Box<dyn MultilinearExtension<TestScalar> + '_>>)> = vec![];
        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 0);
        let debug_str = format!("{:?}", optimizer);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_sumcheck_subpolynomial_type_variants() {
        let types = vec![
            SumcheckSubpolynomialType::ZeroSum,
            SumcheckSubpolynomialType::Identity,
            SumcheckSubpolynomialType::MLE,
            SumcheckSubpolynomialType::TripleProductSum,
        ];
        for ty in types {
            let debug_str = format!("{:?}", ty);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_optimized_sumcheck_terms_debug() {
        let terms: Vec<(SumcheckSubpolynomialType, TestScalar, &Vec<Box<dyn MultilinearExtension<TestScalar> + '_>>)> = vec![];
        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 0);
        let optimized = optimizer.terms();
        let debug_str = format!("{:?}", optimized);
        assert!(!debug_str.is_empty());
    }
}