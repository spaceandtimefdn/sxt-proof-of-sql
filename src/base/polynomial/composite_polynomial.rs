use ark_std::rc::Rc;
use ark_std::vec::Vec;
use hashbrown::HashMap;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::dense_multilinear_extension::DenseMultilinearExtension;

/// Stores a list of products of `DenseMultilinearExtension` that is meant to be added together.
///
/// The polynomial is represented by a list of products of polynomials along with its coefficient that is meant to be added together.
///
/// This data structure of the polynomial is a list of list of `(coefficient, DenseMultilinearExtension)`.
/// * Number of products n = `self.products.len()`,
/// * Number of multiplicands of ith product m_i = `self.products[i].1.len()`,
/// * Coefficient of ith product c_i = `self.products[i].0`
///
/// The resulting polynomial is
///
/// $$\sum_{i=0}^{n}C_i\cdot\prod_{j=0}^{m_i}P_{ij}$$
///
/// The result polynomial is used as the prover key.
pub struct CompositePolynomial {
    /// max number of multiplicands in each product
    pub max_multiplicands: usize,
    /// number of variables of the polynomial
    pub num_variables: usize,
    /// list of reference to products (as usize) of multilinear extension
    pub products: Vec<(Scalar, Vec<usize>)>,
    /// Stores multilinear extensions in which product multiplicand can refer to.
    pub flattened_ml_extensions: Vec<Rc<DenseMultilinearExtension>>,
    raw_pointers_lookup_table: HashMap<*const DenseMultilinearExtension, usize>,
}
