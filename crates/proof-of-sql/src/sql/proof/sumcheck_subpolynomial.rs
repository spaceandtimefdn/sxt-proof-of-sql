use super::CompositePolynomialBuilder;
use crate::base::{polynomial::MultilinearExtension, scalar::Scalar};
use alloc::{boxed::Box, vec::Vec};

/// The type of a sumcheck subpolynomial
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum SumcheckSubpolynomialType {
    /// The subpolynomial should be zero at every entry/row
    Identity,
    /// The subpolynomial should sum to zero across every entry/row
    ZeroSum,
}

/// A term in a sumcheck subpolynomial, represented as a product of multilinear
/// extensions and a constant.
pub type SumcheckSubpolynomialTerm<'a, S> = (S, Vec<Box<dyn MultilinearExtension<S> + 'a>>);

/// A polynomial that can be used to check a contraint and can be aggregated
/// into a single sumcheck polynomial.
/// There are two types of subpolynomials:
/// 1. [`Identity`](SumcheckSubpolynomialType::Identity): the subpolynomial should be zero at every entry/row
/// 2. [`ZeroSum`](SumcheckSubpolynomialType::ZeroSum): the subpolynomial should sum to zero across every entry/row
///
/// The subpolynomial is represented as a sum of terms, where each term is a
/// product of multilinear extensions and a constant.
#[derive(Debug)]
pub struct SumcheckSubpolynomial<'a, S: Scalar> {
    terms: Vec<SumcheckSubpolynomialTerm<'a, S>>,
    subpolynomial_type: SumcheckSubpolynomialType,
}

impl<'a, S: Scalar> SumcheckSubpolynomial<'a, S> {
    /// Form subpolynomial from a sum of multilinear extension products
    pub fn new(
        subpolynomial_type: SumcheckSubpolynomialType,
        terms: Vec<SumcheckSubpolynomialTerm<'a, S>>,
    ) -> Self {
        Self {
            terms,
            subpolynomial_type,
        }
    }

    /// Combine the subpolynomial into a combined composite polynomial
    pub fn compose(
        &self,
        composite_polynomial: &mut CompositePolynomialBuilder<S>,
        group_multiplier: S,
    ) {
        for (mult, term) in &self.terms {
            match self.subpolynomial_type {
                SumcheckSubpolynomialType::Identity => {
                    composite_polynomial.produce_fr_multiplicand(&(*mult * group_multiplier), term);
                }
                SumcheckSubpolynomialType::ZeroSum => composite_polynomial
                    .produce_zerosum_multiplicand(&(*mult * group_multiplier), term),
            }
        }
    }

    pub(crate) fn subpolynomial_type(&self) -> SumcheckSubpolynomialType {
        self.subpolynomial_type
    }

    /// Returns an iterator over the terms of the subpolynomial, where each term's
    /// coefficient is multiplied by the given multiplier.
    ///
    /// # Arguments
    ///
    /// * `multiplier` - The scalar value to multiply each term's coefficient by.
    ///
    /// # Returns
    ///
    /// An iterator that yields tuples containing the subpolynomial type, the
    /// multiplied coefficient, and a slice of multilinear extensions.
    pub(crate) fn iter_mul_by(
        &self,
        multiplier: S,
    ) -> impl Iterator<
        Item = (
            SumcheckSubpolynomialType,
            S,
            &Vec<Box<dyn MultilinearExtension<S> + 'a>>,
        ),
    > {
        self.terms.iter().map(move |(coeff, multiplicands)| {
            (self.subpolynomial_type, multiplier * *coeff, multiplicands)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompositePolynomialBuilder, SumcheckSubpolynomial, SumcheckSubpolynomialTerm,
        SumcheckSubpolynomialType,
    };
    use crate::base::scalar::test_scalar::TestScalar;
    use crate::proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar;
    use alloc::boxed::Box;

    #[test]
    fn test_iter_mul_by() {
        let mle1 = vec![TestScalar::from(1), TestScalar::from(2)];
        let mle2 = vec![TestScalar::from(3), TestScalar::from(4)];

        let terms: Vec<SumcheckSubpolynomialTerm<_>> = vec![
            (TestScalar::from(2), vec![Box::new(&mle1)]),
            (TestScalar::from(3), vec![Box::new(&mle2)]),
        ];
        let subpoly = SumcheckSubpolynomial::new(SumcheckSubpolynomialType::Identity, terms);

        let multiplier = TestScalar::from(5);
        let mut iter = subpoly.iter_mul_by(multiplier);

        let (subpoly_type, coeff, _extensions) = iter.next().unwrap();
        assert_eq!(subpoly_type, SumcheckSubpolynomialType::Identity);
        assert_eq!(coeff, TestScalar::from(10));

        let (subpoly_type, coeff, _extensions) = iter.next().unwrap();
        assert_eq!(subpoly_type, SumcheckSubpolynomialType::Identity);
        assert_eq!(coeff, TestScalar::from(15));

        assert!(iter.next().is_none());
    }

    #[test]
    fn we_can_compose_identity_and_zero_sum_subpolynomials() {
        let fr = [Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
        let mle1 = [10, 20];
        let mle2 = [11, 21];
        let mle3 = [12, 22];

        let mut builder = CompositePolynomialBuilder::new(1, &fr);
        let identity_terms: Vec<SumcheckSubpolynomialTerm<_>> = vec![(
            Curve25519Scalar::from(2u64),
            vec![Box::new(&mle1), Box::new(&mle2)],
        )];
        let zerosum_terms: Vec<SumcheckSubpolynomialTerm<_>> =
            vec![(Curve25519Scalar::from(3u64), vec![Box::new(&mle3)])];
        let group_multiplier = Curve25519Scalar::from(5u64);

        SumcheckSubpolynomial::new(SumcheckSubpolynomialType::Identity, identity_terms)
            .compose(&mut builder, group_multiplier);
        SumcheckSubpolynomial::new(SumcheckSubpolynomialType::ZeroSum, zerosum_terms)
            .compose(&mut builder, group_multiplier);

        let p = builder.make_composite_polynomial();
        assert_eq!(p.products.len(), 3);
        assert_eq!(p.flattened_ml_extensions.len(), 5);

        let pt = [Curve25519Scalar::from(9_268_764_u64)];
        let m0 = Curve25519Scalar::from(1u64) - pt[0];
        let m1 = pt[0];
        let eval_fr = fr[0] * m0 + fr[1] * m1;
        let eval1 = Curve25519Scalar::from(mle1[0]) * m0 + Curve25519Scalar::from(mle1[1]) * m1;
        let eval2 = Curve25519Scalar::from(mle2[0]) * m0 + Curve25519Scalar::from(mle2[1]) * m1;
        let eval3 = Curve25519Scalar::from(mle3[0]) * m0 + Curve25519Scalar::from(mle3[1]) * m1;
        let expected = Curve25519Scalar::from(10u64) * eval_fr * eval1 * eval2
            + Curve25519Scalar::from(15u64) * eval3;
        assert_eq!(p.evaluate(&pt), expected);
    }
}
