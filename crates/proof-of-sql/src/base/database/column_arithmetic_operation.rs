use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{
        slice_decimal_operation::{
            try_add_decimal_columns, try_divide_decimal_columns, try_multiply_decimal_columns,
            try_subtract_decimal_columns,
        },
        slice_operation::{
            try_add, try_div, try_mul, try_slice_binary_op, try_slice_binary_op_left_upcast,
            try_slice_binary_op_right_upcast, try_sub,
        },
        ColumnType, OwnedColumn,
    },
    math::decimal::Precision,
    scalar::Scalar,
};
use alloc::{string::ToString, vec::Vec};
use core::fmt::Debug;
use num_bigint::BigInt;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

pub trait ArithmeticOp {
    fn op<T>(l: &T, r: &T) -> ColumnOperationResult<T>
    where
        T: Debug + CheckedDiv + CheckedMul + CheckedAdd + CheckedSub;
    fn decimal_op<S, T0, T1>(
        lhs: &[T0],
        rhs: &[T1],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
    where
        S: Scalar + From<T0> + From<T1>,
        T0: Copy + Debug + Into<BigInt>,
        T1: Copy + Debug + Into<BigInt>;

    #[expect(clippy::too_many_lines)]
    fn owned_column_element_wise_arithmetic<S: Scalar>(
        lhs: &OwnedColumn<S>,
        rhs: &OwnedColumn<S>,
    ) -> ColumnOperationResult<OwnedColumn<S>> {
        if lhs.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: lhs.len(),
                len_b: rhs.len(),
            });
        }
        match (&lhs, &rhs) {
            // We can cast a u8 into any of the signed types without incident. However, we cannot cast
            // a signed type into a u8 without potentially losing data. Therefore, we need a way to check
            // that a signed type is greater than zero before we can cast it into a u8.
            (OwnedColumn::Uint8(_), OwnedColumn::TinyInt(_)) => {
                Err(ColumnOperationError::SignedCastingError {
                    left_type: ColumnType::Uint8,
                    right_type: ColumnType::TinyInt,
                })
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::Uint8(rhs)) => {
                Ok(OwnedColumn::Uint8(try_slice_binary_op(lhs, rhs, Self::op)?))
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::SmallInt(rhs)) => Ok(OwnedColumn::SmallInt(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Uint8(lhs), OwnedColumn::Int(rhs)) => Ok(OwnedColumn::Int(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Uint8(lhs), OwnedColumn::BigInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Uint8(lhs), OwnedColumn::Int128(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Uint8(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            (OwnedColumn::TinyInt(_), OwnedColumn::Uint8(_)) => {
                Err(ColumnOperationError::SignedCastingError {
                    left_type: ColumnType::TinyInt,
                    right_type: ColumnType::Uint8,
                })
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::TinyInt(rhs)) => Ok(OwnedColumn::TinyInt(
                try_slice_binary_op(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::TinyInt(lhs), OwnedColumn::SmallInt(rhs)) => Ok(OwnedColumn::SmallInt(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::TinyInt(lhs), OwnedColumn::Int(rhs)) => Ok(OwnedColumn::Int(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::TinyInt(lhs), OwnedColumn::BigInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::TinyInt(lhs), OwnedColumn::Int128(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::TinyInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }

            (OwnedColumn::SmallInt(lhs), OwnedColumn::TinyInt(rhs)) => Ok(OwnedColumn::SmallInt(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::SmallInt(lhs), OwnedColumn::SmallInt(rhs)) => Ok(OwnedColumn::SmallInt(
                try_slice_binary_op(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::SmallInt(lhs), OwnedColumn::Int(rhs)) => Ok(OwnedColumn::Int(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::SmallInt(lhs), OwnedColumn::BigInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::SmallInt(lhs), OwnedColumn::Int128(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::SmallInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }

            (OwnedColumn::Int(lhs), OwnedColumn::TinyInt(rhs)) => Ok(OwnedColumn::Int(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int(lhs), OwnedColumn::SmallInt(rhs)) => Ok(OwnedColumn::Int(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int(lhs), OwnedColumn::Int(rhs)) => {
                Ok(OwnedColumn::Int(try_slice_binary_op(lhs, rhs, Self::op)?))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::BigInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int(lhs), OwnedColumn::Int128(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }

            (OwnedColumn::BigInt(lhs), OwnedColumn::TinyInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::BigInt(lhs), OwnedColumn::SmallInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::BigInt(lhs), OwnedColumn::Int(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::BigInt(lhs), OwnedColumn::BigInt(rhs)) => Ok(OwnedColumn::BigInt(
                try_slice_binary_op(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::BigInt(lhs), OwnedColumn::Int128(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_left_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::BigInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }

            (OwnedColumn::Int128(lhs), OwnedColumn::TinyInt(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int128(lhs), OwnedColumn::SmallInt(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int128(lhs), OwnedColumn::Int(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int128(lhs), OwnedColumn::BigInt(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op_right_upcast(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int128(lhs), OwnedColumn::Int128(rhs)) => Ok(OwnedColumn::Int128(
                try_slice_binary_op(lhs, rhs, Self::op)?,
            )),
            (OwnedColumn::Int128(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }

            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::TinyInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            (
                OwnedColumn::Decimal75(_, _, lhs_values),
                OwnedColumn::Decimal75(_, _, rhs_values),
            ) => {
                let (new_precision, new_scale, new_values) =
                    Self::decimal_op(lhs_values, rhs_values, lhs.column_type(), rhs.column_type())?;
                Ok(OwnedColumn::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "ArithmeticOp".to_string(),
                left_type: lhs.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

pub struct AddOp {}
impl ArithmeticOp for AddOp {
    fn op<T>(l: &T, r: &T) -> ColumnOperationResult<T>
    where
        T: CheckedAdd + Debug,
    {
        try_add(l, r)
    }

    fn decimal_op<S, T0, T1>(
        lhs: &[T0],
        rhs: &[T1],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
    where
        S: Scalar + From<T0> + From<T1>,
        T0: Copy,
        T1: Copy,
    {
        try_add_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }
}

pub struct SubOp {}
impl ArithmeticOp for SubOp {
    fn op<T>(l: &T, r: &T) -> ColumnOperationResult<T>
    where
        T: CheckedSub + Debug,
    {
        try_sub(l, r)
    }

    fn decimal_op<S, T0, T1>(
        lhs: &[T0],
        rhs: &[T1],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
    where
        S: Scalar + From<T0> + From<T1>,
        T0: Copy,
        T1: Copy,
    {
        try_subtract_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }
}

pub struct MulOp {}
impl ArithmeticOp for MulOp {
    fn op<T>(l: &T, r: &T) -> ColumnOperationResult<T>
    where
        T: CheckedMul + Debug,
    {
        try_mul(l, r)
    }

    fn decimal_op<S, T0, T1>(
        lhs: &[T0],
        rhs: &[T1],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
    where
        S: Scalar + From<T0> + From<T1>,
        T0: Copy,
        T1: Copy,
    {
        try_multiply_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }
}

pub struct DivOp {}
impl ArithmeticOp for DivOp {
    fn op<T>(l: &T, r: &T) -> ColumnOperationResult<T>
    where
        T: CheckedDiv + Debug,
    {
        try_div(l, r)
    }

    fn decimal_op<S, T0, T1>(
        lhs: &[T0],
        rhs: &[T1],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
    where
        S: Scalar + From<T0> + From<T1>,
        T0: Copy + Debug + Into<BigInt>,
        T1: Copy + Debug + Into<BigInt>,
    {
        try_divide_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::{ColumnOperationError, OwnedColumn},
        scalar::test_scalar::TestScalar,
    };

    type S = TestScalar;

    // ── AddOp ────────────────────────────────────────────────────────────────

    #[test]
    fn add_bigint_columns_succeeds() {
        let lhs = OwnedColumn::<S>::BigInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<S>::BigInt(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::<S>::BigInt(vec![11, 22, 33]));
    }

    #[test]
    fn add_returns_error_on_different_lengths() {
        let lhs = OwnedColumn::<S>::BigInt(vec![1, 2]);
        let rhs = OwnedColumn::<S>::BigInt(vec![10, 20, 30]);
        let err = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 }
        ));
    }

    #[test]
    fn add_returns_error_on_varchar_column() {
        let lhs = OwnedColumn::<S>::VarChar(vec!["a".to_string(), "b".to_string()]);
        let rhs = OwnedColumn::<S>::VarChar(vec!["c".to_string(), "d".to_string()]);
        let err = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::BinaryOperationInvalidColumnType { .. }
        ));
    }

    #[test]
    fn add_int128_overflow_returns_error() {
        let lhs = OwnedColumn::<S>::Int128(vec![i128::MAX]);
        let rhs = OwnedColumn::<S>::Int128(vec![1]);
        let err = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(err, ColumnOperationError::IntegerOverflow { .. }));
    }

    // ── SubOp ────────────────────────────────────────────────────────────────

    #[test]
    fn sub_bigint_columns_succeeds() {
        let lhs = OwnedColumn::<S>::BigInt(vec![10, 20, 30]);
        let rhs = OwnedColumn::<S>::BigInt(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::<S>::BigInt(vec![9, 18, 27]));
    }

    #[test]
    fn sub_returns_error_on_different_lengths() {
        let lhs = OwnedColumn::<S>::BigInt(vec![10, 20]);
        let rhs = OwnedColumn::<S>::BigInt(vec![1, 2, 3]);
        let err = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 }
        ));
    }

    #[test]
    fn sub_int_underflow_returns_error() {
        let lhs = OwnedColumn::<S>::Int128(vec![i128::MIN]);
        let rhs = OwnedColumn::<S>::Int128(vec![1]);
        let err = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(err, ColumnOperationError::IntegerOverflow { .. }));
    }

    // ── MulOp ────────────────────────────────────────────────────────────────

    #[test]
    fn mul_bigint_columns_succeeds() {
        let lhs = OwnedColumn::<S>::BigInt(vec![2, 3, 4]);
        let rhs = OwnedColumn::<S>::BigInt(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::<S>::BigInt(vec![10, 18, 28]));
    }

    #[test]
    fn mul_returns_error_on_different_lengths() {
        let lhs = OwnedColumn::<S>::BigInt(vec![2, 3]);
        let rhs = OwnedColumn::<S>::BigInt(vec![5, 6, 7]);
        let err = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 }
        ));
    }

    #[test]
    fn mul_int_overflow_returns_error() {
        let lhs = OwnedColumn::<S>::Int128(vec![i128::MAX]);
        let rhs = OwnedColumn::<S>::Int128(vec![2]);
        let err = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(err, ColumnOperationError::IntegerOverflow { .. }));
    }

    // ── DivOp ────────────────────────────────────────────────────────────────

    #[test]
    fn div_bigint_columns_succeeds() {
        let lhs = OwnedColumn::<S>::BigInt(vec![10, 20, 30]);
        let rhs = OwnedColumn::<S>::BigInt(vec![2, 4, 5]);
        let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::<S>::BigInt(vec![5, 5, 6]));
    }

    #[test]
    fn div_returns_error_on_different_lengths() {
        let lhs = OwnedColumn::<S>::BigInt(vec![10, 20]);
        let rhs = OwnedColumn::<S>::BigInt(vec![2, 4, 5]);
        let err = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 3 }
        ));
    }

    #[test]
    fn div_by_zero_returns_error() {
        let lhs = OwnedColumn::<S>::BigInt(vec![10]);
        let rhs = OwnedColumn::<S>::BigInt(vec![0]);
        let err = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(err, ColumnOperationError::DivisionByZero));
    }

    #[test]
    fn div_returns_error_on_varchar_column() {
        let lhs = OwnedColumn::<S>::VarChar(vec!["x".to_string()]);
        let rhs = OwnedColumn::<S>::VarChar(vec!["y".to_string()]);
        let err = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::BinaryOperationInvalidColumnType { .. }
        ));
    }

    // ── SignedCastingError (Uint8 OP TinyInt) ─────────────────────────────────

    #[test]
    fn add_uint8_tinyint_returns_signed_casting_error() {
        let lhs = OwnedColumn::<S>::Uint8(vec![1u8, 2u8]);
        let rhs = OwnedColumn::<S>::TinyInt(vec![1i8, 2i8]);
        let err = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::SignedCastingError {
                left_type: ColumnType::Uint8,
                right_type: ColumnType::TinyInt,
                ..
            }
        ));
    }
}
