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
    use super::{SumcheckTerm, SumcheckTermOptimizer};
    use crate::{
        base::scalar::test_scalar::TestScalar,
        sql::proof::SumcheckSubpolynomialType::{Identity, ZeroSum},
    };
    use alloc::{boxed::Box, vec, vec::Vec};

    fn scalar(value: u64) -> TestScalar {
        TestScalar::from(value)
    }

    fn term_values(term: &SumcheckTerm<'_, TestScalar>) -> Vec<Vec<TestScalar>> {
        term.iter()
            .map(|multiplicand| multiplicand.to_sumcheck_term(2))
            .collect()
    }

    #[test]
    fn new_merges_constant_only_terms() {
        let constant_term: SumcheckTerm<'_, TestScalar> = vec![];
        let terms = vec![
            (Identity, scalar(2), &constant_term),
            (Identity, scalar(3), &constant_term),
        ];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 4);
        let optimized_terms = optimizer.terms();
        let optimized = optimized_terms.into_iter().collect::<Vec<_>>();

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].0, Identity);
        assert_eq!(optimized[0].1, scalar(5));
        assert!(optimized[0].2.is_empty());
    }

    #[test]
    fn new_keeps_single_linear_term_unmerged() {
        let linear_values = vec![scalar(1), scalar(2), scalar(3), scalar(4)];
        let linear_term: SumcheckTerm<'_, TestScalar> = vec![Box::new(&linear_values)];
        let terms = vec![(ZeroSum, scalar(7), &linear_term)];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 4);
        let optimized_terms = optimizer.terms();
        let optimized = optimized_terms.into_iter().collect::<Vec<_>>();

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].0, ZeroSum);
        assert_eq!(optimized[0].1, scalar(7));
        assert_eq!(term_values(optimized[0].2), vec![linear_values.clone()]);
    }

    #[test]
    fn new_merges_multiple_linear_terms() {
        let first_values = vec![scalar(1), scalar(2), scalar(3), scalar(4)];
        let second_values = vec![scalar(5), scalar(6), scalar(7), scalar(8)];
        let first_term: SumcheckTerm<'_, TestScalar> = vec![Box::new(&first_values)];
        let second_term: SumcheckTerm<'_, TestScalar> = vec![Box::new(&second_values)];
        let terms = vec![
            (Identity, scalar(2), &first_term),
            (Identity, scalar(3), &second_term),
        ];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 4);
        let optimized_terms = optimizer.terms();
        let optimized = optimized_terms.into_iter().collect::<Vec<_>>();

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].0, Identity);
        assert_eq!(optimized[0].1, scalar(1));
        assert_eq!(
            term_values(optimized[0].2),
            vec![vec![scalar(17), scalar(22), scalar(27), scalar(32)]]
        );
    }

    #[test]
    fn new_merges_constant_and_linear_terms() {
        let constant_term: SumcheckTerm<'_, TestScalar> = vec![];
        let linear_values = vec![scalar(1), scalar(2), scalar(3), scalar(4)];
        let linear_term: SumcheckTerm<'_, TestScalar> = vec![Box::new(&linear_values)];
        let terms = vec![
            (ZeroSum, scalar(5), &constant_term),
            (ZeroSum, scalar(3), &linear_term),
        ];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 4);
        let optimized_terms = optimizer.terms();
        let optimized = optimized_terms.into_iter().collect::<Vec<_>>();

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].0, ZeroSum);
        assert_eq!(optimized[0].1, scalar(1));
        assert_eq!(
            term_values(optimized[0].2),
            vec![vec![scalar(8), scalar(11), scalar(14), scalar(17)]]
        );
    }

    #[test]
    fn new_keeps_superlinear_terms_unmerged() {
        let first_values = vec![scalar(1), scalar(2), scalar(3), scalar(4)];
        let second_values = vec![scalar(5), scalar(6), scalar(7), scalar(8)];
        let superlinear_term: SumcheckTerm<'_, TestScalar> =
            vec![Box::new(&first_values), Box::new(&second_values)];
        let terms = vec![(Identity, scalar(9), &superlinear_term)];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 4);
        let optimized_terms = optimizer.terms();
        let optimized = optimized_terms.into_iter().collect::<Vec<_>>();

        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].0, Identity);
        assert_eq!(optimized[0].1, scalar(9));
        assert_eq!(
            term_values(optimized[0].2),
            vec![first_values.clone(), second_values.clone()]
        );
    }

    #[test]
    fn into_iter_yields_unmerged_terms_before_merged_terms() {
        let constant_term: SumcheckTerm<'_, TestScalar> = vec![];
        let linear_values = vec![scalar(1), scalar(2), scalar(3), scalar(4)];
        let first_superlinear_values = vec![scalar(4), scalar(3), scalar(2), scalar(1)];
        let second_superlinear_values = vec![scalar(8), scalar(7), scalar(6), scalar(5)];
        let linear_term: SumcheckTerm<'_, TestScalar> = vec![Box::new(&linear_values)];
        let superlinear_term: SumcheckTerm<'_, TestScalar> = vec![
            Box::new(&first_superlinear_values),
            Box::new(&second_superlinear_values),
        ];
        let terms = vec![
            (Identity, scalar(5), &constant_term),
            (Identity, scalar(3), &linear_term),
            (Identity, scalar(11), &superlinear_term),
        ];

        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 4);
        let optimized_terms = optimizer.terms();
        let optimized = optimized_terms.into_iter().collect::<Vec<_>>();

        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized[0].0, Identity);
        assert_eq!(optimized[0].1, scalar(11));
        assert_eq!(
            term_values(optimized[0].2),
            vec![
                first_superlinear_values.clone(),
                second_superlinear_values.clone(),
            ]
        );
        assert_eq!(optimized[1].0, Identity);
        assert_eq!(optimized[1].1, scalar(1));
        assert_eq!(
            term_values(optimized[1].2),
            vec![vec![scalar(8), scalar(11), scalar(14), scalar(17)]]
        );
    }
}
