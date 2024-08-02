use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{column_operation::*, OwnedColumn},
    scalar::Scalar,
};
use core::ops::{Add, Mul, Sub};
use proof_of_sql_parser::intermediate_ast::BinaryOperator;

impl<S: Scalar> Add for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength(
                self.len(),
                rhs.len(),
            ));
        }
        match (&self, &rhs) {
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::SmallInt(try_add_slices(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Int(try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Int(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Int(try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(try_add_slices(lhs, rhs)?)),
            (Self::Int(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::Int(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::BigInt(try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(lhs), Self::Int(rhs)) => {
                Ok(Self::BigInt(try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(lhs), Self::BigInt(rhs)) => Ok(Self::BigInt(try_add_slices(lhs, rhs)?)),
            (Self::BigInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_add_slices_with_casting(lhs, rhs)?))
            }
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Int128(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Int128(try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(lhs), Self::Int(rhs)) => {
                Ok(Self::Int128(try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Int128(try_add_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(lhs), Self::Int128(rhs)) => Ok(Self::Int128(try_add_slices(lhs, rhs)?)),
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_add_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Add,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

impl<S: Scalar> Sub for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength(
                self.len(),
                rhs.len(),
            ));
        }
        match (&self, &rhs) {
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::SmallInt(try_subtract_slices(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Int(try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Int(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Int(try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(try_subtract_slices(lhs, rhs)?)),
            (Self::Int(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::Int(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::BigInt(try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::BigInt(lhs), Self::Int(rhs)) => {
                Ok(Self::BigInt(try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::BigInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_subtract_slices(lhs, rhs)?))
            }
            (Self::BigInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_subtract_slices_left_upcast(lhs, rhs)?))
            }
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Int128(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Int128(try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int128(lhs), Self::Int(rhs)) => {
                Ok(Self::Int128(try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int128(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Int128(try_subtract_slices_right_upcast(lhs, rhs)?))
            }
            (Self::Int128(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_subtract_slices(lhs, rhs)?))
            }
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_subtract_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Subtract,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

impl<S: Scalar> Mul for OwnedColumn<S> {
    type Output = ColumnOperationResult<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength(
                self.len(),
                rhs.len(),
            ));
        }
        match (&self, &rhs) {
            (Self::SmallInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::SmallInt(try_multiply_slices(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::Int(rhs)) => {
                Ok(Self::Int(try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::SmallInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Int(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Int(try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(try_multiply_slices(lhs, rhs)?)),
            (Self::Int(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::Int(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::Int(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::BigInt(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::BigInt(try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(lhs), Self::Int(rhs)) => {
                Ok(Self::BigInt(try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::BigInt(lhs), Self::BigInt(rhs)) => {
                Ok(Self::BigInt(try_multiply_slices(lhs, rhs)?))
            }
            (Self::BigInt(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_multiply_slices_with_casting(lhs, rhs)?))
            }
            (Self::BigInt(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Int128(lhs), Self::SmallInt(rhs)) => {
                Ok(Self::Int128(try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(lhs), Self::Int(rhs)) => {
                Ok(Self::Int128(try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(lhs), Self::BigInt(rhs)) => {
                Ok(Self::Int128(try_multiply_slices_with_casting(rhs, lhs)?))
            }
            (Self::Int128(lhs), Self::Int128(rhs)) => {
                Ok(Self::Int128(try_multiply_slices(lhs, rhs)?))
            }
            (Self::Int128(lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::SmallInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::BigInt(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Int128(rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            (Self::Decimal75(_, _, lhs_values), Self::Decimal75(_, _, rhs_values)) => {
                let (new_precision, new_scale, new_values) = try_multiply_decimal_columns(
                    lhs_values,
                    rhs_values,
                    self.column_type(),
                    rhs.column_type(),
                )?;
                Ok(Self::Decimal75(new_precision, new_scale, new_values))
            }
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: BinaryOperator::Multiply,
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::Curve25519Scalar};

    #[test]
    fn we_cannot_do_arithmetic_on_columns_with_different_lengths() {
        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(vec![1, 2]);
        let result = lhs.clone() + rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength(_, _))
        ));

        let result = lhs.clone() - rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength(_, _))
        ));

        let result = lhs * rhs;
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength(_, _))
        ));
    }

    #[test]
    fn we_cannot_do_arithmetic_on_nonnumeric_columns() {
        let lhs = OwnedColumn::<Curve25519Scalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        let rhs = OwnedColumn::<Curve25519Scalar>::Scalar(vec![
            Curve25519Scalar::from(1),
            Curve25519Scalar::from(2),
            Curve25519Scalar::from(3),
        ]);
        let result = lhs.clone() + rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.clone() - rhs.clone();
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs * rhs;
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_add_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::SmallInt(vec![1_i16, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::SmallInt(vec![1_i16, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::SmallInt(vec![2_i16, 4, 6]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::Int128(vec![1_i128, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs + rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(vec![2_i128, 4, 6]))
        );
    }

    #[test]
    fn we_can_try_add_decimal_columns() {
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [2, 4, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(6).unwrap(),
                2,
                expected_scalars
            )
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(51).unwrap(), 3, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [11, 22, 33].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(52).unwrap(),
                3,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs + rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(13).unwrap(),
                2,
                expected_scalars
            )
        );
    }

    #[test]
    fn we_can_try_subtract_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![4_i32, 5, 2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int(vec![3_i32, 3, -1]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(vec![1_i64, 2, 5]);
        let result = lhs - rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::BigInt(vec![2_i64, 0, -2]))
        );
    }

    #[test]
    fn we_can_try_subtract_decimal_columns() {
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [4, 5, 2].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [3, 3, -1].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(6).unwrap(),
                2,
                expected_scalars
            )
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [4, 5, 2].iter().map(Curve25519Scalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let lhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(25).unwrap(), 2, lhs_scalars);
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(51).unwrap(), 3, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [39, 48, 17].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(52).unwrap(),
                3,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs - rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(13).unwrap(),
                2,
                expected_scalars
            )
        );
    }

    #[test]
    fn we_can_try_multiply_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<Curve25519Scalar>::BigInt(vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<Curve25519Scalar>::BigInt(vec![1_i64, 2, 3]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::BigInt(vec![4_i64, 10, -6]))
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<Curve25519Scalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs * rhs;
        assert_eq!(
            result,
            Ok(OwnedColumn::<Curve25519Scalar>::Int128(vec![3_i128, 4, 15]))
        );
    }

    #[test]
    fn we_can_try_multiply_decimal_columns() {
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 2].iter().map(Curve25519Scalar::from).collect();
        let lhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs_scalars = [-1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [-4, 10, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(11).unwrap(),
                4,
                expected_scalars
            )
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<Curve25519Scalar>::Int(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(Curve25519Scalar::from).collect();
        let rhs =
            OwnedColumn::<Curve25519Scalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = (lhs * rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(Curve25519Scalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<Curve25519Scalar>::Decimal75(
                Precision::new(16).unwrap(),
                2,
                expected_scalars
            )
        );
    }
}
