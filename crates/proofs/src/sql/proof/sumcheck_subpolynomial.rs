use super::{CompositePolynomialBuilder, MultilinearExtension};

use curve25519_dalek::scalar::Scalar;

/// A polynomial that sums to zero across binary values and can be aggregated
/// into a single sumcheck polynomial
pub struct SumcheckSubpolynomial<'a> {
    terms: Vec<(Scalar, Vec<Box<dyn MultilinearExtension + 'a>>)>,
}

impl<'a> SumcheckSubpolynomial<'a> {
    /// Form subpolynomial from a sum of multilinear extension products
    pub fn new(terms: Vec<(Scalar, Vec<Box<dyn MultilinearExtension + 'a>>)>) -> Self {
        Self { terms }
    }

    pub fn compose(
        &self,
        composite_polynomial: &mut CompositePolynomialBuilder,
        group_multiplier: Scalar,
    ) {
        for (mult, term) in self.terms.iter() {
            composite_polynomial.produce_fr_multiplicand(&(mult * group_multiplier), term);
        }
    }
}
