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
    use crate::base::{database::ColumnOperationError, scalar::test_scalar::TestScalar};

    #[test]
    fn test_equal_op_bigint_same() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, true, true]));
    }

    #[test]
    fn test_equal_op_bigint_partial() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 9, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_equal_op_empty() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![]));
    }

    #[test]
    fn test_equal_op_length_mismatch() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { len_a: 2, len_b: 1 })
        ));
    }

    #[test]
    fn test_equal_op_varchar() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string(), "b".to_string()]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string(), "c".to_string()]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false]));
    }

    #[test]
    fn test_equal_op_int128() {
        let lhs = OwnedColumn::<TestScalar>::Int128(vec![100, 200]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![100, 201]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false]));
    }

    #[test]
    fn test_equal_op_uint8_upcast_to_smallint() {
        let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 4]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, true, false]));
    }

    #[test]
    fn test_equal_op_signed_unsigned_error() {
        let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(result, Err(ColumnOperationError::SignedCastingError { .. })));
    }

    #[test]
    fn test_equal_op_tinyint() {
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![5, 10, -3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![5, 9, -3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_equal_op_boolean() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, true]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    // ── GreaterThanOp ─────────────────────────────────────────────────────────

    #[test]
    fn test_gt_bigint() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![3, 1, 2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 2]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_gt_varchar_returns_error() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string()]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["b".to_string()]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn test_gt_int_upcast_bigint() {
        let lhs = OwnedColumn::<TestScalar>::Int(vec![10, 5, 1]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![5, 5, 2]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_gt_length_mismatch() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { len_a: 3, len_b: 2 })
        ));
    }

    #[test]
    fn test_gt_tinyint() {
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![10, 5, -1]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![5, 10, -2]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_gt_empty() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![]));
    }

    // ── LessThanOp ───────────────────────────────────────────────────────────

    #[test]
    fn test_lt_bigint() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 5, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![5, 5, 2]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_lt_varchar_returns_error() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string()]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["b".to_string()]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn test_lt_smallint_upcast_int() {
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 10, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![5, 5, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_lt_empty() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![]));
    }

    #[test]
    fn test_lt_length_mismatch() {
        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 })
        ));
    }

    #[test]
    fn test_lt_tinyint_negative() {
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![-5, 0, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![0, 0, 2]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }
}
