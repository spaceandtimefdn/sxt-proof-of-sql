use super::SumcheckSubpolynomialType;
use crate::base::{map::IndexMap, polynomial::MultilinearExtension, scalar::Scalar};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    iter::{Chain, Copied, Flatten, Map},
    slice,
};

type SumcheckTerm<'a, S> = Vec<Box<dyn MultilinearExtension<S> + 'a>>;

pub struct SumcheckTermOptimizer<'a, S: Scalar> {
    merged_terms: Vec<(SumcheckSubpolynomialType, S, Vec<Vec<S>>)>,
    old_grouped_terms: Vec<Vec<(SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>>,
}
pub struct OptimizedSumcheckTerms<'a, S: Scalar> {
    old_grouped_terms: &'a Vec<Vec<(SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>>,
    new_mle_terms: Vec<(SumcheckSubpolynomialType, S, SumcheckTerm<'a, S>)>,
}

fn merge_subquadratic_terms<'a, S: Scalar + 'a>(
    maybe_constant_terms: Option<Vec<(SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>>,
    maybe_linear_terms: Option<Vec<(SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>>,
    merged_terms: &mut Vec<(SumcheckSubpolynomialType, S, Vec<Vec<S>>)>,
    term_length: usize,
    ty: SumcheckSubpolynomialType,
) -> Option<Vec<(SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>> {
    let maybe_constant_sum =
        maybe_constant_terms.map(|terms| terms.into_iter().map(|(_, coeff, _)| coeff).sum());

    match (maybe_constant_sum, maybe_linear_terms) {
        (Some(constant_sum), None) => {
            merged_terms.push((ty, constant_sum, vec![]));
            None
        }
        (maybe_constant_sum, Some(linear_terms))
            if maybe_constant_sum.is_some() || linear_terms.len() >= 2 =>
        {
            let mut combined_term = vec![maybe_constant_sum.unwrap_or(S::ZERO); term_length];
            for (_, coeff, linear_term) in linear_terms {
                linear_term[0].mul_add(&mut combined_term, &coeff);
            }
            merged_terms.push((ty, S::ONE, vec![combined_term]));
            None
        }
        (_, maybe_linear_terms) => maybe_linear_terms,
    }
}

impl<'a, S: Scalar + 'a> SumcheckTermOptimizer<'a, S> {
    pub fn new(
        all_terms: impl Iterator<Item = (SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>,
        term_length: usize,
    ) -> Self {
        let mut groups = all_terms.fold(
            IndexMap::<_, Vec<_>>::default(),
            |mut lookup, (ty, coeff, multiplicands)| {
                lookup
                    .entry((ty, multiplicands.len().min(2)))
                    .or_default()
                    .push((ty, coeff, multiplicands));
                lookup
            },
        );
        let mut merged_terms = Vec::with_capacity(2);
        let old_grouped_terms = [
            SumcheckSubpolynomialType::ZeroSum,
            SumcheckSubpolynomialType::Identity,
        ]
        .into_iter()
        .flat_map(|ty| {
            let maybe_constant_terms = groups.swap_remove(&(ty, 0));
            let maybe_linear_terms = groups.swap_remove(&(ty, 1));
            let maybe_superlinear_terms = groups.swap_remove(&(ty, 2));

            let maybe_combined_terms = merge_subquadratic_terms(
                maybe_constant_terms,
                maybe_linear_terms,
                &mut merged_terms,
                term_length,
                ty,
            );

            [maybe_combined_terms, maybe_superlinear_terms]
                .into_iter()
                .flatten()
        })
        .collect();

        Self {
            merged_terms,
            old_grouped_terms,
        }
    }
}

impl<'a, S: Scalar + 'a> SumcheckTermOptimizer<'a, S> {
    pub fn terms(&'a self) -> OptimizedSumcheckTerms<'a, S> {
        OptimizedSumcheckTerms {
            old_grouped_terms: &self.old_grouped_terms,
            new_mle_terms: self
                .merged_terms
                .iter()
                .map(|(ty, coeff, terms)| {
                    (
                        *ty,
                        *coeff,
                        terms
                            .iter()
                            .map(|mle| -> Box<dyn MultilinearExtension<S>> { Box::new(mle) })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect(),
        }
    }
}

impl<'a, S: Scalar + 'a> IntoIterator for &'a OptimizedSumcheckTerms<'a, S> {
    type Item = (SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>);

    // Currently, `impl Trait` in associated types is unstable. We can change this to the following when it stabilizes:
    // type IntoIter = impl Iterator<Item = (SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>;
    type IntoIter = Chain<
        Copied<
            Flatten<slice::Iter<'a, Vec<(SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>)>>>,
        >,
        Map<
            slice::Iter<'a, (SumcheckSubpolynomialType, S, SumcheckTerm<'a, S>)>,
            fn(
                &'a (SumcheckSubpolynomialType, S, SumcheckTerm<'a, S>),
            ) -> (SumcheckSubpolynomialType, S, &'a SumcheckTerm<'a, S>),
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let result = self.old_grouped_terms.iter().flatten().copied().chain(
            self.new_mle_terms
                .iter()
                .map((|(ty, coeff, terms)| (*ty, *coeff, terms)) as fn(&'a _) -> _),
        );
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn constants_and_multiple_linear_terms_merge_by_type() {
        let linear_a = vec![TestScalar::from(5), TestScalar::from(7)];
        let linear_b = vec![TestScalar::from(11), TestScalar::from(13)];
        let constant_term = vec![];
        let linear_term_a: SumcheckTerm<'_, TestScalar> = vec![Box::new(&linear_a)];
        let linear_term_b: SumcheckTerm<'_, TestScalar> = vec![Box::new(&linear_b)];

        let terms = vec![
            (
                SumcheckSubpolynomialType::ZeroSum,
                TestScalar::from(2),
                &constant_term,
            ),
            (
                SumcheckSubpolynomialType::ZeroSum,
                TestScalar::from(3),
                &linear_term_a,
            ),
            (
                SumcheckSubpolynomialType::ZeroSum,
                TestScalar::from(4),
                &linear_term_b,
            ),
        ];
        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 2);
        let optimized = optimizer.terms();
        let optimized_terms: Vec<_> = (&optimized).into_iter().collect();

        assert_eq!(optimized_terms.len(), 1);
        let (ty, coeff, multiplicands) = optimized_terms[0];
        assert_eq!(ty, SumcheckSubpolynomialType::ZeroSum);
        assert_eq!(coeff, TestScalar::ONE);
        assert_eq!(multiplicands.len(), 1);
        assert_eq!(
            multiplicands[0].to_sumcheck_term(1),
            vec![
                TestScalar::from(2 + 3 * 5 + 4 * 11),
                TestScalar::from(2 + 3 * 7 + 4 * 13),
            ]
        );
    }

    #[test]
    fn single_linear_term_without_constant_is_preserved() {
        let linear = vec![TestScalar::from(5), TestScalar::from(7)];
        let linear_term: SumcheckTerm<'_, TestScalar> = vec![Box::new(&linear)];
        let terms = vec![(
            SumcheckSubpolynomialType::Identity,
            TestScalar::from(3),
            &linear_term,
        )];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 2);
        let optimized = optimizer.terms();
        let optimized_terms: Vec<_> = (&optimized).into_iter().collect();

        assert_eq!(optimized_terms.len(), 1);
        let (ty, coeff, multiplicands) = optimized_terms[0];
        assert_eq!(ty, SumcheckSubpolynomialType::Identity);
        assert_eq!(coeff, TestScalar::from(3));
        assert_eq!(multiplicands.len(), 1);
        assert_eq!(
            multiplicands[0].to_sumcheck_term(1),
            vec![TestScalar::from(5), TestScalar::from(7)]
        );
    }
}
