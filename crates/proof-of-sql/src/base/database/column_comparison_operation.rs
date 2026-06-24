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
    use super::{ComparisonOp, EqualOp, GreaterThanOp, LessThanOp};
    use alloc::{string::ToString, vec};

    // EqualOp::op tests
    #[test]
    fn equal_op_i64_equal_values_returns_true() {
        assert!(EqualOp::op::<i64>(&5, &5));
    }

    #[test]
    fn equal_op_i64_different_values_returns_false() {
        assert!(!EqualOp::op::<i64>(&5, &6));
    }

    #[test]
    fn equal_op_i32_equal_values_returns_true() {
        assert!(EqualOp::op::<i32>(&0, &0));
    }

    #[test]
    fn equal_op_i32_different_values_returns_false() {
        assert!(!EqualOp::op::<i32>(&1, &2));
    }

    #[test]
    fn equal_op_bool_equal_values_returns_true() {
        assert!(EqualOp::op::<bool>(&true, &true));
        assert!(EqualOp::op::<bool>(&false, &false));
    }

    #[test]
    fn equal_op_bool_different_values_returns_false() {
        assert!(!EqualOp::op::<bool>(&true, &false));
    }

    #[test]
    fn equal_op_i64_negative_equal_values_returns_true() {
        assert!(EqualOp::op::<i64>(&-5, &-5));
    }

    #[test]
    fn equal_op_i64_min_equal_to_itself() {
        assert!(EqualOp::op::<i64>(&i64::MIN, &i64::MIN));
    }

    // GreaterThanOp::op tests
    #[test]
    fn greater_than_op_i64_greater_value_returns_true() {
        assert!(GreaterThanOp::op::<i64>(&5, &3));
    }

    #[test]
    fn greater_than_op_i64_lesser_value_returns_false() {
        assert!(!GreaterThanOp::op::<i64>(&3, &5));
    }

    #[test]
    fn greater_than_op_i64_equal_values_returns_false() {
        assert!(!GreaterThanOp::op::<i64>(&5, &5));
    }

    #[test]
    fn greater_than_op_i32_positive() {
        assert!(GreaterThanOp::op::<i32>(&10, &0));
    }

    #[test]
    fn greater_than_op_i64_negative_both() {
        assert!(GreaterThanOp::op::<i64>(&-1, &-5));
    }

    #[test]
    fn greater_than_op_positive_over_negative() {
        assert!(GreaterThanOp::op::<i64>(&1, &-1));
    }

    // LessThanOp::op tests
    #[test]
    fn less_than_op_i64_lesser_value_returns_true() {
        assert!(LessThanOp::op::<i64>(&3, &5));
    }

    #[test]
    fn less_than_op_i64_greater_value_returns_false() {
        assert!(!LessThanOp::op::<i64>(&5, &3));
    }

    #[test]
    fn less_than_op_i64_equal_values_returns_false() {
        assert!(!LessThanOp::op::<i64>(&5, &5));
    }

    #[test]
    fn less_than_op_negative_less_than_zero() {
        assert!(LessThanOp::op::<i64>(&-5, &0));
    }

    #[test]
    fn less_than_op_i32_zero_less_than_positive() {
        assert!(LessThanOp::op::<i32>(&0, &1));
    }

    #[test]
    fn less_than_op_negative_both() {
        assert!(LessThanOp::op::<i64>(&-5, &-1));
    }

    // EqualOp::string_op tests
    #[test]
    fn equal_op_string_op_equal_strings_returns_true() {
        let result = EqualOp::string_op(&["hello".to_string()], &["hello".to_string()]);
        assert_eq!(result.unwrap(), vec![true]);
    }

    #[test]
    fn equal_op_string_op_different_strings_returns_false() {
        let result = EqualOp::string_op(&["hello".to_string()], &["world".to_string()]);
        assert_eq!(result.unwrap(), vec![false]);
    }

    #[test]
    fn equal_op_string_op_empty_slices_returns_empty() {
        let result = EqualOp::string_op(&[], &[]);
        assert_eq!(result.unwrap(), vec![]);
    }

    #[test]
    fn equal_op_string_op_multiple_elements() {
        let lhs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let rhs = vec!["a".to_string(), "x".to_string(), "c".to_string()];
        let result = EqualOp::string_op(&lhs, &rhs).unwrap();
        assert_eq!(result, vec![true, false, true]);
    }

    // GreaterThanOp::string_op returns error
    #[test]
    fn greater_than_op_string_op_returns_error() {
        let result = GreaterThanOp::string_op(&["a".to_string()], &["b".to_string()]);
        assert!(result.is_err());
    }

    // LessThanOp::string_op returns error
    #[test]
    fn less_than_op_string_op_returns_error() {
        let result = LessThanOp::string_op(&["a".to_string()], &["b".to_string()]);
        assert!(result.is_err());
    }

    // owned_column_element_wise_comparison with different lengths
    #[test]
    fn comparison_on_different_length_columns_returns_error() {
        use crate::base::{database::OwnedColumn, scalar::test_scalar::TestScalar};
        let lhs = OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 2]);
        assert!(EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).is_err());
    }

    // BigInt column element-wise equality
    #[test]
    fn equal_op_bigint_columns_element_wise() {
        use crate::base::{database::OwnedColumn, scalar::test_scalar::TestScalar};
        let lhs = OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 5, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::<TestScalar>::Boolean(alloc::vec![true, false, true]));
    }
}
