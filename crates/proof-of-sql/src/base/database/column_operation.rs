use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::ColumnType,
    math::decimal::{scale_scalar, DecimalError, Precision},
    scalar::Scalar,
};
use core::fmt::Debug;
use num_traits::ops::checked::{CheckedAdd, CheckedMul, CheckedSub};
use proof_of_sql_parser::intermediate_ast::BinaryOperator;

// For decimal type manipulation please refer to
// https://learn.microsoft.com/en-us/sql/t-sql/data-types/precision-scale-and-length-transact-sql?view=sql-server-ver16

/// Determine the output type of an add or subtract operation if it is possible
/// to add or subtract the two input types. If the types are not compatible, return
/// an error.
pub fn try_add_subtract_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
    operator: BinaryOperator,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator,
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs.is_integer() && rhs.is_integer() {
        // We can unwrap here because we know that both types are integers
        return Ok(lhs.max_integer_type(&rhs).unwrap());
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value =
            lhs.precision_value().expect("Numeric types have precision") as i16;
        let right_precision_value =
            rhs.precision_value().expect("Numeric types have precision") as i16;
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        let scale = left_scale.max(right_scale);
        let precision_value: i16 = scale as i16
            + (left_precision_value - left_scale as i16)
                .max(right_precision_value - right_scale as i16)
            + 1_i16;
        let precision = u8::try_from(precision_value)
            .map_err(|_| {
                ColumnOperationError::DecimalConversionError(DecimalError::InvalidPrecision(
                    precision_value.to_string(),
                ))
            })
            .and_then(|p| {
                Precision::new(p).map_err(|_| {
                    ColumnOperationError::DecimalConversionError(DecimalError::InvalidPrecision(
                        p.to_string(),
                    ))
                })
            })?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a multiplication operation if it is possible
/// to multiply the two input types. If the types are not compatible, return
/// an error.
pub fn try_multiply_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: BinaryOperator::Multiply,
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs.is_integer() && rhs.is_integer() {
        // We can unwrap here because we know that both types are integers
        return Ok(lhs.max_integer_type(&rhs).unwrap());
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value = lhs.precision_value().expect("Numeric types have precision");
        let right_precision_value = rhs.precision_value().expect("Numeric types have precision");
        let precision_value = left_precision_value + right_precision_value + 1;
        let precision = Precision::new(precision_value).map_err(|_| {
            ColumnOperationError::DecimalConversionError(DecimalError::InvalidPrecision(format!(
                "Required precision {} is beyond what we can support",
                precision_value
            )))
        })?;
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        let scale = left_scale.checked_add(right_scale).ok_or(
            ColumnOperationError::DecimalConversionError(DecimalError::InvalidScale(
                left_scale as i16 + right_scale as i16,
            )),
        )?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Try to add two slices of the same length.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_add_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedAdd<Output = T> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_add(r)
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer addition {:?} + {:?}",
                    l, r
                )))
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

/// Subtract one slice from another of the same length.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_subtract_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedSub<Output = T> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_sub(r)
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer subtraction {:?} - {:?}",
                    l, r
                )))
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

/// Multiply two slices of the same length.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_multiply_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedMul<Output = T> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_mul(r)
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer multiplication {:?} * {:?}",
                    l, r
                )))
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

/// Add two slices of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_add_slices_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug,
    LargerType: From<SmallerType> + CheckedAdd<Output = LargerType> + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            LargerType::from(*l)
                .checked_add(r)
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer addition {:?} + {:?}",
                    l, r
                )))
        })
        .collect()
}

/// Subtract one slice from another of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here
pub(super) fn try_subtract_slices_left_upcast<SmallerType, LargerType>(
    lhs: &[SmallerType],
    rhs: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug,
    LargerType: From<SmallerType> + CheckedSub<Output = LargerType> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            LargerType::from(*l)
                .checked_sub(r)
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer subtraction {:?} - {:?}",
                    l, r
                )))
        })
        .collect()
}

/// Subtract one slice from another of the same length, casting the right slice to the type of the left slice.
///
/// We do not check for length equality here
pub(super) fn try_subtract_slices_right_upcast<SmallerType, LargerType>(
    lhs: &[LargerType],
    rhs: &[SmallerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug,
    LargerType: From<SmallerType> + CheckedSub<Output = LargerType> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            l.checked_sub(&LargerType::from(*r))
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer subtraction {:?} - {:?}",
                    l, r
                )))
        })
        .collect()
}

/// Multiply two slices of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_multiply_slices_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug,
    LargerType: From<SmallerType> + CheckedMul<Output = LargerType> + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            LargerType::from(*l)
                .checked_mul(r)
                .ok_or(ColumnOperationError::IntegerOverflow(format!(
                    "Overflow in integer multiplication {:?} * {:?}",
                    l, r
                )))
        })
        .collect()
}

/// Add two numerical slices as decimals.
///
/// We do not check for length equality here
pub(super) fn try_add_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar + core::convert::From<T0> + core::convert::From<T1>,
    T0: Copy,
    T1: Copy,
{
    let new_column_type =
        try_add_subtract_column_types(left_column_type, right_column_type, BinaryOperator::Add)?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let left_upscale = new_scale
        - left_column_type
            .scale()
            .expect("numeric columns have scale");
    let right_upscale = new_scale
        - right_column_type
            .scale()
            .expect("numeric columns have scale");
    // One of left_scale and right_scale is 0 so we can avoid scaling when unnecessary
    let scalars: Vec<S> = if left_upscale > 0 {
        let upscale_factor = scale_scalar(S::ONE, left_upscale)?;
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) * upscale_factor + S::from(*r))
            .collect()
    } else if right_upscale > 0 {
        let upscale_factor = scale_scalar(S::ONE, right_upscale)?;
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) + upscale_factor * S::from(*r))
            .collect()
    } else {
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) + S::from(*r))
            .collect()
    };
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        scalars,
    ))
}

/// Subtract one numerical slice from another as decimals.
///
/// We do not check for length equality here
pub(super) fn try_subtract_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar + core::convert::From<T0> + core::convert::From<T1>,
    T0: Copy,
    T1: Copy,
{
    let new_column_type = try_add_subtract_column_types(
        left_column_type,
        right_column_type,
        BinaryOperator::Subtract,
    )?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let left_upscale = new_scale
        - left_column_type
            .scale()
            .expect("numeric columns have scale");
    let right_upscale = new_scale
        - right_column_type
            .scale()
            .expect("numeric columns have scale");
    // One of left_scale and right_scale is 0 so we can avoid scaling when unnecessary
    let scalars: Vec<S> = if left_upscale > 0 {
        let upscale_factor = scale_scalar(S::ONE, left_upscale)?;
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) * upscale_factor - S::from(*r))
            .collect()
    } else if right_upscale > 0 {
        let upscale_factor = scale_scalar(S::ONE, right_upscale)?;
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) - upscale_factor * S::from(*r))
            .collect()
    } else {
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) - S::from(*r))
            .collect()
    };
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        scalars,
    ))
}

/// Multiply two numerical slices as decimals.
///
/// We do not check for length equality here
pub(super) fn try_multiply_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar + core::convert::From<T0> + core::convert::From<T1>,
    T0: Copy,
    T1: Copy,
{
    let new_column_type = try_multiply_column_types(left_column_type, right_column_type)?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let scalars: Vec<S> = lhs
        .iter()
        .zip(rhs)
        .map(|(l, r)| S::from(*l) * S::from(*r))
        .collect();
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        scalars,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::scalar::Curve25519Scalar;

    #[test]
    fn we_can_add_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal with nonnegative scale and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(21).unwrap(), 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(59).unwrap(), 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(74).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), -14);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_add_non_numeric_types() {
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_add_some_numeric_types_due_to_decimal_issues() {
        let lhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 4);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), 4);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidPrecision(_)
            ))
        ));

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 10);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Add),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidPrecision(_)
            ))
        ));
    }

    #[test]
    fn we_can_subtract_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(21).unwrap(), 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(59).unwrap(), 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(61).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), -14);
        let actual = try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_subtract_non_numeric_types() {
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_subtract_some_numeric_types_due_to_decimal_issues() {
        let lhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 0);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), 1);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidPrecision(_)
            ))
        ));

        let lhs = ColumnType::Int128;
        let rhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 12);
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs, BinaryOperator::Subtract),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidPrecision(_)
            ))
        ));
    }

    #[test]
    fn we_can_multiply_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::SmallInt;
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Int;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(16).unwrap(), 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(31).unwrap(), 5);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(16).unwrap(), -2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(56).unwrap(), -8);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(61).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(13).unwrap(), -14);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), -27);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_multiply_non_numeric_types() {
        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_multiply_some_numeric_types_due_to_decimal_issues() {
        // Invalid precision
        let lhs = ColumnType::Decimal75(Precision::new(38).unwrap(), 4);
        let rhs = ColumnType::Decimal75(Precision::new(37).unwrap(), 4);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidPrecision(_)
            ))
        ));

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(Precision::new(65).unwrap(), 0);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidPrecision(_)
            ))
        ));

        // Invalid scale
        let lhs = ColumnType::Decimal75(Precision::new(5).unwrap(), -64_i8);
        let rhs = ColumnType::Decimal75(Precision::new(5).unwrap(), -65_i8);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidScale(_)
            ))
        ));

        let lhs = ColumnType::Decimal75(Precision::new(5).unwrap(), 64_i8);
        let rhs = ColumnType::Decimal75(Precision::new(5).unwrap(), 64_i8);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError(
                DecimalError::InvalidScale(_)
            ))
        ));
    }

    #[test]
    fn we_can_try_add_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_add_slices(&lhs, &rhs).unwrap();
        let expected = vec![5_i16, -3, 9];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_add_slices_if_overflow() {
        let lhs = [i16::MAX, 1];
        let rhs = [1_i16, 1];
        assert!(matches!(
            try_add_slices(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_add_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_add_slices_with_casting(&lhs, &rhs).unwrap();
        let expected = vec![5_i32, -3, 9];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_add_slices_with_cast_if_overflow() {
        let lhs = [-1_i16, 1];
        let rhs = [i32::MIN, 1];
        assert!(matches!(
            try_add_slices_with_casting(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_add_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i16, -2, 3];
        let rhs = [4_i16, 5, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(104),
            Curve25519Scalar::from(-195),
            Curve25519Scalar::from(298),
        ];
        let expected = (Precision::new(11).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::BigInt;
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(471),
            Curve25519Scalar::from(1418),
            Curve25519Scalar::from(-177),
        ];
        let expected = (Precision::new(20).unwrap(), 0, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(111),
            Curve25519Scalar::from(68),
            Curve25519Scalar::from(3),
        ];
        let expected = (Precision::new(14).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        // and with result having maximum precision
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(69).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(50).unwrap(), 3);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(400071),
            Curve25519Scalar::from(1499918),
            Curve25519Scalar::from(-199977),
        ];
        let expected = (Precision::new(75).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        // and with result having maximum precision and minimum scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let right_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(75),
            Curve25519Scalar::from(-67),
            Curve25519Scalar::from(21),
        ];
        let expected = (Precision::new(75).unwrap(), -128, expected_scalars);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_try_subtract_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_subtract_slices(&lhs, &rhs).unwrap();
        let expected = vec![-3_i16, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_if_overflow() {
        let lhs = [i128::MIN, 1];
        let rhs = [1_i128, 1];
        assert!(matches!(
            try_subtract_slices(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_subtract_slices_left_upcast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_subtract_slices_left_upcast(&lhs, &rhs).unwrap();
        let expected = vec![-3_i32, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_left_upcast_if_overflow() {
        let lhs = [0_i16, 1];
        let rhs = [i32::MIN, 1];
        assert!(matches!(
            try_subtract_slices_left_upcast(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_subtract_slices_right_upcast() {
        let lhs = [1_i32, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_subtract_slices_right_upcast(&lhs, &rhs).unwrap();
        let expected = vec![-3_i32, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_right_upcast_if_overflow() {
        let lhs = [i32::MIN, 1];
        let rhs = [1_i16, 1];
        assert!(matches!(
            try_subtract_slices_right_upcast(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_subtract_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i16, -2, 3];
        let rhs = [4_i16, 5, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(96),
            Curve25519Scalar::from(-205),
            Curve25519Scalar::from(302),
        ];
        let expected = (Precision::new(11).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::BigInt;
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(329),
            Curve25519Scalar::from(1582),
            Curve25519Scalar::from(-223),
        ];
        let expected = (Precision::new(20).unwrap(), 0, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(-31),
            Curve25519Scalar::from(232),
            Curve25519Scalar::from(-43),
        ];
        let expected = (Precision::new(14).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        // and with result having maximum precision
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(69).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(50).unwrap(), 3);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(399929),
            Curve25519Scalar::from(1500082),
            Curve25519Scalar::from(-200023),
        ];
        let expected = (Precision::new(75).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        // and with result having maximum precision and minimum scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let right_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(-67),
            Curve25519Scalar::from(97),
            Curve25519Scalar::from(-25),
        ];
        let expected = (Precision::new(75).unwrap(), -128, expected_scalars);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_try_multiply_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_multiply_slices(&lhs, &rhs).unwrap();
        let expected = vec![4_i16, -10, 18];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_multiply_slices_if_overflow() {
        let lhs = [i32::MAX, 2];
        let rhs = [2, 2];
        assert!(matches!(
            try_multiply_slices(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_multiply_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_multiply_slices_with_casting(&lhs, &rhs).unwrap();
        let expected = vec![4_i32, -10, 18];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_multiply_slices_with_cast_if_overflow() {
        let lhs = [2_i16, 2];
        let rhs = [i32::MAX, 2];
        assert!(matches!(
            try_multiply_slices_with_casting(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow(_))
        ));
    }

    #[test]
    fn we_can_try_multiply_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i16, -2, 3];
        let rhs = [4_i16, 5, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(4),
            Curve25519Scalar::from(-10),
            Curve25519Scalar::from(-6),
        ];
        let expected = (Precision::new(16).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::BigInt;
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(284),
            Curve25519Scalar::from(-1230),
            Curve25519Scalar::from(-46),
        ];
        let expected = (Precision::new(30).unwrap(), -2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        // and with result having maximum precision and maximum scale
        let lhs = [4_i16, 25, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(42).unwrap(), 72);
        let right_column_type = ColumnType::Decimal75(Precision::new(32).unwrap(), 55);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(284),
            Curve25519Scalar::from(-2050),
            Curve25519Scalar::from(-46),
        ];
        let expected = (Precision::new(75).unwrap(), 127, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        // and with result having maximum precision
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(69).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(5).unwrap(), 3);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(284),
            Curve25519Scalar::from(-1230),
            Curve25519Scalar::from(-46),
        ];
        let expected = (Precision::new(75).unwrap(), 1, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        // and with result having maximum precision and minimum scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(Curve25519Scalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(34).unwrap(), -64);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), -64);
        let actual: (Precision, i8, Vec<Curve25519Scalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            Curve25519Scalar::from(284),
            Curve25519Scalar::from(-1230),
            Curve25519Scalar::from(-46),
        ];
        let expected = (Precision::new(75).unwrap(), -128, expected_scalars);
        assert_eq!(expected, actual);
    }
}
