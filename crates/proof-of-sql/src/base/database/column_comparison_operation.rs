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
    use crate::base::scalar::Curve25519Scalar;

    #[test]
    fn test_equal_op_basic_comparison() {
        assert!(EqualOp::op(&5, &5));
        assert!(!EqualOp::op(&5, &10));
        assert!(EqualOp::op(&true, &true));
        assert!(!EqualOp::op(&true, &false));
    }

    #[test]
    fn test_equal_op_string_comparison() {
        let lhs = vec!["hello".to_string(), "world".to_string()];
        let rhs = vec!["hello".to_string(), "test".to_string()];
        let result = EqualOp::string_op(&lhs, &rhs).unwrap();
        assert_eq!(result, vec![true, false]);
    }

    #[test]
    fn test_greater_than_op_basic_comparison() {
        assert!(GreaterThanOp::op(&10, &5));
        assert!(!GreaterThanOp::op(&5, &10));
        assert!(!GreaterThanOp::op(&5, &5));
    }

    #[test]
    fn test_greater_than_op_string_comparison_returns_error() {
        let lhs = vec!["hello".to_string()];
        let rhs = vec!["world".to_string()];
        let result = GreaterThanOp::string_op(&lhs, &rhs);
        assert!(result.is_err());
        match result {
            Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator,
                left_type,
                right_type,
            }) => {
                assert_eq!(operator, ">");
                assert_eq!(left_type, ColumnType::VarChar);
                assert_eq!(right_type, ColumnType::VarChar);
            }
            _ => panic!("Expected BinaryOperationInvalidColumnType error"),
        }
    }

    #[test]
    fn test_less_than_op_basic_comparison() {
        assert!(LessThanOp::op(&5, &10));
        assert!(!LessThanOp::op(&10, &5));
        assert!(!LessThanOp::op(&5, &5));
    }

    #[test]
    fn test_less_than_op_string_comparison_returns_error() {
        let lhs = vec!["hello".to_string()];
        let rhs = vec!["world".to_string()];
        let result = LessThanOp::string_op(&lhs, &rhs);
        assert!(result.is_err());
        match result {
            Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator,
                left_type,
                right_type,
            }) => {
                assert_eq!(operator, "<");
                assert_eq!(left_type, ColumnType::VarChar);
                assert_eq!(right_type, ColumnType::VarChar);
            }
            _ => panic!("Expected BinaryOperationInvalidColumnType error"),
        }
    }

    #[test]
    fn test_equal_op_owned_column_same_type() {
        let lhs = OwnedColumn::<Curve25519Scalar>::Int128(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(vec![1, 5, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false, true]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_equal_op_owned_column_type_upcast() {
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(vec![1, 5, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false, true]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_equal_op_owned_column_boolean() {
        let lhs = OwnedColumn::<Curve25519Scalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Boolean(vec![true, true, false]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false, false]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_equal_op_owned_column_varchar() {
        let lhs = OwnedColumn::<Curve25519Scalar>::VarChar(vec![
            "hello".to_string(),
            "world".to_string(),
        ]);
        let rhs = OwnedColumn::<Curve25519Scalar>::VarChar(vec![
            "hello".to_string(),
            "test".to_string(),
        ]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_owned_column_comparison_different_lengths() {
        let lhs = OwnedColumn::<Curve25519Scalar>::Int128(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(vec![1, 2]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(result.is_err());
        match result {
            Err(ColumnOperationError::DifferentColumnLength { len_a, len_b }) => {
                assert_eq!(len_a, 3);
                assert_eq!(len_b, 2);
            }
            _ => panic!("Expected DifferentColumnLength error"),
        }
    }

    #[test]
    fn test_owned_column_comparison_uint8_tinyint_error() {
        let lhs = OwnedColumn::<Curve25519Scalar>::Uint8(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::TinyInt(vec![1, 2, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(result.is_err());
        match result {
            Err(ColumnOperationError::SignedCastingError {
                left_type,
                right_type,
            }) => {
                assert_eq!(left_type, ColumnType::Uint8);
                assert_eq!(right_type, ColumnType::TinyInt);
            }
            _ => panic!("Expected SignedCastingError"),
        }
    }

    #[test]
    fn test_owned_column_comparison_tinyint_uint8_error() {
        let lhs = OwnedColumn::<Curve25519Scalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Uint8(vec![1, 2, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(result.is_err());
        match result {
            Err(ColumnOperationError::SignedCastingError {
                left_type,
                right_type,
            }) => {
                assert_eq!(left_type, ColumnType::TinyInt);
                assert_eq!(right_type, ColumnType::Uint8);
            }
            _ => panic!("Expected SignedCastingError"),
        }
    }

    #[test]
    fn test_greater_than_owned_column() {
        let lhs = OwnedColumn::<Curve25519Scalar>::BigInt(vec![10, 5, 8]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(vec![5, 10, 8]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false, false]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_less_than_owned_column() {
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![5, 10, 8]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(vec![10, 5, 8]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false, false]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_comparison_with_mixed_integer_types() {
        // Test SmallInt vs Int
        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(vec![5, 10]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(vec![5, 8]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        match result {
            OwnedColumn::Boolean(b) => {
                assert_eq!(b, vec![true, false]);
            }
            _ => panic!("Expected Boolean column"),
        }
    }

    #[test]
    fn test_comparison_invalid_column_types() {
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::VarChar(vec!["test".to_string(); 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(result.is_err());
        match result {
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. }) => {}
            _ => panic!("Expected BinaryOperationInvalidColumnType error"),
        }
    }
}
