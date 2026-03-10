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

    // create Decimal75 column for testing
    fn decimal_column(
        precision: u8,
        scale: i8,
        values: Vec<TestScalar>,
    ) -> OwnedColumn<TestScalar> {
        OwnedColumn::Decimal75(Precision::new(precision).unwrap(), scale, values)
    }

    #[test]
    fn test_uint8_with_tinyint_error() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
    }

    #[test]
    fn test_tinyint_with_uint8_error() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
    }

    #[test]
    fn test_uint8_with_uint8_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Uint8(vec![5, 7, 9]));
    }

    #[test]
    fn test_uint8_with_smallint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::SmallInt(vec![5, 7, 9]));
    }

    #[test]
    fn test_uint8_with_int_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int(vec![5, 7, 9]));
    }

    #[test]
    fn test_uint8_with_bigint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::BigInt(vec![5, 7, 9]));
    }

    #[test]
    fn test_uint8_with_int128_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![5, 7, 9]));
    }

    #[test]
    fn test_uint8_with_decimal75_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(400),
                TestScalar::from(500),
                TestScalar::from(600),
            ],
        );
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_tinyint_with_smallint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::SmallInt(vec![5, 7, 9]));
    }

    #[test]
    fn test_smallint_with_tinyint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::SmallInt(vec![5, 7, 9]));
    }

    #[test]
    fn test_smallint_with_int_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int(vec![5, 7, 9]));
    }

    #[test]
    fn test_smallint_with_bigint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::BigInt(vec![5, 7, 9]));
    }

    #[test]
    fn test_smallint_with_int128_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![4, 5, 6]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![5, 7, 9]));
    }

    #[test]
    fn test_int_with_tinyint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int(vec![11, 22, 33]));
    }

    #[test]
    fn test_int_with_smallint_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int(vec![11, 22, 33]));
    }

    #[test]
    fn test_bigint_with_tinyint_sub() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::BigInt(vec![9, 18, 27]));
    }

    #[test]
    fn test_bigint_with_smallint_sub() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::BigInt(vec![9, 18, 27]));
    }

    #[test]
    fn test_bigint_with_int_sub() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::BigInt(vec![9, 18, 27]));
    }

    #[test]
    fn test_bigint_with_int128_mul() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![2, 3, 4]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![10, 18, 28]));
    }

    #[test]
    fn test_bigint_with_decimal75_add() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1, 2, 3]);
        let rhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(400),
                TestScalar::from(500),
                TestScalar::from(600),
            ],
        );
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_int128_with_tinyint_mul() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![2, 3, 4]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![10, 18, 28]));
    }

    #[test]
    fn test_int128_with_smallint_mul() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![2, 3, 4]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![10, 18, 28]));
    }

    #[test]
    fn test_int128_with_bigint_mul() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![2, 3, 4]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![10, 18, 28]));
    }

    #[test]
    fn test_int128_with_int128_mul() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![2, 3, 4]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 6, 7]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Int128(vec![10, 18, 28]));
    }

    #[test]
    fn test_int128_with_decimal75_div() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![10, 20, 30]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(2),
                TestScalar::from(4),
                TestScalar::from(5),
            ],
        );
        let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_decimal75_with_tinyint_add() {
        let lhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(100),
                TestScalar::from(200),
                TestScalar::from(300),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_decimal75_with_smallint_sub() {
        let lhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(500),
                TestScalar::from(600),
                TestScalar::from(700),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 2, 3]);
        let result = SubOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_decimal75_with_int_mul() {
        let lhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(100),
                TestScalar::from(200),
                TestScalar::from(300),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2, 3, 4]);
        let result = MulOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_decimal75_with_bigint_div() {
        let lhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(1000),
                TestScalar::from(2000),
                TestScalar::from(3000),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![2, 4, 5]);
        let result = DivOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }

    #[test]
    fn test_decimal75_with_int128_add() {
        let lhs = decimal_column(
            10,
            2,
            vec![
                TestScalar::from(100),
                TestScalar::from(200),
                TestScalar::from(300),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3]);
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        assert!(matches!(result, OwnedColumn::Decimal75(_, _, ref values) if values.len() == 3));
    }
}
