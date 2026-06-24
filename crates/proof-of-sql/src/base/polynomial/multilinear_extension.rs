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
    use alloc::{vec, vec::Vec};

    #[test]
    fn slice_inner_product_with_first_basis_vector() {
        let data: &[i64] = &[10, 20, 30, 40];
        let eval: Vec<TestScalar> = vec![
            TestScalar::from(1u64),
            TestScalar::from(0u64),
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        ];
        assert_eq!(data.inner_product(&eval), TestScalar::from(10u64));
    }

    #[test]
    fn slice_inner_product_with_second_basis_vector() {
        let data: &[i64] = &[10, 20, 30, 40];
        let eval: Vec<TestScalar> = vec![
            TestScalar::from(0u64),
            TestScalar::from(1u64),
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        ];
        assert_eq!(data.inner_product(&eval), TestScalar::from(20u64));
    }

    #[test]
    fn slice_inner_product_all_zero_eval_is_zero() {
        let data: &[i64] = &[1, 2, 3];
        let eval: Vec<TestScalar> = vec![
            TestScalar::from(0u64),
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        ];
        assert_eq!(data.inner_product(&eval), TestScalar::from(0u64));
    }

    #[test]
    fn slice_inner_product_empty_is_zero() {
        let data: &[i64] = &[];
        let eval: Vec<TestScalar> = vec![];
        assert_eq!(data.inner_product(&eval), TestScalar::from(0u64));
    }

    #[test]
    fn slice_mul_add_with_zero_multiplier_unchanged() {
        let data: &[i64] = &[1, 2, 3];
        let mut result: Vec<TestScalar> = vec![
            TestScalar::from(5u64),
            TestScalar::from(5u64),
            TestScalar::from(5u64),
        ];
        let multiplier = TestScalar::from(0u64);
        data.mul_add(&mut result, &multiplier);
        // result unchanged since multiplier is 0
        assert_eq!(result[0], TestScalar::from(5u64));
        assert_eq!(result[1], TestScalar::from(5u64));
        assert_eq!(result[2], TestScalar::from(5u64));
    }

    #[test]
    fn slice_mul_add_with_unit_multiplier_adds_data() {
        let data: &[i64] = &[1, 2, 3];
        let mut result: Vec<TestScalar> = vec![
            TestScalar::from(0u64),
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        ];
        let multiplier = TestScalar::from(1u64);
        data.mul_add(&mut result, &multiplier);
        assert_eq!(result[0], TestScalar::from(1u64));
        assert_eq!(result[1], TestScalar::from(2u64));
        assert_eq!(result[2], TestScalar::from(3u64));
    }

    #[test]
    fn slice_to_sumcheck_term_length_is_power_of_two() {
        let data: &[i64] = &[1, 2, 3, 4];
        let term: Vec<TestScalar> = data.to_sumcheck_term(2);
        assert_eq!(term.len(), 4);
    }

    #[test]
    fn slice_to_sumcheck_term_with_more_vars_pads_with_zeros() {
        let data: &[i64] = &[1, 2];
        let term: Vec<TestScalar> = data.to_sumcheck_term(2);
        assert_eq!(term.len(), 4);
        assert_eq!(term[2], TestScalar::from(0u64));
        assert_eq!(term[3], TestScalar::from(0u64));
    }

    #[test]
    fn slice_to_sumcheck_term_data_preserved() {
        let data: &[i64] = &[5, 10];
        let term: Vec<TestScalar> = data.to_sumcheck_term(1);
        assert_eq!(term.len(), 2);
        assert_eq!(term[0], TestScalar::from(5u64));
        assert_eq!(term[1], TestScalar::from(10u64));
    }

    #[test]
    fn slice_id_returns_correct_length() {
        let data: &[i64] = &[1, 2, 3];
        let (_, len) = MultilinearExtension::<TestScalar>::id(&data);
        assert_eq!(len, 3);
    }

    #[test]
    fn slice_id_same_slice_same_pointer() {
        let data: &[i64] = &[1, 2, 3];
        let (ptr1, _) = MultilinearExtension::<TestScalar>::id(&data);
        let (ptr2, _) = MultilinearExtension::<TestScalar>::id(&data);
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn evaluate_at_point_empty_point_returns_single_element() {
        let data: &[i64] = &[42];
        let result: TestScalar = data.evaluate_at_point(&[]);
        assert_eq!(result, TestScalar::from(42u64));
    }

    #[test]
    fn evaluate_at_point_single_point_zero_gives_first_element() {
        // point=[0]: basis=[1, 0], so evaluation = data[0]*1 + data[1]*0 = data[0]
        let data: &[i64] = &[10, 20];
        let result: TestScalar = data.evaluate_at_point(&[TestScalar::from(0u64)]);
        assert_eq!(result, TestScalar::from(10u64));
    }

    #[test]
    fn evaluate_at_point_single_point_one_gives_second_element() {
        // point=[1]: basis=[0, 1], so evaluation = data[0]*0 + data[1]*1 = data[1]
        let data: &[i64] = &[10, 20];
        let result: TestScalar = data.evaluate_at_point(&[TestScalar::from(1u64)]);
        assert_eq!(result, TestScalar::from(20u64));
    }

    #[test]
    fn vec_inner_product_works_same_as_slice() {
        let data_vec: Vec<i64> = vec![3, 6, 9];
        let data_slice: &[i64] = &[3, 6, 9];
        let eval: Vec<TestScalar> = vec![
            TestScalar::from(1u64),
            TestScalar::from(0u64),
            TestScalar::from(0u64),
        ];
        let from_vec = (&data_vec).inner_product(&eval);
        let from_slice = data_slice.inner_product(&eval);
        assert_eq!(from_vec, from_slice);
    }

    #[test]
    fn array_inner_product_works() {
        let data: &[i64; 2] = &[7, 14];
        let eval: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(0u64)];
        assert_eq!(data.inner_product(&eval), TestScalar::from(7u64));
    }

    #[test]
    fn boolean_slice_inner_product() {
        let data: &[bool] = &[true, false, true];
        let eval: Vec<TestScalar> = vec![
            TestScalar::from(1u64),
            TestScalar::from(1u64),
            TestScalar::from(1u64),
        ];
        // true -> 1, false -> 0, so sum = 1+0+1 = 2
        assert_eq!(data.inner_product(&eval), TestScalar::from(2u64));
    }

    #[test]
    fn slice_mul_add_with_multiplier_two() {
        let data: &[i64] = &[3, 4];
        let mut result: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(1u64)];
        let multiplier = TestScalar::from(2u64);
        data.mul_add(&mut result, &multiplier);
        // result[0] = 1 + 3*2 = 7, result[1] = 1 + 4*2 = 9
        assert_eq!(result[0], TestScalar::from(7u64));
        assert_eq!(result[1], TestScalar::from(9u64));
    }
}
