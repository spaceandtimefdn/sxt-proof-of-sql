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

    fn decimal_column(
        precision: u8,
        scale: i8,
        values: Vec<TestScalar>,
    ) -> OwnedColumn<TestScalar> {
        OwnedColumn::Decimal75(Precision::new(precision).unwrap(), scale, values)
    }

    // Uint8 tests
    #[test]
    fn test_uint8_with_tinyint_comparison_error() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
    }

    #[test]
    fn test_uint8_with_uint8_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 4, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_uint8_with_smallint_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![5, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 4, 3]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_uint8_with_int_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 5, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2, 4, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_uint8_with_bigint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_uint8_with_int128_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_uint8_with_decimal75_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(2),
                TestScalar::from(1),
                TestScalar::from(3),
            ],
        );
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    // TinyInt tests
    #[test]
    fn test_tinyint_with_uint8_comparison_error() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
    }

    #[test]
    fn test_tinyint_with_tinyint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 4, 3]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_tinyint_with_smallint_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![5, 2, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 4, 3]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_tinyint_with_int_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 5, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2, 4, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_tinyint_with_bigint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_tinyint_with_int128_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_tinyint_with_decimal75_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(2),
                TestScalar::from(1),
                TestScalar::from(3),
            ],
        );
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    // SmallInt tests
    #[test]
    fn test_smallint_with_smallint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_smallint_with_int_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_smallint_with_bigint_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1, 5, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![2, 4, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_smallint_with_int128_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![100, 200, 300]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![100, 250, 300]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_smallint_with_decimal75_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 20, 30]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(5),
                TestScalar::from(25),
                TestScalar::from(30),
            ],
        );
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    // Int tests
    #[test]
    fn test_int_with_tinyint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_int_with_int_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_int_with_bigint_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1, 5, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![2, 4, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_int_with_int128_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![100, 200, 300]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![100, 250, 300]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_int_with_decimal75_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![10, 20, 30]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(5),
                TestScalar::from(25),
                TestScalar::from(30),
            ],
        );
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    // BigInt tests
    #[test]
    fn test_bigint_with_tinyint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_bigint_with_smallint_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_bigint_with_int_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1, 5, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2, 4, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_bigint_with_bigint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![100, 200, 300]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![100, 250, 300]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_bigint_with_int128_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_bigint_with_decimal75_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1, 5, 3]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(2),
                TestScalar::from(4),
                TestScalar::from(3),
            ],
        );
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    // Int128 tests
    #[test]
    fn test_int128_with_tinyint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_int128_with_smallint_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_int128_with_int_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 5, 3]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![2, 4, 3]);
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_int128_with_bigint_eq() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![100, 200, 300]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![100, 250, 300]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_int128_with_int128_gt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![10, 20, 30]);
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    #[test]
    fn test_int128_with_decimal75_lt() {
        let lhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 5, 3]);
        let rhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(2),
                TestScalar::from(4),
                TestScalar::from(3),
            ],
        );
        let result = LessThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }

    // Decimal75 tests
    #[test]
    fn test_decimal75_with_smallint_eq() {
        let lhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(10),
                TestScalar::from(20),
                TestScalar::from(30),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 25, 30]);
        let result = EqualOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, true]));
    }

    #[test]
    fn test_decimal75_with_int128_gt() {
        let lhs = decimal_column(
            10,
            0,
            vec![
                TestScalar::from(10),
                TestScalar::from(20),
                TestScalar::from(30),
            ],
        );
        let rhs: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![5, 25, 30]);
        let result = GreaterThanOp::owned_column_element_wise_comparison(&lhs, &rhs).unwrap();
        assert_eq!(result, OwnedColumn::Boolean(vec![true, false, false]));
    }
}
