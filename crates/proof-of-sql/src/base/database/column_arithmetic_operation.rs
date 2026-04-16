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
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};
    use alloc::{string::ToString, vec};

    #[test]
    fn arithmetic_rejects_unsupported_signed_uint8_mixed_pairs() {
        let uint8s = OwnedColumn::<TestScalar>::Uint8(vec![1, 2, 3]);
        let tinyints = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);

        let left_err = AddOp::owned_column_element_wise_arithmetic(&uint8s, &tinyints).unwrap_err();
        assert_eq!(
            left_err,
            ColumnOperationError::SignedCastingError {
                left_type: ColumnType::Uint8,
                right_type: ColumnType::TinyInt,
            }
        );

        let right_err =
            AddOp::owned_column_element_wise_arithmetic(&tinyints, &uint8s).unwrap_err();
        assert_eq!(
            right_err,
            ColumnOperationError::SignedCastingError {
                left_type: ColumnType::TinyInt,
                right_type: ColumnType::Uint8,
            }
        );
    }

    #[test]
    fn arithmetic_scalar_wrappers_surface_overflow_and_divide_by_zero() {
        let add_err = AddOp::op(&u8::MAX, &1_u8).unwrap_err();
        assert_eq!(
            add_err,
            ColumnOperationError::IntegerOverflow {
                error: "Overflow in integer addition 255 + 1".to_string(),
            }
        );

        let sub_err = SubOp::op(&i8::MIN, &1_i8).unwrap_err();
        assert_eq!(
            sub_err,
            ColumnOperationError::IntegerOverflow {
                error: "Overflow in integer subtraction -128 - 1".to_string(),
            }
        );

        let mul_err = MulOp::op(&i16::MAX, &2_i16).unwrap_err();
        assert_eq!(
            mul_err,
            ColumnOperationError::IntegerOverflow {
                error: "Overflow in integer multiplication 32767 * 2".to_string(),
            }
        );

        let div_err = DivOp::op(&3_i16, &0_i16).unwrap_err();
        assert_eq!(div_err, ColumnOperationError::DivisionByZero);
    }

    #[test]
    fn decimal_helpers_delegate_to_expected_arithmetic_routines() {
        let integers = [1_i8, -2, 3];
        let decimals = [4_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let integer_type = ColumnType::TinyInt;
        let decimal_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);

        let added = AddOp::decimal_op(&integers, &decimals, integer_type, decimal_type).unwrap();
        assert_eq!(
            added,
            (
                Precision::new(11).unwrap(),
                2,
                vec![
                    TestScalar::from(104_i64),
                    TestScalar::from(-195_i64),
                    TestScalar::from(298_i64),
                ],
            )
        );

        let subtracted =
            SubOp::decimal_op(&integers, &decimals, integer_type, decimal_type).unwrap();
        assert_eq!(
            subtracted,
            (
                Precision::new(11).unwrap(),
                2,
                vec![
                    TestScalar::from(96_i64),
                    TestScalar::from(-205_i64),
                    TestScalar::from(302_i64),
                ],
            )
        );

        let multiplied =
            MulOp::decimal_op(&integers, &decimals, integer_type, decimal_type).unwrap();
        assert_eq!(
            multiplied,
            (
                Precision::new(14).unwrap(),
                2,
                vec![
                    TestScalar::from(4_i64),
                    TestScalar::from(-10_i64),
                    TestScalar::from(-6_i64),
                ],
            )
        );

        let dividend = [0_i8, 2, 3];
        let divisors = [4_i8, 5, 2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let division_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 2);
        let divided = DivOp::decimal_op(&dividend, &divisors, integer_type, division_type).unwrap();
        assert_eq!(
            divided,
            (
                Precision::new(11).unwrap(),
                6,
                vec![
                    TestScalar::from(0_i64),
                    TestScalar::from(40_000_000_i64),
                    TestScalar::from(150_000_000_i64),
                ],
            )
        );
    }
}
