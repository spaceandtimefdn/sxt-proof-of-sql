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
    use super::SumcheckTermOptimizer;
    use crate::{
        base::{polynomial::MultilinearExtension, scalar::test_scalar::TestScalar},
        sql::proof::SumcheckSubpolynomialType,
    };
    use alloc::{boxed::Box, vec};

    #[test]
    fn we_can_merge_constant_and_linear_terms() {
        let linear_a = vec![
            TestScalar::from(1_u32),
            TestScalar::from(2_u32),
            TestScalar::from(3_u32),
        ];
        let linear_b = vec![
            TestScalar::from(10_u32),
            TestScalar::from(20_u32),
            TestScalar::from(30_u32),
        ];
        let constant_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> = vec![];
        let linear_a_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
            vec![Box::new(&linear_a)];
        let linear_b_term: Vec<Box<dyn MultilinearExtension<TestScalar>>> =
            vec![Box::new(&linear_b)];

        let terms = vec![
            (
                SumcheckSubpolynomialType::Identity,
                TestScalar::from(5_u32),
                &constant_term,
            ),
            (
                SumcheckSubpolynomialType::Identity,
                TestScalar::from(2_u32),
                &linear_a_term,
            ),
            (
                SumcheckSubpolynomialType::Identity,
                TestScalar::from(3_u32),
                &linear_b_term,
            ),
        ];
        let optimizer = SumcheckTermOptimizer::new(terms.into_iter(), 3);
        let optimized_terms = optimizer.terms();
        let collected_terms: Vec<_> = (&optimized_terms).into_iter().collect();

        assert_eq!(collected_terms.len(), 1);
        assert_eq!(collected_terms[0].0, SumcheckSubpolynomialType::Identity);
        assert_eq!(collected_terms[0].1, TestScalar::from(1_u32));
        assert_eq!(collected_terms[0].2.len(), 1);
        assert_eq!(
            collected_terms[0].2[0].to_sumcheck_term(2),
            vec![
                TestScalar::from(37_u32),
                TestScalar::from(69_u32),
                TestScalar::from(101_u32),
                TestScalar::from(0_u32)
            ]
        );
    }
}
