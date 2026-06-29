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
    use super::{AddOp, ArithmeticOp};
    use crate::base::{
        database::{ColumnOperationError, OwnedColumn},
        math::decimal::Precision,
        scalar::test_scalar::TestScalar,
    };
    use alloc::{string::ToString, vec, vec::Vec};

    type TestColumn = OwnedColumn<TestScalar>;

    fn scalars(values: [i128; 2]) -> Vec<TestScalar> {
        values.into_iter().map(TestScalar::from).collect()
    }

    fn decimal(values: [i128; 2]) -> TestColumn {
        OwnedColumn::Decimal75(Precision::new(5).unwrap(), 2, scalars(values))
    }

    fn assert_adds(lhs: TestColumn, rhs: TestColumn, expected: TestColumn) {
        assert_eq!(
            AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap(),
            expected
        );
    }

    fn assert_decimal_adds(lhs: TestColumn, rhs: TestColumn, expected_values: [i128; 2]) {
        let result = AddOp::owned_column_element_wise_arithmetic(&lhs, &rhs).unwrap();
        let OwnedColumn::Decimal75(_, scale, values) = result else {
            panic!("expected Decimal75 result");
        };
        assert_eq!(scale, 2);
        assert_eq!(values, scalars(expected_values));
    }

    #[test]
    fn we_can_add_integer_columns_across_supported_upcasts() {
        let cases = vec![
            (
                OwnedColumn::Uint8(vec![1_u8, 2]),
                OwnedColumn::Uint8(vec![3_u8, 4]),
                OwnedColumn::Uint8(vec![4_u8, 6]),
            ),
            (
                OwnedColumn::Uint8(vec![1_u8, 2]),
                OwnedColumn::SmallInt(vec![3_i16, 4]),
                OwnedColumn::SmallInt(vec![4_i16, 6]),
            ),
            (
                OwnedColumn::Uint8(vec![1_u8, 2]),
                OwnedColumn::Int(vec![3_i32, 4]),
                OwnedColumn::Int(vec![4_i32, 6]),
            ),
            (
                OwnedColumn::Uint8(vec![1_u8, 2]),
                OwnedColumn::BigInt(vec![3_i64, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::Uint8(vec![1_u8, 2]),
                OwnedColumn::Int128(vec![3_i128, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::TinyInt(vec![1_i8, 2]),
                OwnedColumn::TinyInt(vec![3_i8, 4]),
                OwnedColumn::TinyInt(vec![4_i8, 6]),
            ),
            (
                OwnedColumn::TinyInt(vec![1_i8, 2]),
                OwnedColumn::SmallInt(vec![3_i16, 4]),
                OwnedColumn::SmallInt(vec![4_i16, 6]),
            ),
            (
                OwnedColumn::TinyInt(vec![1_i8, 2]),
                OwnedColumn::Int(vec![3_i32, 4]),
                OwnedColumn::Int(vec![4_i32, 6]),
            ),
            (
                OwnedColumn::TinyInt(vec![1_i8, 2]),
                OwnedColumn::BigInt(vec![3_i64, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::TinyInt(vec![1_i8, 2]),
                OwnedColumn::Int128(vec![3_i128, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::SmallInt(vec![1_i16, 2]),
                OwnedColumn::TinyInt(vec![3_i8, 4]),
                OwnedColumn::SmallInt(vec![4_i16, 6]),
            ),
            (
                OwnedColumn::SmallInt(vec![1_i16, 2]),
                OwnedColumn::SmallInt(vec![3_i16, 4]),
                OwnedColumn::SmallInt(vec![4_i16, 6]),
            ),
            (
                OwnedColumn::SmallInt(vec![1_i16, 2]),
                OwnedColumn::Int(vec![3_i32, 4]),
                OwnedColumn::Int(vec![4_i32, 6]),
            ),
            (
                OwnedColumn::SmallInt(vec![1_i16, 2]),
                OwnedColumn::BigInt(vec![3_i64, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::SmallInt(vec![1_i16, 2]),
                OwnedColumn::Int128(vec![3_i128, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::Int(vec![1_i32, 2]),
                OwnedColumn::TinyInt(vec![3_i8, 4]),
                OwnedColumn::Int(vec![4_i32, 6]),
            ),
            (
                OwnedColumn::Int(vec![1_i32, 2]),
                OwnedColumn::SmallInt(vec![3_i16, 4]),
                OwnedColumn::Int(vec![4_i32, 6]),
            ),
            (
                OwnedColumn::Int(vec![1_i32, 2]),
                OwnedColumn::Int(vec![3_i32, 4]),
                OwnedColumn::Int(vec![4_i32, 6]),
            ),
            (
                OwnedColumn::Int(vec![1_i32, 2]),
                OwnedColumn::BigInt(vec![3_i64, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::Int(vec![1_i32, 2]),
                OwnedColumn::Int128(vec![3_i128, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::BigInt(vec![1_i64, 2]),
                OwnedColumn::TinyInt(vec![3_i8, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::BigInt(vec![1_i64, 2]),
                OwnedColumn::SmallInt(vec![3_i16, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::BigInt(vec![1_i64, 2]),
                OwnedColumn::Int(vec![3_i32, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::BigInt(vec![1_i64, 2]),
                OwnedColumn::BigInt(vec![3_i64, 4]),
                OwnedColumn::BigInt(vec![4_i64, 6]),
            ),
            (
                OwnedColumn::BigInt(vec![1_i64, 2]),
                OwnedColumn::Int128(vec![3_i128, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::Int128(vec![1_i128, 2]),
                OwnedColumn::TinyInt(vec![3_i8, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::Int128(vec![1_i128, 2]),
                OwnedColumn::SmallInt(vec![3_i16, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::Int128(vec![1_i128, 2]),
                OwnedColumn::Int(vec![3_i32, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::Int128(vec![1_i128, 2]),
                OwnedColumn::BigInt(vec![3_i64, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
            (
                OwnedColumn::Int128(vec![1_i128, 2]),
                OwnedColumn::Int128(vec![3_i128, 4]),
                OwnedColumn::Int128(vec![4_i128, 6]),
            ),
        ];

        for (lhs, rhs, expected) in cases {
            assert_adds(lhs, rhs, expected);
        }
    }

    #[test]
    fn we_can_add_decimal_columns_across_integer_upcasts() {
        let integer_columns = vec![
            OwnedColumn::Uint8(vec![1_u8, 2]),
            OwnedColumn::TinyInt(vec![1_i8, 2]),
            OwnedColumn::SmallInt(vec![1_i16, 2]),
            OwnedColumn::Int(vec![1_i32, 2]),
            OwnedColumn::BigInt(vec![1_i64, 2]),
            OwnedColumn::Int128(vec![1_i128, 2]),
        ];

        for integer_column in integer_columns {
            assert_decimal_adds(integer_column.clone(), decimal([3, 4]), [103, 204]);
            if !matches!(integer_column, OwnedColumn::Uint8(_)) {
                assert_decimal_adds(decimal([3, 4]), integer_column, [103, 204]);
            }
        }

        assert_decimal_adds(decimal([3, 4]), decimal([5, 6]), [8, 10]);
    }

    #[test]
    fn arithmetic_reports_signed_casting_and_invalid_type_errors() {
        assert!(matches!(
            AddOp::owned_column_element_wise_arithmetic(
                &OwnedColumn::<TestScalar>::Uint8(vec![1_u8]),
                &OwnedColumn::<TestScalar>::Uint8(vec![2_u8, 3])
            ),
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let uint8 = OwnedColumn::<TestScalar>::Uint8(vec![1_u8, 2]);
        let tinyint = OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 4]);

        assert!(matches!(
            AddOp::owned_column_element_wise_arithmetic(&uint8, &tinyint),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
        assert!(matches!(
            AddOp::owned_column_element_wise_arithmetic(&tinyint, &uint8),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));

        let strings = OwnedColumn::<TestScalar>::VarChar(vec!["1".to_string(), "2".to_string()]);
        let bigint = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2]);
        assert!(matches!(
            AddOp::owned_column_element_wise_arithmetic(&strings, &bigint),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }
}
