use crate::base::{database::Column, if_rayon, scalar::Scalar, slice_ops};
use alloc::vec::Vec;
use core::{ffi::c_void, fmt::Debug};
use num_traits::Zero;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Interface for operating on multilinear extension's in-place
pub trait MultilinearExtension<S: Scalar>: Debug + Sync {
    /// Given an evaluation vector, compute the evaluation of the multilinear
    /// extension
    fn inner_product(&self, evaluation_vec: &[S]) -> S;

    /// multiply and add the MLE to a scalar vector
    fn mul_add(&self, res: &mut [S], multiplier: &S);

    /// convert the MLE to a form that can be used in sumcheck
    fn to_sumcheck_term(&self, num_vars: usize) -> Vec<S>;

    /// pointer to identify the slice forming the MLE
    fn id(&self) -> (*const c_void, usize);

    #[cfg(test)]
    /// Given an evaluation point, compute the evaluation of the multilinear
    /// extension. This is inefficient and should only be used for testing.
    fn evaluate_at_point(&self, evaluation_point: &[S]) -> S {
        let mut evaluation_vec = vec![Default::default(); 1 << evaluation_point.len()];
        super::compute_evaluation_vector(&mut evaluation_vec, evaluation_point);
        self.inner_product(&evaluation_vec)
    }
}

impl<'a, T: Sync + Debug, S: Scalar> MultilinearExtension<S> for &'a [T]
where
    &'a T: Into<S>,
{
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        slice_ops::inner_product(evaluation_vec, self)
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        slice_ops::mul_add_assign(res, *multiplier, self);
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Vec<S> {
        let values = self;
        let n = 1 << num_vars;
        assert!(n >= values.len());
        if_rayon!(values.par_iter(), values.iter())
            .map(Into::into)
            .chain(if_rayon!(
                rayon::iter::repeat_n(Zero::zero(), n - values.len()),
                itertools::repeat_n(Zero::zero(), n - values.len())
            ))
            .collect()
    }

    fn id(&self) -> (*const c_void, usize) {
        (self.as_ptr().cast::<c_void>(), self.len())
    }
}

/// Macro to implement [`MultilinearExtension`] for slice-like types by delegating to the slice impl.
macro_rules! slice_like_mle_impl {
    () => {
        fn inner_product(&self, evaluation_vec: &[S]) -> S {
            (&self[..]).inner_product(evaluation_vec)
        }

        fn mul_add(&self, res: &mut [S], multiplier: &S) {
            (&self[..]).mul_add(res, multiplier)
        }

        fn to_sumcheck_term(&self, num_vars: usize) -> Vec<S> {
            (&self[..]).to_sumcheck_term(num_vars)
        }

        fn id(&self) -> (*const c_void, usize) {
            (&self[..]).id()
        }
    };
}

impl<'a, T: Sync + Debug, S: Scalar> MultilinearExtension<S> for &'a Vec<T>
where
    &'a T: Into<S>,
{
    slice_like_mle_impl!();
}

impl<'a, T: Sync + Debug, const N: usize, S: Scalar> MultilinearExtension<S> for &'a [T; N]
where
    &'a T: Into<S>,
{
    slice_like_mle_impl!();
}

impl<S: Scalar> MultilinearExtension<S> for &Column<'_, S> {
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        match self {
            Column::Boolean(c) => c.inner_product(evaluation_vec),
            Column::Scalar(c)
            | Column::VarChar((_, c))
            | Column::VarBinary((_, c))
            | Column::Decimal75(_, _, c) => c.inner_product(evaluation_vec),
            Column::Uint8(c) => c.inner_product(evaluation_vec),
            Column::TinyInt(c) => c.inner_product(evaluation_vec),
            Column::SmallInt(c) => c.inner_product(evaluation_vec),
            Column::Int(c) => c.inner_product(evaluation_vec),
            Column::BigInt(c) | Column::TimestampTZ(_, _, c) => c.inner_product(evaluation_vec),
            Column::Int128(c) => c.inner_product(evaluation_vec),
        }
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        match self {
            Column::Boolean(c) => c.mul_add(res, multiplier),
            Column::Scalar(c)
            | Column::VarChar((_, c))
            | Column::VarBinary((_, c))
            | Column::Decimal75(_, _, c) => {
                c.mul_add(res, multiplier);
            }
            Column::Uint8(c) => c.mul_add(res, multiplier),
            Column::TinyInt(c) => c.mul_add(res, multiplier),
            Column::SmallInt(c) => c.mul_add(res, multiplier),
            Column::Int(c) => c.mul_add(res, multiplier),
            Column::BigInt(c) | Column::TimestampTZ(_, _, c) => c.mul_add(res, multiplier),
            Column::Int128(c) => c.mul_add(res, multiplier),
        }
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Vec<S> {
        match self {
            Column::Boolean(c) => c.to_sumcheck_term(num_vars),
            Column::Scalar(c)
            | Column::VarChar((_, c))
            | Column::VarBinary((_, c))
            | Column::Decimal75(_, _, c) => c.to_sumcheck_term(num_vars),
            Column::Uint8(c) => c.to_sumcheck_term(num_vars),
            Column::TinyInt(c) => c.to_sumcheck_term(num_vars),
            Column::SmallInt(c) => c.to_sumcheck_term(num_vars),
            Column::Int(c) => c.to_sumcheck_term(num_vars),
            Column::BigInt(c) | Column::TimestampTZ(_, _, c) => c.to_sumcheck_term(num_vars),
            Column::Int128(c) => c.to_sumcheck_term(num_vars),
        }
    }

    fn id(&self) -> (*const c_void, usize) {
        match self {
            Column::Boolean(c) => MultilinearExtension::<S>::id(c),
            Column::Scalar(c)
            | Column::VarChar((_, c))
            | Column::VarBinary((_, c))
            | Column::Decimal75(_, _, c) => MultilinearExtension::<S>::id(c),
            Column::Uint8(c) => MultilinearExtension::<S>::id(c),
            Column::TinyInt(c) => MultilinearExtension::<S>::id(c),
            Column::SmallInt(c) => MultilinearExtension::<S>::id(c),
            Column::Int(c) => MultilinearExtension::<S>::id(c),
            Column::BigInt(c) | Column::TimestampTZ(_, _, c) => MultilinearExtension::<S>::id(c),
            Column::Int128(c) => MultilinearExtension::<S>::id(c),
        }
    }
}

impl<S: Scalar> MultilinearExtension<S> for Column<'_, S> {
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        (&self).inner_product(evaluation_vec)
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        (&self).mul_add(res, multiplier);
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Vec<S> {
        (&self).to_sumcheck_term(num_vars)
    }

    fn id(&self) -> (*const c_void, usize) {
        (&self).id()
    }
}

#[cfg(test)]
mod tests {
    use super::MultilinearExtension;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn we_can_use_multilinear_extension_methods_for_arrays() {
        let values = [2_i64, 3, 4];
        let extension = &values;
        let evaluation_vec = vec![TestScalar::from(10_u32), 20.into(), 30.into()];

        assert_eq!(
            extension.inner_product(&evaluation_vec),
            TestScalar::from(200_u32)
        );

        let mut result = vec![TestScalar::from(1_u32); 3];
        extension.mul_add(&mut result, &TestScalar::from(2_u32));
        assert_eq!(
            result,
            vec![
                TestScalar::from(5_u32),
                TestScalar::from(7_u32),
                TestScalar::from(9_u32)
            ]
        );
        assert_eq!(
            <&[i64; 3] as MultilinearExtension<TestScalar>>::to_sumcheck_term(&extension, 2),
            vec![
                TestScalar::from(2_u32),
                TestScalar::from(3_u32),
                TestScalar::from(4_u32),
                TestScalar::from(0_u32)
            ]
        );
    }
}
