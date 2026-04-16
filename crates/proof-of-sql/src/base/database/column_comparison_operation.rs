use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{
        slice_decimal_operation::{eq_decimal_columns, ge_decimal_columns, le_decimal_columns},
        slice_operation::{
            slice_binary_op, slice_binary_op_left_upcast, slice_binary_op_right_upcast,
        },
        ColumnType, OwnedColumn,
    },
    scalar::Scalar,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{cmp::Ord, fmt::Debug};
use num_traits::Zero;

pub trait ComparisonOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + Ord;

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>;

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>;

    /// Return an error if op is not implemented for string
    fn string_op(lhs: &[String], rhs: &[String]) -> ColumnOperationResult<Vec<bool>>;

    #[expect(clippy::too_many_lines)]
    fn owned_column_element_wise_comparison<S: Scalar>(
        lhs: &OwnedColumn<S>,
        rhs: &OwnedColumn<S>,
    ) -> ColumnOperationResult<OwnedColumn<S>> {
        if lhs.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: lhs.len(),
                len_b: rhs.len(),
            });
        }
        let result = match (&lhs, &rhs) {
            (OwnedColumn::Uint8(_), OwnedColumn::TinyInt(_)) => {
                Err(ColumnOperationError::SignedCastingError {
                    left_type: ColumnType::Uint8,
                    right_type: ColumnType::TinyInt,
                })
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::Uint8(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Uint8(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Uint8(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::TinyInt(_), OwnedColumn::Uint8(_)) => {
                return Err(ColumnOperationError::SignedCastingError {
                    left_type: ColumnType::TinyInt,
                    right_type: ColumnType::Uint8,
                })
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::TinyInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::SmallInt(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::SmallInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::Int(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::BigInt(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op_left_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::BigInt(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::Int128(lhs), OwnedColumn::TinyInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::SmallInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::Int(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::BigInt(rhs)) => {
                Ok(slice_binary_op_right_upcast(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs), OwnedColumn::Int128(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::Int128(lhs_values), OwnedColumn::Decimal75(_, _, rhs_values)) => {
                Ok(Self::decimal_op_left_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }

            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::TinyInt(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::SmallInt(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::Int(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::BigInt(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (OwnedColumn::Decimal75(_, _, lhs_values), OwnedColumn::Int128(rhs_values)) => {
                Ok(Self::decimal_op_right_upcast(
                    lhs_values,
                    rhs_values,
                    lhs.column_type(),
                    rhs.column_type(),
                ))
            }
            (
                OwnedColumn::Decimal75(_, _, lhs_values),
                OwnedColumn::Decimal75(_, _, rhs_values),
            ) => Ok(Self::decimal_op_left_upcast(
                lhs_values,
                rhs_values,
                lhs.column_type(),
                rhs.column_type(),
            )),

            (OwnedColumn::Boolean(lhs), OwnedColumn::Boolean(rhs)) => {
                Ok(slice_binary_op(lhs, rhs, Self::op))
            }
            (OwnedColumn::VarChar(lhs), OwnedColumn::VarChar(rhs)) => Self::string_op(lhs, rhs),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "ComparisonOp".to_string(),
                left_type: lhs.column_type(),
                right_type: rhs.column_type(),
            }),
        }?;
        Ok(OwnedColumn::Boolean(result))
    }
}

pub struct EqualOp {}
impl ComparisonOp for EqualOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + PartialEq,
    {
        l == r
    }

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + PartialEq + Zero + Into<S>,
    {
        eq_decimal_columns(lhs, rhs, left_column_type, right_column_type)
    }

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + PartialEq + Zero + Into<S>,
    {
        eq_decimal_columns(rhs, lhs, right_column_type, left_column_type)
    }

    fn string_op(lhs: &[String], rhs: &[String]) -> ColumnOperationResult<Vec<bool>> {
        Ok(lhs.iter().zip(rhs.iter()).map(|(l, r)| l == r).collect())
    }
}

pub struct GreaterThanOp {}
impl ComparisonOp for GreaterThanOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + Ord,
    {
        l > r
    }

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        le_decimal_columns(lhs, rhs, left_column_type, right_column_type)
            .iter()
            .map(|b| !b)
            .collect()
    }

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        ge_decimal_columns(rhs, lhs, right_column_type, left_column_type)
            .iter()
            .map(|b| !b)
            .collect()
    }

    fn string_op(_lhs: &[String], _rhs: &[String]) -> ColumnOperationResult<Vec<bool>> {
        Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: ">".to_string(),
            left_type: ColumnType::VarChar,
            right_type: ColumnType::VarChar,
        })
    }
}

pub struct LessThanOp {}
impl ComparisonOp for LessThanOp {
    fn op<T>(l: &T, r: &T) -> bool
    where
        T: Debug + Ord,
    {
        l < r
    }

    fn decimal_op_left_upcast<S, T>(
        lhs: &[T],
        rhs: &[S],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        ge_decimal_columns(lhs, rhs, left_column_type, right_column_type)
            .iter()
            .map(|b| !b)
            .collect()
    }

    fn decimal_op_right_upcast<S, T>(
        lhs: &[S],
        rhs: &[T],
        left_column_type: ColumnType,
        right_column_type: ColumnType,
    ) -> Vec<bool>
    where
        S: Scalar,
        T: Copy + Debug + Ord + Zero + Into<S>,
    {
        le_decimal_columns(rhs, lhs, right_column_type, left_column_type)
            .iter()
            .map(|b| !b)
            .collect()
    }

    fn string_op(_lhs: &[String], _rhs: &[String]) -> ColumnOperationResult<Vec<bool>> {
        Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "<".to_string(),
            left_type: ColumnType::VarChar,
            right_type: ColumnType::VarChar,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};
    use alloc::{string::ToString, vec};

    #[test]
    fn equal_string_op_compares_each_varchar_pair() {
        let lhs = ["Space", "and", "Time"].map(ToString::to_string);
        let rhs = ["Space", "or", "Time"].map(ToString::to_string);

        let actual = <EqualOp as ComparisonOp>::string_op(&lhs, &rhs).unwrap();

        assert_eq!(actual, vec![true, false, true]);
    }

    #[test]
    fn ordering_string_ops_reject_varchar_inputs() {
        let lhs = ["a", "b"].map(ToString::to_string);
        let rhs = ["a", "c"].map(ToString::to_string);

        let gt_err = <GreaterThanOp as ComparisonOp>::string_op(&lhs, &rhs).unwrap_err();
        assert_eq!(
            gt_err,
            ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: ">".to_string(),
                left_type: ColumnType::VarChar,
                right_type: ColumnType::VarChar,
            }
        );

        let lt_err = <LessThanOp as ComparisonOp>::string_op(&lhs, &rhs).unwrap_err();
        assert_eq!(
            lt_err,
            ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "<".to_string(),
                left_type: ColumnType::VarChar,
                right_type: ColumnType::VarChar,
            }
        );
    }

    #[test]
    fn comparison_rejects_unsupported_signed_uint8_mixed_comparisons() {
        let uint8s = OwnedColumn::<TestScalar>::Uint8(vec![1, 2, 3]);
        let tinyints = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);

        let left_err =
            <EqualOp as ComparisonOp>::owned_column_element_wise_comparison(&uint8s, &tinyints)
                .unwrap_err();
        assert_eq!(
            left_err,
            ColumnOperationError::SignedCastingError {
                left_type: ColumnType::Uint8,
                right_type: ColumnType::TinyInt,
            }
        );

        let right_err =
            <EqualOp as ComparisonOp>::owned_column_element_wise_comparison(&tinyints, &uint8s)
                .unwrap_err();
        assert_eq!(
            right_err,
            ColumnOperationError::SignedCastingError {
                left_type: ColumnType::TinyInt,
                right_type: ColumnType::Uint8,
            }
        );
    }

    #[test]
    fn decimal_equality_helpers_are_symmetric_for_integer_upcasts() {
        let integers = [1_i16, 2, 4];
        let decimals = [10_i64, 20, 30].map(TestScalar::from);
        let integer_type = ColumnType::SmallInt;
        let decimal_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 1);

        let left_upcast = <EqualOp as ComparisonOp>::decimal_op_left_upcast::<TestScalar, i16>(
            &integers,
            &decimals,
            integer_type,
            decimal_type,
        );
        assert_eq!(left_upcast, vec![true, true, false]);

        let right_upcast = <EqualOp as ComparisonOp>::decimal_op_right_upcast::<TestScalar, i16>(
            &decimals,
            &integers,
            decimal_type,
            integer_type,
        );
        assert_eq!(right_upcast, vec![true, true, false]);
    }

    #[test]
    fn decimal_ordering_helpers_flip_inclusive_checks_consistently() {
        let integers = [0_i16, 2, 4];
        let decimals = [10_i64, 20, 30].map(TestScalar::from);
        let integer_type = ColumnType::SmallInt;
        let decimal_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 1);

        let gt_left = <GreaterThanOp as ComparisonOp>::decimal_op_left_upcast::<TestScalar, i16>(
            &integers,
            &decimals,
            integer_type,
            decimal_type,
        );
        assert_eq!(gt_left, vec![false, false, true]);

        let gt_right =
            <GreaterThanOp as ComparisonOp>::decimal_op_right_upcast::<TestScalar, i16>(
                &decimals,
                &integers,
                decimal_type,
                integer_type,
            );
        assert_eq!(gt_right, vec![true, false, false]);

        let lt_left = <LessThanOp as ComparisonOp>::decimal_op_left_upcast::<TestScalar, i16>(
            &integers,
            &decimals,
            integer_type,
            decimal_type,
        );
        assert_eq!(lt_left, vec![true, false, false]);

        let lt_right = <LessThanOp as ComparisonOp>::decimal_op_right_upcast::<TestScalar, i16>(
            &decimals,
            &integers,
            decimal_type,
            integer_type,
        );
        assert_eq!(lt_right, vec![false, false, true]);
    }
}
