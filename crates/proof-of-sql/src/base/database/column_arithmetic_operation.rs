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
    use crate::base::scalar::test_scalar::TestScalar;

    fn bigint_col(vals: Vec<i64>) -> OwnedColumn<TestScalar> {
        OwnedColumn::BigInt(vals)
    }

    fn int_col(vals: Vec<i32>) -> OwnedColumn<TestScalar> {
        OwnedColumn::Int(vals)
    }

    fn tinyint_col(vals: Vec<i8>) -> OwnedColumn<TestScalar> {
        OwnedColumn::TinyInt(vals)
    }

    fn smallint_col(vals: Vec<i16>) -> OwnedColumn<TestScalar> {
        OwnedColumn::SmallInt(vals)
    }

    fn int128_col(vals: Vec<i128>) -> OwnedColumn<TestScalar> {
        OwnedColumn::Int128(vals)
    }

    fn uint8_col(vals: Vec<u8>) -> OwnedColumn<TestScalar> {
        OwnedColumn::Uint8(vals)
    }

    // --- AddOp tests ---

    #[test]
    fn add_bigint() {
        let lhs = bigint_col(vec![1, 2, 3]);
        let rhs = bigint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_int() {
        let lhs = int_col(vec![1, 2, 3]);
        let rhs = int_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_tinyint() {
        let lhs = tinyint_col(vec![1, 2, 3]);
        let rhs = tinyint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, tinyint_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_uint8() {
        let lhs = uint8_col(vec![1, 2, 3]);
        let rhs = uint8_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, uint8_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_int128() {
        let lhs = int128_col(vec![1, 2, 3]);
        let rhs = int128_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int128_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_smallint() {
        let lhs = smallint_col(vec![1, 2, 3]);
        let rhs = smallint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, smallint_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_upcast_int_to_bigint() {
        let lhs = int_col(vec![1, 2, 3]);
        let rhs = bigint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_upcast_tinyint_to_bigint() {
        let lhs = tinyint_col(vec![1, 2, 3]);
        let rhs = bigint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_upcast_smallint_to_int() {
        let lhs = smallint_col(vec![1, 2, 3]);
        let rhs = int_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_upcast_uint8_to_smallint() {
        let lhs = uint8_col(vec![1, 2, 3]);
        let rhs = smallint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, smallint_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_upcast_uint8_to_int() {
        let lhs = uint8_col(vec![1, 2, 3]);
        let rhs = int_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int_col(vec![11, 22, 33]));
    }

    #[test]
    fn add_upcast_uint8_to_int128() {
        let lhs = uint8_col(vec![1, 2, 3]);
        let rhs = int128_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int128_col(vec![11, 22, 33]));
    }

    // --- SubOp tests ---

    #[test]
    fn sub_bigint() {
        let lhs = bigint_col(vec![10, 20, 30]);
        let rhs = bigint_col(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![9, 18, 27]));
    }

    #[test]
    fn sub_int() {
        let lhs = int_col(vec![10, 20, 30]);
        let rhs = int_col(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int_col(vec![9, 18, 27]));
    }

    // --- MulOp tests ---

    #[test]
    fn mul_bigint() {
        let lhs = bigint_col(vec![2, 3, 4]);
        let rhs = bigint_col(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![10, 18, 28]));
    }

    #[test]
    fn mul_int() {
        let lhs = int_col(vec![2, 3, 4]);
        let rhs = int_col(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int_col(vec![10, 18, 28]));
    }

    // --- DivOp tests ---

    #[test]
    fn div_bigint() {
        let lhs = bigint_col(vec![10, 20, 30]);
        let rhs = bigint_col(vec![2, 5, 3]);
        let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![5, 4, 10]));
    }

    #[test]
    fn div_by_zero_errors() {
        let lhs = bigint_col(vec![10]);
        let rhs = bigint_col(vec![0]);
        assert!(DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).is_err());
    }

    // --- Error cases ---

    #[test]
    fn different_length_errors() {
        let lhs = bigint_col(vec![1, 2]);
        let rhs = bigint_col(vec![1]);
        assert!(AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).is_err());
    }

    #[test]
    fn signed_casting_error_uint8_tinyint() {
        let lhs = uint8_col(vec![1]);
        let rhs = tinyint_col(vec![1]);
        assert!(AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).is_err());
    }

    #[test]
    fn signed_casting_error_tinyint_uint8() {
        let lhs = tinyint_col(vec![1]);
        let rhs = uint8_col(vec![1]);
        assert!(AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).is_err());
    }

    #[test]
    fn incompatible_types_boolean_bigint() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true]);
        let rhs = bigint_col(vec![1]);
        assert!(AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).is_err());
    }

    #[test]
    fn add_empty_columns() {
        let lhs = bigint_col(vec![]);
        let rhs = bigint_col(vec![]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![]));
    }

    #[test]
    fn right_upcast_bigint_to_int128() {
        let lhs = int128_col(vec![1, 2, 3]);
        let rhs = bigint_col(vec![10, 20, 30]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int128_col(vec![11, 22, 33]));
    }

    #[test]
    fn right_upcast_tinyint_to_smallint() {
        let lhs = smallint_col(vec![10, 20, 30]);
        let rhs = tinyint_col(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, smallint_col(vec![11, 22, 33]));
    }

    #[test]
    fn right_upcast_tinyint_to_int() {
        let lhs = int_col(vec![10, 20, 30]);
        let rhs = tinyint_col(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, int_col(vec![11, 22, 33]));
    }

    #[test]
    fn right_upcast_smallint_to_bigint() {
        let lhs = bigint_col(vec![10, 20, 30]);
        let rhs = smallint_col(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, bigint_col(vec![11, 22, 33]));
    }
}
