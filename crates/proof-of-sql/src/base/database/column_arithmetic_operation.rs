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
    use super::{AddOp, ArithmeticOp, DivOp, MulOp, SubOp};

    // AddOp::op tests
    #[test]
    fn add_op_i64_basic_addition() {
        assert_eq!(AddOp::op::<i64>(&5, &3).unwrap(), 8);
    }

    #[test]
    fn add_op_i64_overflow_returns_error() {
        assert!(AddOp::op::<i64>(&i64::MAX, &1).is_err());
    }

    #[test]
    fn add_op_i32_basic_addition() {
        assert_eq!(AddOp::op::<i32>(&10, &20).unwrap(), 30);
    }

    #[test]
    fn add_op_i64_negative_values() {
        assert_eq!(AddOp::op::<i64>(&-5, &3).unwrap(), -2);
    }

    #[test]
    fn add_op_i64_both_negative() {
        assert_eq!(AddOp::op::<i64>(&-5, &-3).unwrap(), -8);
    }

    #[test]
    fn add_op_i64_identity_zero() {
        assert_eq!(AddOp::op::<i64>(&42, &0).unwrap(), 42);
    }

    // SubOp::op tests
    #[test]
    fn sub_op_i64_basic_subtraction() {
        assert_eq!(SubOp::op::<i64>(&10, &3).unwrap(), 7);
    }

    #[test]
    fn sub_op_i64_underflow_returns_error() {
        assert!(SubOp::op::<i64>(&i64::MIN, &1).is_err());
    }

    #[test]
    fn sub_op_i32_basic_subtraction() {
        assert_eq!(SubOp::op::<i32>(&20, &5).unwrap(), 15);
    }

    #[test]
    fn sub_op_i64_same_values_is_zero() {
        assert_eq!(SubOp::op::<i64>(&7, &7).unwrap(), 0);
    }

    #[test]
    fn sub_op_i64_negative_result() {
        assert_eq!(SubOp::op::<i64>(&3, &5).unwrap(), -2);
    }

    // MulOp::op tests
    #[test]
    fn mul_op_i64_basic_multiplication() {
        assert_eq!(MulOp::op::<i64>(&5, &3).unwrap(), 15);
    }

    #[test]
    fn mul_op_i64_overflow_returns_error() {
        assert!(MulOp::op::<i64>(&i64::MAX, &2).is_err());
    }

    #[test]
    fn mul_op_i64_by_zero_returns_zero() {
        assert_eq!(MulOp::op::<i64>(&100, &0).unwrap(), 0);
    }

    #[test]
    fn mul_op_i64_negative_positive() {
        assert_eq!(MulOp::op::<i64>(&-3, &4).unwrap(), -12);
    }

    #[test]
    fn mul_op_i64_negative_negative() {
        assert_eq!(MulOp::op::<i64>(&-3, &-4).unwrap(), 12);
    }

    #[test]
    fn mul_op_i32_basic() {
        assert_eq!(MulOp::op::<i32>(&7, &8).unwrap(), 56);
    }

    // DivOp::op tests
    #[test]
    fn div_op_i64_basic_division() {
        assert_eq!(DivOp::op::<i64>(&10, &2).unwrap(), 5);
    }

    #[test]
    fn div_op_i64_division_by_zero_returns_error() {
        assert!(DivOp::op::<i64>(&10, &0).is_err());
    }

    #[test]
    fn div_op_i64_truncates_towards_zero() {
        assert_eq!(DivOp::op::<i64>(&7, &2).unwrap(), 3);
    }

    #[test]
    fn div_op_i32_basic() {
        assert_eq!(DivOp::op::<i32>(&100, &5).unwrap(), 20);
    }

    #[test]
    fn div_op_i64_negative_dividend() {
        assert_eq!(DivOp::op::<i64>(&-10, &2).unwrap(), -5);
    }

    // owned_column_element_wise_arithmetic with different lengths
    #[test]
    fn arithmetic_on_different_length_columns_returns_error() {
        use crate::base::{database::OwnedColumn, scalar::test_scalar::TestScalar};
        let lhs = OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 2]);
        assert!(AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).is_err());
    }
}
