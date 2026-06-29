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

    // --- EqualOp::op ---

    #[test]
    fn we_can_compare_equal_integers_with_equal_op() {
        assert!(EqualOp::op::<i64>(&7, &7));
        assert!(!EqualOp::op::<i64>(&7, &8));
        assert!(EqualOp::op::<bool>(&true, &true));
        assert!(!EqualOp::op::<bool>(&false, &true));
    }

    // --- GreaterThanOp::op ---

    #[test]
    fn we_can_compare_integers_with_greater_than_op() {
        assert!(GreaterThanOp::op::<i64>(&8, &7));
        assert!(!GreaterThanOp::op::<i64>(&7, &8));
        assert!(!GreaterThanOp::op::<i64>(&7, &7));
    }

    // --- LessThanOp::op ---

    #[test]
    fn we_can_compare_integers_with_less_than_op() {
        assert!(LessThanOp::op::<i64>(&7, &8));
        assert!(!LessThanOp::op::<i64>(&8, &7));
        assert!(!LessThanOp::op::<i64>(&7, &7));
    }

    // --- string_op ---

    #[test]
    fn we_can_compare_equal_strings_with_equal_op() {
        let lhs = vec!["alpha".to_string(), "beta".to_string()];
        let rhs = vec!["alpha".to_string(), "gamma".to_string()];
        let result = EqualOp::string_op(&lhs, &rhs).unwrap();
        assert_eq!(result, vec![true, false]);
    }

    #[test]
    fn we_cannot_compare_strings_with_greater_than_op() {
        let lhs = vec!["a".to_string()];
        let rhs = vec!["b".to_string()];
        assert!(matches!(
            GreaterThanOp::string_op(&lhs, &rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_compare_strings_with_less_than_op() {
        let lhs = vec!["a".to_string()];
        let rhs = vec!["b".to_string()];
        assert!(matches!(
            LessThanOp::string_op(&lhs, &rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    // --- owned_column_element_wise_comparison: error paths ---

    #[test]
    fn we_cannot_compare_columns_of_different_lengths() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true]);
        assert!(matches!(
            EqualOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
        assert!(matches!(
            GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
        assert!(matches!(
            LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
    }

    #[test]
    fn we_cannot_compare_incompatible_column_types() {
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string()]);
        assert!(matches!(
            EqualOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_compare_uint8_with_tinyint() {
        let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1]);
        assert!(matches!(
            EqualOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
    }

    // --- Boolean × Boolean ---

    #[test]
    fn we_can_compare_boolean_columns_for_equality() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, true, false])
        );
    }

    // --- VarChar × VarChar ---

    #[test]
    fn we_can_compare_varchar_columns_for_equality() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["foo".to_string(), "bar".to_string()]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["foo".to_string(), "baz".to_string()]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }

    #[test]
    fn we_cannot_compare_varchar_columns_with_ordering_ops() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string()]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["b".to_string()]);
        assert!(matches!(
            GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
        assert!(matches!(
            LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    // --- Decimal75 × integer: decimal_op_left_upcast and decimal_op_right_upcast ---

    #[test]
    fn we_can_compare_uint8_column_with_decimal75_column() {
        let lhs = OwnedColumn::<TestScalar>::Uint8(vec![5, 10]);
        let rhs_scalars: Vec<TestScalar> = vec![5i64.into(), 99i64.into()];
        let rhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(5).unwrap(),
            0,
            rhs_scalars,
        );
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }

    #[test]
    fn we_can_compare_decimal75_column_with_bigint_column() {
        let lhs_scalars: Vec<TestScalar> = vec![100i64.into(), 200i64.into()];
        let lhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(5).unwrap(),
            0,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![100, 300]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }

    #[test]
    fn we_can_compare_decimal75_columns_with_each_other() {
        // 100 at scale 2 == 10 at scale 1  (both represent 1.0)
        // 200 at scale 2 != 10 at scale 1  (2.0 vs 1.0)
        let lhs_scalars: Vec<TestScalar> = vec![100i64.into(), 200i64.into()];
        let rhs_scalars: Vec<TestScalar> = vec![10i64.into(), 10i64.into()];
        let lhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(5).unwrap(),
            2,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(5).unwrap(),
            1,
            rhs_scalars,
        );
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }

    #[test]
    fn we_can_apply_greater_than_to_decimal75_and_bigint_columns() {
        let lhs_scalars: Vec<TestScalar> = vec![50i64.into(), 5i64.into()];
        let lhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(5).unwrap(),
            0,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![10, 10]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }

    #[test]
    fn we_can_apply_less_than_to_decimal75_and_bigint_columns() {
        let lhs_scalars: Vec<TestScalar> = vec![5i64.into(), 50i64.into()];
        let lhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(5).unwrap(),
            0,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![10, 10]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }

    #[test]
    fn we_can_compare_decimal75_column_with_tinyint_column() {
        let lhs_scalars: Vec<TestScalar> = vec![3i64.into(), 7i64.into()];
        let lhs = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(3).unwrap(),
            0,
            lhs_scalars,
        );
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![3i8, 8]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Boolean(vec![true, false])
        );
    }
}
