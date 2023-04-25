pub mod ark_scalar;
#[cfg(test)]
mod ark_scalar_test;
pub use ark_scalar::{
    convert_ark_scalar_slice_to_data_slice, from_ark_scalar, to_ark_scalar, ArkScalar,
};

mod dense_multilinear_extension;
pub use dense_multilinear_extension::DenseMultilinearExtension;

mod composite_polynomial;
pub use composite_polynomial::{CompositePolynomial, CompositePolynomialInfo};
#[cfg(test)]
mod composite_polynomial_test;

mod interpolate;
#[cfg(test)]
mod interpolate_test;
pub use interpolate::interpolate_uni_poly;
