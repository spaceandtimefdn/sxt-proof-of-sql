use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::ColumnType,
    math::decimal::{DecimalError, Precision},
};
use alloc::string::ToString;
// For decimal type manipulation please refer to
// https://learn.microsoft.com/en-us/sql/t-sql/data-types/precision-scale-and-length-transact-sql?view=sql-server-ver16

/// Determine the output type of an add or subtract operation if it is possible
/// to add or subtract the two input types. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_add_subtract_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "+/-".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value = lhs.precision_value().expect("Numeric types have precision");
        let right_precision_value = rhs.precision_value().expect("Numeric types have precision");
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        if left_scale != right_scale {
            return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "+/-".to_string(),
                left_type: lhs,
                right_type: rhs,
            });
        }
        let precision_value = (left_precision_value.max(right_precision_value) + 1_u8).min(75_u8);
        let precision =
            Precision::new(precision_value).expect("Precision value should be in range 0-75");
        Ok(ColumnType::Decimal75(precision, left_scale))
    }
}

/// Determine the output type of an add or subtract operation if it is possible
/// to add or subtract the two input types. Scaling is allowed. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_add_subtract_column_types_with_scaling(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "+/-".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value =
            i16::from(lhs.precision_value().expect("Numeric types have precision"));
        let right_precision_value =
            i16::from(rhs.precision_value().expect("Numeric types have precision"));
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        let scale = left_scale.max(right_scale);
        let precision_value: i16 = (i16::from(scale)
            + (left_precision_value - i16::from(left_scale))
                .max(right_precision_value - i16::from(right_scale))
            + 1_i16)
            .min(75_i16);
        let precision = Precision::new(
            u8::try_from(precision_value).expect("Precision value should be in range 0-75"),
        )
        .expect("Precision value should be in range 0-75");
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a multiplication operation if it is possible
/// to multiply the two input types. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_multiply_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "*".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value = lhs.precision_value().expect("Numeric types have precision");
        let right_precision_value = rhs.precision_value().expect("Numeric types have precision");
        let precision_value = (left_precision_value + right_precision_value + 1).min(75_u8);
        let precision =
            Precision::new(precision_value).expect("Precision value should be in range 0-75");
        let left_scale = lhs.scale().expect("Numeric types have scale");
        let right_scale = rhs.scale().expect("Numeric types have scale");
        let scale = left_scale.checked_add(right_scale).ok_or(
            ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale {
                    scale: (i16::from(left_scale) + i16::from(right_scale)).to_string(),
                },
            },
        )?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a divide/modulo operation if it is possible
/// to divide and modulo the two input types. If the types are not compatible, return
/// an error.
pub fn try_divide_modulo_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<(ColumnType, ColumnType)> {
    if lhs.is_integer() && lhs.is_signed() && rhs.is_integer() && rhs.is_signed() {
        Ok((lhs, lhs))
    } else {
        Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "/%".to_string(),
            left_type: lhs,
            right_type: rhs,
        })
    }
}

/// Determine the output type of a division operation if it is possible
/// to divide the two input types. If the types are not compatible, return
/// an error.
///
/// # Panics
///
/// - Panics if `lhs` or `rhs` does not have a precision or scale when they are expected to be numeric types.
/// - Panics if `lhs` or `rhs` is an integer, and `lhs.max_integer_type(&rhs)` returns `None`.
pub fn try_divide_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<ColumnType> {
    if !lhs.is_numeric()
        || !rhs.is_numeric()
        || lhs == ColumnType::Scalar
        || rhs == ColumnType::Scalar
    {
        return Err(ColumnOperationError::BinaryOperationInvalidColumnType {
            operator: "/".to_string(),
            left_type: lhs,
            right_type: rhs,
        });
    }
    let left_precision_value =
        i16::from(lhs.precision_value().expect("Numeric types have precision"));
    let right_precision_value =
        i16::from(rhs.precision_value().expect("Numeric types have precision"));
    let left_scale = i16::from(lhs.scale().expect("Numeric types have scale"));
    let right_scale = i16::from(rhs.scale().expect("Numeric types have scale"));
    let raw_scale = (left_scale + right_precision_value + 1_i16).max(6_i16);
    let precision_value: i16 =
        (left_precision_value - left_scale + right_scale + raw_scale).min(75_i16);
    let scale =
        i8::try_from(raw_scale).map_err(|_| ColumnOperationError::DecimalConversionError {
            source: DecimalError::InvalidScale {
                scale: raw_scale.to_string(),
            },
        })?;
    let precision = Precision::new(
        u8::try_from(precision_value).expect("Precision value should be in range 0-75"),
    )
    .expect("Precision value should be in range 0-75");
    Ok(ColumnType::Decimal75(precision, scale))
}

/// Verifies that `from` can be cast to `to`. For now, this supports a limited number of casts.
#[expect(clippy::missing_panics_doc)]
pub fn try_cast_types(from: ColumnType, to: ColumnType) -> ColumnOperationResult<()> {
    match (from, to) {
        (
            ColumnType::Boolean,
            ColumnType::TinyInt
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::Int128
            | ColumnType::BigInt,
        )
        | (ColumnType::TimestampTZ(_, _), ColumnType::BigInt)
        | (ColumnType::Uint8, ColumnType::Uint8)
        | (ColumnType::TinyInt, ColumnType::TinyInt) => true,
        (
            ColumnType::TinyInt
            | ColumnType::Uint8
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::Int128
            | ColumnType::BigInt,
            ColumnType::Decimal75(_, 0)
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::BigInt
            | ColumnType::Int128,
        )
        | (ColumnType::Decimal75(_, _), ColumnType::Decimal75(_, _)) => {
            to.precision_value().unwrap() >= from.precision_value().unwrap()
                && to.scale() == from.scale()
        }
        _ => false,
    }
    .then_some(())
    .ok_or(ColumnOperationError::CastingError {
        left_type: from,
        right_type: to,
    })
}

/// Verifies that `from` can be cast to `to`.
/// Casting can only be supported if the resulting data type is a superset of the input data type.
/// For example Deciaml(6,1) can be cast to Decimal(7,1), but not vice versa.
#[expect(clippy::missing_panics_doc)]
pub fn try_scale_cast_types(from: ColumnType, to: ColumnType) -> ColumnOperationResult<()> {
    match (from, to) {
        (
            ColumnType::TinyInt
            | ColumnType::Uint8
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::Int128
            | ColumnType::BigInt
            | ColumnType::Decimal75(_, _),
            ColumnType::Decimal75(precision, scale),
        ) => {
            let from_precision = i16::from(from.precision_value().unwrap());
            let from_scale = i16::from(from.scale().unwrap());
            let to_precision = i16::from(precision.value());
            let to_scale = i16::from(scale);
            to_scale >= from_scale && (to_precision - to_scale) >= (from_precision - from_scale)
        }
        (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _)) => {
            to.scale().unwrap() >= from.scale().unwrap()
        }
        _ => false,
    }
    .then_some(())
    .ok_or(ColumnOperationError::ScaleCastingError {
        left_type: from,
        right_type: to,
    })
}

/// Verfies that the equality operator can be used on the two types
pub fn try_equals_types(lhs: ColumnType, rhs: ColumnType) -> ColumnOperationResult<()> {
    (matches!(
        (lhs, rhs),
        (ColumnType::VarChar, ColumnType::VarChar)
            | (ColumnType::VarBinary, ColumnType::VarBinary)
            | (ColumnType::Boolean, ColumnType::Boolean)
            | (_, ColumnType::Scalar)
            | (ColumnType::Scalar, _)
    ) || (lhs.is_numeric() && rhs.is_numeric() && lhs.scale() == rhs.scale())
        || matches!(
            (lhs, rhs),
            (ColumnType::TimestampTZ(left_tu, _), ColumnType::TimestampTZ(right_tu, _)) if
                left_tu == right_tu
        ))
    .then_some(())
    .ok_or(ColumnOperationError::BinaryOperationInvalidColumnType {
        operator: "=".to_string(),
        left_type: lhs,
        right_type: rhs,
    })
}

/// Verifies that two types can be compared using inequalities
pub fn try_inequality_types(lhs: ColumnType, rhs: ColumnType) -> ColumnOperationResult<()> {
    (lhs != ColumnType::VarChar
        && rhs != ColumnType::VarChar
        // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
        && !matches!(lhs, ColumnType::Decimal75(precision, _) if precision.value() > 38)
        && !matches!(rhs, ColumnType::Decimal75(precision, _) if precision.value() > 38)
        && (lhs.is_numeric() && rhs.is_numeric() && lhs.scale() == rhs.scale()
            || matches!(
                (lhs, rhs),
                (ColumnType::Boolean, ColumnType::Boolean)
            )
            || matches!(
                (lhs, rhs),
                (ColumnType::TimestampTZ(left_tu, _), ColumnType::TimestampTZ(right_tu, _)) if
                left_tu == right_tu
        )))
    .then_some(())
    .ok_or(ColumnOperationError::BinaryOperationInvalidColumnType {
        operator: "</>".to_string(),
        left_type: lhs,
        right_type: rhs,
    })
}

/// Verfies that the equality operator can be used on the two types with scaling allowed
pub fn try_equals_types_with_scaling(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<()> {
    (matches!(
        (lhs, rhs),
        (ColumnType::VarChar, ColumnType::VarChar)
            | (ColumnType::VarBinary, ColumnType::VarBinary)
            | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
            | (ColumnType::Boolean, ColumnType::Boolean)
            | (_, ColumnType::Scalar)
            | (ColumnType::Scalar, _)
    ) || (lhs.is_numeric() && rhs.is_numeric()))
    .then_some(())
    .ok_or(ColumnOperationError::BinaryOperationInvalidColumnType {
        operator: "=".to_string(),
        left_type: lhs,
        right_type: rhs,
    })
}

/// Verifies that two types can be compared using inequalities with scaling allowed
pub fn try_inequality_types_with_scaling(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ColumnOperationResult<()> {
    (lhs != ColumnType::VarChar
        && rhs != ColumnType::VarChar
        // Due to constraints in bitwise_verification we limit the precision of decimal types to 38
        && !matches!(lhs, ColumnType::Decimal75(precision, _) if precision.value() > 38)
        && !matches!(rhs, ColumnType::Decimal75(precision, _) if precision.value() > 38)
        && (lhs.is_numeric() && rhs.is_numeric()
            || matches!(
                (lhs, rhs),
                (ColumnType::Boolean, ColumnType::Boolean)
                    | (ColumnType::TimestampTZ(_, _), ColumnType::TimestampTZ(_, _))
            )))
    .then_some(())
    .ok_or(ColumnOperationError::BinaryOperationInvalidColumnType {
        operator: "</>".to_string(),
        left_type: lhs,
        right_type: rhs,
    })
}

/// Verifies that two types can be used in a logical AND or OR expressions
#[must_use]
pub fn can_and_or_types(lhs: ColumnType, rhs: ColumnType) -> bool {
    lhs == ColumnType::Boolean && rhs == ColumnType::Boolean
}

/// Verifies that the NOT expr can be used on an expression
#[must_use]
pub fn can_not_type(datatype: ColumnType) -> bool {
    datatype == ColumnType::Boolean
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use itertools::iproduct;

    #[test]
    fn we_can_add_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(4).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(6).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(6).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with scale 0
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 0);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are decimals with the same scale
        for scale in [-3, 0, 3] {
            let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), scale);
            let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), scale);
            let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
            let expected = ColumnType::Decimal75(Precision::new(21).unwrap(), scale);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn we_can_add_some_numeric_types_with_precision_capping() {
        let lhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 4);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), 4);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 4);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(Precision::new(65).unwrap(), 10);
        let rhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 10);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 10);
        let actual = try_add_subtract_column_types(lhs, rhs).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_add_non_numeric_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_add_numeric_types_with_different_scales() {
        // lhs is a decimal with nonnegative scale and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(21).unwrap(), 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(59).unwrap(), 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(74).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), -14);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_add_numeric_types_with_scaling() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(4).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(6).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(6).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal with nonnegative scale and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(21).unwrap(), 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(59).unwrap(), 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(74).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), -14);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_add_non_numeric_types_with_scaling() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types_with_scaling(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types_with_scaling(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types_with_scaling(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_add_some_numeric_types_with_precision_capping_and_scaling() {
        let lhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 4);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), 4);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 4);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        assert_eq!(expected, actual);

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 10);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 10);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_subtract_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(4).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(6).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(6).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 2);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(21).unwrap(), 3);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(13).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(59).unwrap(), 5);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(61).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), -14);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), -13);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_subtract_non_numeric_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types_with_scaling(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types_with_scaling(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_add_subtract_column_types_with_scaling(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_subtract_some_numeric_types_with_precision_capping() {
        let lhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 0);
        let rhs = ColumnType::Decimal75(Precision::new(73).unwrap(), 1);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 1);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        assert_eq!(expected, actual);

        let lhs = ColumnType::Int128;
        let rhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 12);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 12);
        let actual = try_add_subtract_column_types_with_scaling(lhs, rhs).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_multiply_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(7).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(9).unwrap(), 0);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(16).unwrap(), 0);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a scalar
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Scalar;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Scalar;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Scalar;
        assert_eq!(expected, actual);

        // lhs is a decimal and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(14).unwrap(), 2);
        assert_eq!(expected, actual);

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
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(14).unwrap(), -2);
        assert_eq!(expected, actual);

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
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

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
    fn we_can_multiply_some_numeric_types_with_precision_capping() {
        let lhs = ColumnType::Decimal75(Precision::new(38).unwrap(), 4);
        let rhs = ColumnType::Decimal75(Precision::new(37).unwrap(), 4);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 8);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        assert_eq!(expected, actual);

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(Precision::new(65).unwrap(), 0);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 0);
        let actual = try_multiply_column_types(lhs, rhs).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_multiply_some_numeric_types_with_invalid_scale() {
        // Invalid scale
        let lhs = ColumnType::Decimal75(Precision::new(5).unwrap(), -64_i8);
        let rhs = ColumnType::Decimal75(Precision::new(5).unwrap(), -65_i8);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale { .. }
            })
        ));

        let lhs = ColumnType::Decimal75(Precision::new(5).unwrap(), 64_i8);
        let rhs = ColumnType::Decimal75(Precision::new(5).unwrap(), 64_i8);
        assert!(matches!(
            try_multiply_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale { .. }
            })
        ));
    }

    #[test]
    fn we_can_divide_numeric_types() {
        // lhs and rhs are integers with the same precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::TinyInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(9).unwrap(), 6);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(11).unwrap(), 6);
        assert_eq!(expected, actual);

        // lhs and rhs are integers with different precision
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::SmallInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(9).unwrap(), 6);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Int;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(16).unwrap(), 11);
        assert_eq!(expected, actual);

        // lhs is a decimal with nonnegative scale and rhs is an integer
        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::TinyInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(14).unwrap(), 6);
        assert_eq!(expected, actual);

        let lhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let rhs = ColumnType::SmallInt;
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(16).unwrap(), 8);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with nonnegative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(16).unwrap(), 11);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(18).unwrap(), 11);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = ColumnType::Decimal75(Precision::new(20).unwrap(), 3);
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(33).unwrap(), 14);
        assert_eq!(expected, actual);

        // lhs is an integer and rhs is a decimal with negative scale
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(12).unwrap(), 11);
        assert_eq!(expected, actual);

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(14).unwrap(), 11);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = ColumnType::Decimal75(Precision::new(40).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 5);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(64).unwrap(), 6);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals both with negative scale
        // and with result having maximum precision
        let lhs = ColumnType::Decimal75(Precision::new(70).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(13).unwrap(), -14);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 6);
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_divide_non_numeric_or_scalar_types() {
        let lhs = ColumnType::TinyInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::SmallInt;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::VarChar;
        let rhs = ColumnType::VarChar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = ColumnType::Scalar;
        let rhs = ColumnType::Scalar;
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_divide_some_numeric_types_with_precision_capping() {
        let lhs = ColumnType::Decimal75(Precision::new(71).unwrap(), -13);
        let rhs = ColumnType::Decimal75(Precision::new(13).unwrap(), -14);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 6);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        assert_eq!(expected, actual);

        let lhs = ColumnType::Int;
        let rhs = ColumnType::Decimal75(Precision::new(68).unwrap(), 67);
        let expected = ColumnType::Decimal75(Precision::new(75).unwrap(), 69);
        let actual = try_divide_column_types(lhs, rhs).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_divide_some_numeric_types_due_to_decimal_issues() {
        // Invalid scale
        let lhs = ColumnType::Decimal75(Precision::new(15).unwrap(), 53_i8);
        let rhs = ColumnType::Decimal75(Precision::new(75).unwrap(), 40_i8);
        assert!(matches!(
            try_divide_column_types(lhs, rhs),
            Err(ColumnOperationError::DecimalConversionError {
                source: DecimalError::InvalidScale { .. }
            })
        ));
    }

    #[test]
    fn we_can_get_correct_column_types_for_divide_and_modulo() {
        let eligible_columns = [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ];
        for (numerator, denominator) in iproduct!(eligible_columns, eligible_columns) {
            let remainder = try_divide_modulo_column_types(numerator, denominator).unwrap();
            assert_eq!(remainder, (numerator, numerator));
        }
        let ineligible_columns = [
            ColumnType::Uint8,
            ColumnType::Scalar,
            ColumnType::Boolean,
            ColumnType::VarBinary,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::new(1)),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 1),
            ColumnType::VarChar,
        ];
        for (left_type, right_type) in iproduct!(eligible_columns, ineligible_columns)
            .chain(iproduct!(ineligible_columns, eligible_columns))
        {
            let err = try_divide_modulo_column_types(left_type, right_type).unwrap_err();
            assert!(matches!(
                err,
                ColumnOperationError::BinaryOperationInvalidColumnType {
                    operator: _,
                    left_type: _,
                    right_type: _
                }
            ));
        }
    }

    #[test]
    fn we_can_cast_bool_to_signed_ints() {
        for to in [
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ] {
            try_cast_types(ColumnType::Boolean, to).unwrap();
        }
    }

    #[test]
    fn we_cannot_cast_integers_to_decimal_with_lower_precision() {
        for from in [
            ColumnType::TinyInt,
            ColumnType::Uint8,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
            ColumnType::Decimal75(Precision::new(60).unwrap(), 0),
        ] {
            assert!(matches!(
                try_cast_types(from, ColumnType::Decimal75(Precision::new(2).unwrap(), 0)),
                Err(ColumnOperationError::CastingError { .. })
            ));
        }
    }

    #[test]
    fn we_can_cast_integers_and_decimal_to_decimal() {
        for from in [
            ColumnType::TinyInt,
            ColumnType::Uint8,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
            ColumnType::Decimal75(Precision::new(60).unwrap(), 0),
        ] {
            try_cast_types(from, ColumnType::Decimal75(Precision::new(60).unwrap(), 0)).unwrap();
            try_cast_types(
                from,
                ColumnType::Decimal75(Precision::new(from.precision_value().unwrap()).unwrap(), 0),
            )
            .unwrap();
            try_cast_types(from, ColumnType::Decimal75(Precision::new(2).unwrap(), 0)).unwrap_err();
        }
    }

    #[test]
    fn we_can_cast_decimal_to_decimal_with_same_scale() {
        try_cast_types(
            ColumnType::Decimal75(Precision::new(2).unwrap(), 1),
            ColumnType::Decimal75(Precision::new(3).unwrap(), 1),
        )
        .unwrap();
        try_cast_types(
            ColumnType::Decimal75(Precision::new(2).unwrap(), 1),
            ColumnType::Decimal75(Precision::new(3).unwrap(), 2),
        )
        .unwrap_err();
    }

    #[test]
    fn we_can_cast_integers_to_signed_integers() {
        try_cast_types(ColumnType::Uint8, ColumnType::Uint8).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::Uint8, ColumnType::SmallInt).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::Uint8, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::TinyInt).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::SmallInt).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::TinyInt, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::SmallInt, ColumnType::SmallInt).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::SmallInt, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::Int, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::Int, ColumnType::SmallInt).unwrap_err();
        try_cast_types(ColumnType::Int, ColumnType::Int).unwrap();
        try_cast_types(ColumnType::Int, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::Int, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::BigInt, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::BigInt, ColumnType::SmallInt).unwrap_err();
        try_cast_types(ColumnType::BigInt, ColumnType::Int).unwrap_err();
        try_cast_types(ColumnType::BigInt, ColumnType::BigInt).unwrap();
        try_cast_types(ColumnType::BigInt, ColumnType::Int128).unwrap();
        try_cast_types(ColumnType::Int128, ColumnType::TinyInt).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::SmallInt).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::Int).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::BigInt).unwrap_err();
        try_cast_types(ColumnType::Int128, ColumnType::Int128).unwrap();
    }

    #[test]
    fn we_can_cast_timestamp_to_big_int() {
        try_cast_types(
            ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::new(1)),
            ColumnType::BigInt,
        )
        .unwrap();
    }

    #[test]
    fn we_cannot_cast_uncastable_type() {
        let err = try_cast_types(ColumnType::BigInt, ColumnType::Boolean).unwrap_err();
        assert!(matches!(
            err,
            ColumnOperationError::CastingError {
                left_type: ColumnType::BigInt,
                right_type: ColumnType::Boolean
            }
        ));
    }

    #[test]
    fn we_can_properly_determine_if_types_are_scale_castable() {
        for from in [
            ColumnType::Uint8,
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ] {
            let from_precision = Precision::new(from.precision_value().unwrap()).unwrap();
            let two_prec = Precision::new(2).unwrap();
            let forty_prec = Precision::new(40).unwrap();
            try_scale_cast_types(from, ColumnType::Decimal75(two_prec, 0)).unwrap_err();
            try_scale_cast_types(from, ColumnType::Decimal75(two_prec, -1)).unwrap_err();
            try_scale_cast_types(from, ColumnType::Decimal75(two_prec, 1)).unwrap_err();
            try_scale_cast_types(from, ColumnType::Decimal75(from_precision, 0)).unwrap();
            try_scale_cast_types(from, ColumnType::Decimal75(from_precision, -1)).unwrap_err();
            try_scale_cast_types(from, ColumnType::Decimal75(from_precision, 1)).unwrap_err();
            try_scale_cast_types(from, ColumnType::Decimal75(forty_prec, 0)).unwrap();
            try_scale_cast_types(from, ColumnType::Decimal75(forty_prec, -1)).unwrap_err();
            try_scale_cast_types(from, ColumnType::Decimal75(forty_prec, 1)).unwrap();
        }

        let twenty_prec = Precision::new(20).unwrap();

        // from_with_negative_scale
        let neg_scale = ColumnType::Decimal75(twenty_prec, -3);

        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, -4)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, -3)).unwrap();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, -2)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, 0)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_prec, 1)).unwrap_err();

        let nineteen_prec = Precision::new(19).unwrap();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, -4)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, -3)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, -2)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, 0)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(nineteen_prec, 1)).unwrap_err();

        let twenty_one_prec = Precision::new(21).unwrap();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, -4)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, -3)).unwrap();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, -2)).unwrap();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, 0)).unwrap_err();
        try_scale_cast_types(neg_scale, ColumnType::Decimal75(twenty_one_prec, 1)).unwrap_err();

        // from_with_zero_scale
        let zero_scale = ColumnType::Decimal75(twenty_prec, 0);

        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_prec, -1)).unwrap_err();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_prec, 0)).unwrap();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_prec, 1)).unwrap_err();

        try_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, -1)).unwrap_err();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, 0)).unwrap_err();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, 1)).unwrap_err();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(nineteen_prec, 2)).unwrap_err();

        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, -1)).unwrap_err();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, 0)).unwrap();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, 1)).unwrap();
        try_scale_cast_types(zero_scale, ColumnType::Decimal75(twenty_one_prec, 2)).unwrap_err();

        // from_with_positive_scale
        let pos_scale = ColumnType::Decimal75(twenty_prec, 3);

        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, -1)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 0)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 2)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 3)).unwrap();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_prec, 4)).unwrap_err();

        try_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, -1)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 0)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 2)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 3)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(nineteen_prec, 4)).unwrap_err();

        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, -1)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 0)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 2)).unwrap_err();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 3)).unwrap();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 4)).unwrap();
        try_scale_cast_types(pos_scale, ColumnType::Decimal75(twenty_one_prec, 5)).unwrap_err();
    }

    #[test]
    fn we_cannot_scale_cast_nonsense_pairings() {
        try_scale_cast_types(ColumnType::Int128, ColumnType::Boolean).unwrap_err();
    }
}
