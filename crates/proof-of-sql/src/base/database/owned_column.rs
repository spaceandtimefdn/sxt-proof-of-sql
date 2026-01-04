/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
use super::{Column, ColumnCoercionError, ColumnType, OwnedColumnError, OwnedColumnResult};
use crate::base::{
    math::{
        decimal::Precision,
        permutation::{Permutation, PermutationError},
    },
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
    slice_ops::{inner_product_ref_cast, inner_product_with_bytes},
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
/// Supported types for [`OwnedColumn`]
pub enum OwnedColumn<S: Scalar> {
    /// Boolean columns
    Boolean(Vec<bool>),
    /// u8 columns
    Uint8(Vec<u8>),
    /// i8 columns
    TinyInt(Vec<i8>),
    /// i16 columns
    SmallInt(Vec<i16>),
    /// i32 columns
    Int(Vec<i32>),
    /// i64 columns
    BigInt(Vec<i64>),
    /// i128 columns
    Int128(Vec<i128>),
    /// String columns
    VarChar(Vec<String>),
    /// Decimal columns
    #[cfg_attr(test, proptest(skip))]
    Decimal75(Precision, i8, Vec<S>),
    /// Timestamp columns
    #[cfg_attr(test, proptest(skip))]
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, Vec<i64>),
    /// Scalar columns
    #[cfg_attr(test, proptest(skip))]
    Scalar(Vec<S>),
    /// Variable length binary columns
    VarBinary(Vec<Vec<u8>>),
}

impl<S: Scalar> OwnedColumn<S> {
    /// Compute the inner product of the column with a vector of scalars.
    pub(crate) fn inner_product(&self, vec: &[S]) -> S {
        match self {
            OwnedColumn::Boolean(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Uint8(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::TinyInt(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::SmallInt(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Int(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
                inner_product_ref_cast(col, vec)
            }
            OwnedColumn::VarChar(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::VarBinary(col) => inner_product_with_bytes(col, vec),
            OwnedColumn::Int128(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => {
                inner_product_ref_cast(col, vec)
            }
        }
    }

    /// Returns the length of the column.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            OwnedColumn::Boolean(col) => col.len(),
            OwnedColumn::TinyInt(col) => col.len(),
            OwnedColumn::Uint8(col) => col.len(),
            OwnedColumn::SmallInt(col) => col.len(),
            OwnedColumn::Int(col) => col.len(),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::VarBinary(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.len(),
        }
    }

    /// Returns the column with its entries permutated
    pub fn try_permute(&self, permutation: &Permutation) -> Result<Self, PermutationError> {
        Ok(match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(permutation.try_apply(col)?),
            OwnedColumn::TinyInt(col) => OwnedColumn::TinyInt(permutation.try_apply(col)?),
            OwnedColumn::Uint8(col) => OwnedColumn::Uint8(permutation.try_apply(col)?),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(permutation.try_apply(col)?),
            OwnedColumn::Int(col) => OwnedColumn::Int(permutation.try_apply(col)?),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(permutation.try_apply(col)?),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(permutation.try_apply(col)?),
            OwnedColumn::VarBinary(col) => OwnedColumn::VarBinary(permutation.try_apply(col)?),
            OwnedColumn::Int128(col) => OwnedColumn::Int128(permutation.try_apply(col)?),
            OwnedColumn::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, permutation.try_apply(col)?)
            }
            OwnedColumn::Scalar(col) => OwnedColumn::Scalar(permutation.try_apply(col)?),
            OwnedColumn::TimestampTZ(tu, tz, col) => {
                OwnedColumn::TimestampTZ(*tu, *tz, permutation.try_apply(col)?)
            }
        })
    }

    /// Returns the sliced column.
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> Self {
        match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(col[start..end].to_vec()),
            OwnedColumn::TinyInt(col) => OwnedColumn::TinyInt(col[start..end].to_vec()),
            OwnedColumn::Uint8(col) => OwnedColumn::Uint8(col[start..end].to_vec()),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(col[start..end].to_vec()),
            OwnedColumn::Int(col) => OwnedColumn::Int(col[start..end].to_vec()),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(col[start..end].to_vec()),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(col[start..end].to_vec()),
            OwnedColumn::VarBinary(col) => OwnedColumn::VarBinary(col[start..end].to_vec()),
            OwnedColumn::Int128(col) => OwnedColumn::Int128(col[start..end].to_vec()),
            OwnedColumn::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, col[start..end].to_vec())
            }
            OwnedColumn::Scalar(col) => OwnedColumn::Scalar(col[start..end].to_vec()),
            OwnedColumn::TimestampTZ(tu, tz, col) => {
                OwnedColumn::TimestampTZ(*tu, *tz, col[start..end].to_vec())
            }
        }
    }

    /// Returns true if the column is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedColumn::Boolean(col) => col.is_empty(),
            OwnedColumn::TinyInt(col) => col.is_empty(),
            OwnedColumn::Uint8(col) => col.is_empty(),
            OwnedColumn::SmallInt(col) => col.is_empty(),
            OwnedColumn::Int(col) => col.is_empty(),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::VarBinary(col) => col.is_empty(),
            OwnedColumn::Int128(col) => col.is_empty(),
            OwnedColumn::Scalar(col) | OwnedColumn::Decimal75(_, _, col) => col.is_empty(),
        }
    }
    /// Returns the type of the column.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            OwnedColumn::Boolean(_) => ColumnType::Boolean,
            OwnedColumn::TinyInt(_) => ColumnType::TinyInt,
            OwnedColumn::Uint8(_) => ColumnType::Uint8,
            OwnedColumn::SmallInt(_) => ColumnType::SmallInt,
            OwnedColumn::Int(_) => ColumnType::Int,
            OwnedColumn::BigInt(_) => ColumnType::BigInt,
            OwnedColumn::VarChar(_) => ColumnType::VarChar,
            OwnedColumn::VarBinary(_) => ColumnType::VarBinary,
            OwnedColumn::Int128(_) => ColumnType::Int128,
            OwnedColumn::Scalar(_) => ColumnType::Scalar,
            OwnedColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
            OwnedColumn::TimestampTZ(tu, tz, _) => ColumnType::TimestampTZ(*tu, *tz),
        }
    }

    /// Convert a slice of scalars to a vec of owned columns
    pub fn try_from_scalars(scalars: &[S], column_type: ColumnType) -> OwnedColumnResult<Self> {
        match column_type {
            ColumnType::Boolean => Ok(OwnedColumn::Boolean(
                scalars
                    .iter()
                    .map(|s| -> Result<bool, _> { TryInto::<bool>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Uint8 => Ok(OwnedColumn::Uint8(
                scalars
                    .iter()
                    .map(|s| -> Result<u8, _> { TryInto::<u8>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::TinyInt => Ok(OwnedColumn::TinyInt(
                scalars
                    .iter()
                    .map(|s| -> Result<i8, _> { TryInto::<i8>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::SmallInt => Ok(OwnedColumn::SmallInt(
                scalars
                    .iter()
                    .map(|s| -> Result<i16, _> { TryInto::<i16>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Int => Ok(OwnedColumn::Int(
                scalars
                    .iter()
                    .map(|s| -> Result<i32, _> { TryInto::<i32>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::BigInt => Ok(OwnedColumn::BigInt(
                scalars
                    .iter()
                    .map(|s| -> Result<i64, _> { TryInto::<i64>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Int128 => Ok(OwnedColumn::Int128(
                scalars
                    .iter()
                    .map(|s| -> Result<i128, _> { TryInto::<i128>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Scalar => Ok(OwnedColumn::Scalar(scalars.to_vec())),
            ColumnType::Decimal75(precision, scale) => {
                Ok(OwnedColumn::Decimal75(precision, scale, scalars.to_vec()))
            }
            ColumnType::TimestampTZ(tu, tz) => {
                let raw_values: Vec<i64> = scalars
                    .iter()
                    .map(|s| -> Result<i64, _> { TryInto::<i64>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?;
                Ok(OwnedColumn::TimestampTZ(tu, tz, raw_values))
            }
            // Can not convert scalars to VarChar
            ColumnType::VarChar | ColumnType::VarBinary => Err(OwnedColumnError::TypeCastError {
                from_type: ColumnType::Scalar,
                to_type: ColumnType::VarChar,
            }),
        }
    }

    /// Convert a slice of option scalars to a vec of owned columns
    pub fn try_from_option_scalars(
        option_scalars: &[Option<S>],
        column_type: ColumnType,
    ) -> OwnedColumnResult<Self> {
        let scalars = option_scalars
            .iter()
            .copied()
            .collect::<Option<Vec<_>>>()
            .ok_or(OwnedColumnError::Unsupported {
                error: "NULL is not supported yet".to_string(),
            })?;
        Self::try_from_scalars(&scalars, column_type)
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [u8], panicking if it is not.
    pub fn u8_iter(&self) -> impl Iterator<Item = &u8> {
        match self {
            OwnedColumn::Uint8(col) => col.iter(),
            _ => panic!("Expected Uint8 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i8], panicking if it is not.
    pub fn i8_iter(&self) -> impl Iterator<Item = &i8> {
        match self {
            OwnedColumn::TinyInt(col) => col.iter(),
            _ => panic!("Expected TinyInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i16], panicking if it is not.
    pub fn i16_iter(&self) -> impl Iterator<Item = &i16> {
        match self {
            OwnedColumn::SmallInt(col) => col.iter(),
            _ => panic!("Expected SmallInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i32], panicking if it is not.
    pub fn i32_iter(&self) -> impl Iterator<Item = &i32> {
        match self {
            OwnedColumn::Int(col) => col.iter(),
            _ => panic!("Expected Int column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i64], panicking if it is not.
    pub fn i64_iter(&self) -> impl Iterator<Item = &i64> {
        match self {
            OwnedColumn::TimestampTZ(_, _, col) | OwnedColumn::BigInt(col) => col.iter(),
            _ => panic!("Expected TimestampTZ or BigInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i128], panicking if it is not.
    pub fn i128_iter(&self) -> impl Iterator<Item = &i128> {
        match self {
            OwnedColumn::Int128(col) => col.iter(),
            _ => panic!("Expected Int128 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [bool], panicking if it is not.
    pub fn bool_iter(&self) -> impl Iterator<Item = &bool> {
        match self {
            OwnedColumn::Boolean(col) => col.iter(),
            _ => panic!("Expected Boolean column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is a [Scalar], panicking if it is not.
    pub fn scalar_iter(&self) -> impl Iterator<Item = &S> {
        match self {
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.iter(),
            _ => panic!("Expected Scalar or Decimal75 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [String], panicking if it is not.
    pub fn string_iter(&self) -> impl Iterator<Item = &String> {
        match self {
            OwnedColumn::VarChar(col) => col.iter(),
            _ => panic!("Expected VarChar column"),
        }
    }
}

impl<'a, S: Scalar> From<&Column<'a, S>> for OwnedColumn<S> {
    fn from(col: &Column<'a, S>) -> Self {
        match col {
            Column::Boolean(col) => OwnedColumn::Boolean(col.to_vec()),
            Column::TinyInt(col) => OwnedColumn::TinyInt(col.to_vec()),
            Column::Uint8(col) => OwnedColumn::Uint8(col.to_vec()),
            Column::SmallInt(col) => OwnedColumn::SmallInt(col.to_vec()),
            Column::Int(col) => OwnedColumn::Int(col.to_vec()),
            Column::BigInt(col) => OwnedColumn::BigInt(col.to_vec()),
            Column::VarChar((col, _)) => {
                OwnedColumn::VarChar(col.iter().map(ToString::to_string).collect())
            }
            Column::VarBinary((col, _)) => {
                OwnedColumn::VarBinary(col.iter().map(|slice| slice.to_vec()).collect())
            }
            Column::Int128(col) => OwnedColumn::Int128(col.to_vec()),
            Column::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, col.to_vec())
            }
            Column::Scalar(col) => OwnedColumn::Scalar(col.to_vec()),
            Column::TimestampTZ(tu, tz, col) => OwnedColumn::TimestampTZ(*tu, *tz, col.to_vec()),
        }
    }
}

impl<S: Scalar> OwnedColumn<S> {
    /// Attempts to coerce a column of scalars to a numeric column of the specified type.
    /// If the specified type is the same as the current column type, the function will return the column as is.
    ///
    /// # Arguments
    ///
    /// * `to_type` - The target numeric column type to coerce to.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - If the coercion is successful.
    /// * `Err(ColumnCoercionError)` - If the coercion fails due to type mismatch or overflow.
    ///
    /// # Errors
    ///
    /// If the specified type is the same as the current column type, the function will not error.
    ///
    /// Otherwise, this function will return an error if:
    /// * The column type is not `Scalar`.
    /// * The target type is not a numeric type.
    /// * There is an overflow during the coercion.
    pub(crate) fn try_coerce_scalar_to_numeric(
        self,
        to_type: ColumnType,
    ) -> Result<Self, ColumnCoercionError> {
        if self.column_type() == to_type {
            Ok(self)
        } else if let OwnedColumn::Scalar(vec) = self {
            match to_type {
                ColumnType::Uint8 => vec
                    .into_iter()
                    .map(TryInto::try_into)
                    .try_collect()
                    .map_err(|_| ColumnCoercionError::Overflow)
                    .map(OwnedColumn::Uint8),
                ColumnType::TinyInt => vec
                    .into_iter()
                    .map(TryInto::try_into)
                    .try_collect()
                    .map_err(|_| ColumnCoercionError::Overflow)
                    .map(OwnedColumn::TinyInt),
                ColumnType::SmallInt => vec
                    .into_iter()
                    .map(TryInto::try_into)
                    .try_collect()
                    .map_err(|_| ColumnCoercionError::Overflow)
                    .map(OwnedColumn::SmallInt),
                ColumnType::Int => vec
                    .into_iter()
                    .map(TryInto::try_into)
                    .try_collect()
                    .map_err(|_| ColumnCoercionError::Overflow)
                    .map(OwnedColumn::Int),
                ColumnType::BigInt => vec
                    .into_iter()
                    .map(TryInto::try_into)
                    .try_collect()
                    .map_err(|_| ColumnCoercionError::Overflow)
                    .map(OwnedColumn::BigInt),
                ColumnType::Int128 => vec
                    .into_iter()
                    .map(TryInto::try_into)
                    .try_collect()
                    .map_err(|_| ColumnCoercionError::Overflow)
                    .map(OwnedColumn::Int128),
                ColumnType::Decimal75(precision, scale) => {
                    Ok(OwnedColumn::Decimal75(precision, scale, vec))
                }
                _ => Err(ColumnCoercionError::InvalidTypeCoercion),
            }
        } else {
            Err(ColumnCoercionError::InvalidTypeCoercion)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
    };
    use alloc::vec;
    use bumpalo::Bump;

    #[test]
    fn we_can_slice_a_column() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        assert_eq!(col.slice(1, 4), OwnedColumn::Int128(vec![2, 3, 4]));
    }

    #[test]
    fn we_can_permute_a_column() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        let permutation = Permutation::try_new(vec![1, 3, 4, 0, 2]).unwrap();
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Int128(vec![2, 4, 5, 1, 3])
        );
    }

    #[test]
    fn we_can_convert_columns_to_owned_columns_round_trip() {
        let alloc = Bump::new();
        // Integers
        let col: Column<'_, TestScalar> = Column::Int128(&[1, 2, 3, 4, 5]);
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(owned_col, OwnedColumn::Int128(vec![1, 2, 3, 4, 5]));
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);

        // Booleans
        let col: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(
            owned_col,
            OwnedColumn::Boolean(vec![true, false, true, false, true])
        );
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);

        // Strings
        let strs = [
            "Space and Time",
            "מרחב וזמן",
            "Χώρος και Χρόνος",
            "Տարածություն և ժամանակ",
            "ቦታ እና ጊዜ",
            "სივრცე და დრო",
        ];
        let scalars = strs.iter().map(TestScalar::from).collect::<Vec<_>>();
        let col: Column<'_, TestScalar> = Column::VarChar((&strs, &scalars));
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(
            owned_col,
            OwnedColumn::VarChar(
                strs.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            )
        );
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);

        // Decimals
        let scalars: Vec<TestScalar> = [1, 2, 3, 4, 5].iter().map(TestScalar::from).collect();
        let col: Column<'_, TestScalar> =
            Column::Decimal75(Precision::new(75).unwrap(), -128, &scalars);
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(
            owned_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), -128, scalars.clone())
        );
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);
    }

    #[test]
    fn we_can_convert_scalars_to_owned_columns() {
        // Int
        let scalars = [1, 2, 3, 4, 5]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Int128;
        let owned_col = OwnedColumn::try_from_scalars(&scalars, column_type).unwrap();
        assert_eq!(owned_col, OwnedColumn::Int128(vec![1, 2, 3, 4, 5]));

        // Boolean
        let scalars = [true, false, true, false, true]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let owned_col = OwnedColumn::try_from_scalars(&scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Boolean(vec![true, false, true, false, true])
        );

        // Decimal
        let scalars = [1, 2, 3, 4, 5]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), -128);
        let owned_col = OwnedColumn::try_from_scalars(&scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), -128, scalars)
        );
    }

    #[test]
    fn we_cannot_convert_scalars_to_owned_columns_if_varchar() {
        let scalars = ["a", "b", "c", "d", "e"]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::VarChar;
        let res = OwnedColumn::try_from_scalars(&scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::TypeCastError { .. })));
    }

    #[test]
    fn we_cannot_convert_scalars_to_owned_columns_if_overflow() {
        // Int
        let scalars = [i128::MAX, i128::MAX, i128::MAX, i128::MAX, i128::MAX]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::BigInt;
        let res = OwnedColumn::try_from_scalars(&scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // Boolean
        let scalars = [i128::MAX, i128::MAX, i128::MAX, i128::MAX, i128::MAX]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let res = OwnedColumn::try_from_scalars(&scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));
    }

    #[test]
    fn we_can_convert_option_scalars_to_owned_columns() {
        // Int
        let option_scalars = [Some(1), Some(2), Some(3), Some(4), Some(5)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Int128;
        let owned_col = OwnedColumn::try_from_option_scalars(&option_scalars, column_type).unwrap();
        assert_eq!(owned_col, OwnedColumn::Int128(vec![1, 2, 3, 4, 5]));

        // Boolean
        let option_scalars = [Some(true), Some(false), Some(true), Some(false), Some(true)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let owned_col = OwnedColumn::try_from_option_scalars(&option_scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Boolean(vec![true, false, true, false, true])
        );

        // Decimal
        let option_scalars = [Some(1), Some(2), Some(3), Some(4), Some(5)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let scalars = [1, 2, 3, 4, 5]
            .iter()
            .map(|&i| TestScalar::from(i))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), 127);
        let owned_col = OwnedColumn::try_from_option_scalars(&option_scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), 127, scalars)
        );
    }

    #[test]
    fn we_cannot_convert_option_scalars_to_owned_columns_if_varchar() {
        let option_scalars = ["a", "b", "c", "d", "e"]
            .iter()
            .map(|s| Some(TestScalar::from(*s)))
            .collect::<Vec<_>>();
        let column_type = ColumnType::VarChar;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::TypeCastError { .. })));
    }

    #[test]
    fn we_cannot_convert_option_scalars_to_owned_columns_if_overflow() {
        // Int
        let option_scalars = [
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
        ]
        .iter()
        .map(|s| s.map(TestScalar::from))
        .collect::<Vec<_>>();
        let column_type = ColumnType::BigInt;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // Boolean
        let option_scalars = [
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
        ]
        .iter()
        .map(|s| s.map(TestScalar::from))
        .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_option_scalars_to_owned_columns_if_none() {
        // Int
        let option_scalars = [Some(1), Some(2), None, Some(4), Some(5)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Int128;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::Unsupported { .. })));

        // Boolean
        let option_scalars = [Some(true), Some(false), None, Some(false), Some(true)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::Unsupported { .. })));
    }

    #[test]
    fn we_can_coerce_scalar_to_numeric() {
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let col = OwnedColumn::Scalar(scalars.clone());

        // Coerce to TinyInt
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::TinyInt)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::TinyInt(vec![1, 2, 3]));

        // Coerce to SmallInt
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::SmallInt)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::SmallInt(vec![1, 2, 3]));

        // Coerce to Int
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::Int)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::Int(vec![1, 2, 3]));

        // Coerce to BigInt
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::BigInt)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::BigInt(vec![1, 2, 3]));

        // Coerce to Int128
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::Int128)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::Int128(vec![1, 2, 3]));

        // Coerce to Decimal75
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::Decimal75(Precision::new(75).unwrap(), 0))
            .unwrap();
        assert_eq!(
            coerced_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), 0, scalars)
        );
    }

    #[test]
    fn we_cannot_coerce_scalar_to_invalid_type() {
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let col = OwnedColumn::Scalar(scalars);

        // Attempt to coerce to VarChar
        let res = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::VarChar);
        assert!(matches!(res, Err(ColumnCoercionError::InvalidTypeCoercion)));

        // Attempt to coerce non-scalar column
        let col = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let res = col.try_coerce_scalar_to_numeric(ColumnType::BigInt);
        assert!(matches!(res, Err(ColumnCoercionError::InvalidTypeCoercion)));
    }

    #[test]
    fn we_cannot_coerce_scalar_to_numeric_if_overflow() {
        let scalars = vec![TestScalar::from(i128::MAX), -TestScalar::from(i128::MIN)];
        let col = OwnedColumn::Scalar(scalars);

        // Attempt to coerce to TinyInt
        let res = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::TinyInt);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to SmallInt
        let res = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::SmallInt);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to Int
        let res = col.clone().try_coerce_scalar_to_numeric(ColumnType::Int);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to BigInt
        let res = col.clone().try_coerce_scalar_to_numeric(ColumnType::BigInt);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to Int128
        let res = col.try_coerce_scalar_to_numeric(ColumnType::Int128);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));
    }

    #[test]
    fn we_can_slice_and_permute_varbinary_columns() {
        let col = OwnedColumn::<TestScalar>::VarBinary(vec![
            b"foo".to_vec(),
            b"bar".to_vec(),
            b"baz".to_vec(),
            b"qux".to_vec(),
        ]);
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::VarBinary(vec![b"bar".to_vec(), b"baz".to_vec()])
        );
        let permutation = Permutation::try_new(vec![2, 0, 3, 1]).unwrap();
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::VarBinary(vec![
                b"baz".to_vec(),
                b"foo".to_vec(),
                b"qux".to_vec(),
                b"bar".to_vec()
            ])
        );
    }

    #[test]
    fn we_can_convert_varbinary_column_round_trip_using_hash() {
        let raw_bytes = [b"abc".as_ref(), b"xyz".as_ref()];

        let scalars: Vec<TestScalar> = raw_bytes
            .iter()
            .map(|&b| TestScalar::from_byte_slice_via_hash(b))
            .collect();

        let col: Column<'_, TestScalar> =
            Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));

        let owned_col: OwnedColumn<TestScalar> = (&col).into();

        assert_eq!(
            owned_col,
            OwnedColumn::VarBinary(vec![b"abc".to_vec(), b"xyz".to_vec()])
        );

        let bump = bumpalo::Bump::new();
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &bump);

        assert_eq!(col, new_col);
    }

    #[test]
    fn we_can_compute_inner_product_with_varbinary_columns_using_hash() {
        let lhs = OwnedColumn::<TestScalar>::VarBinary(vec![
            b"foo".to_vec(),
            b"bar".to_vec(),
            b"baz".to_vec(),
        ]);

        let scalars = vec![
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];

        let product = lhs.inner_product(&scalars);

        let lhs_hashes: Vec<TestScalar> = [b"foo".as_ref(), b"bar".as_ref(), b"baz".as_ref()]
            .iter()
            .map(|&bytes| TestScalar::from_byte_slice_via_hash(bytes))
            .collect();

        let expected =
            lhs_hashes[0] * scalars[0] + lhs_hashes[1] * scalars[1] + lhs_hashes[2] * scalars[2];

        assert_eq!(product, expected);
    }

    #[test]
    fn we_can_check_is_empty_for_all_column_types() {
        // Boolean
        let col: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true]);
        assert!(!col.is_empty());

        // TinyInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1]);
        assert!(!col.is_empty());

        // Uint8
        let col: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1]);
        assert!(!col.is_empty());

        // SmallInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![1]);
        assert!(!col.is_empty());

        // Int
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1]);
        assert!(!col.is_empty());

        // BigInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1]);
        assert!(!col.is_empty());

        // Int128
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1]);
        assert!(!col.is_empty());

        // VarChar
        let col: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec!["test".to_string()]);
        assert!(!col.is_empty());

        // VarBinary
        let col: OwnedColumn<TestScalar> = OwnedColumn::VarBinary(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::VarBinary(vec![vec![1, 2]]);
        assert!(!col.is_empty());

        // Scalar
        let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(vec![TestScalar::from(1)]);
        assert!(!col.is_empty());

        // Decimal75
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::Decimal75(Precision::new(10).unwrap(), 2, vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::Decimal75(Precision::new(10).unwrap(), 2, vec![TestScalar::from(1)]);
        assert!(!col.is_empty());

        // TimestampTZ
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![]);
        assert!(col.is_empty());
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![100]);
        assert!(!col.is_empty());
    }

    #[test]
    fn we_can_slice_all_column_types() {
        // Boolean
        let col: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, false, true, false]);
        assert_eq!(col.slice(1, 3), OwnedColumn::Boolean(vec![false, true]));

        // TinyInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3, 4]);
        assert_eq!(col.slice(1, 3), OwnedColumn::TinyInt(vec![2, 3]));

        // Uint8
        let col: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3, 4]);
        assert_eq!(col.slice(1, 3), OwnedColumn::Uint8(vec![2, 3]));

        // SmallInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 20, 30, 40]);
        assert_eq!(col.slice(1, 3), OwnedColumn::SmallInt(vec![20, 30]));

        // Int
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![100, 200, 300, 400]);
        assert_eq!(col.slice(1, 3), OwnedColumn::Int(vec![200, 300]));

        // BigInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1000, 2000, 3000, 4000]);
        assert_eq!(col.slice(1, 3), OwnedColumn::BigInt(vec![2000, 3000]));

        // Int128
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![10000, 20000, 30000, 40000]);
        assert_eq!(col.slice(1, 3), OwnedColumn::Int128(vec![20000, 30000]));

        // VarChar
        let col: OwnedColumn<TestScalar> = OwnedColumn::VarChar(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::VarChar(vec!["b".to_string(), "c".to_string()])
        );

        // VarBinary
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::VarBinary(vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]]);
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::VarBinary(vec![vec![3, 4], vec![5, 6]])
        );

        // Scalar
        let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
            TestScalar::from(4),
        ]);
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::Scalar(vec![TestScalar::from(2), TestScalar::from(3)])
        );

        // Decimal75
        let col: OwnedColumn<TestScalar> = OwnedColumn::Decimal75(
            Precision::new(10).unwrap(),
            2,
            vec![
                TestScalar::from(10),
                TestScalar::from(20),
                TestScalar::from(30),
                TestScalar::from(40),
            ],
        );
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::Decimal75(
                Precision::new(10).unwrap(),
                2,
                vec![TestScalar::from(20), TestScalar::from(30)]
            )
        );

        // TimestampTZ
        let col: OwnedColumn<TestScalar> = OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            vec![1, 2, 3, 4],
        );
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![2, 3])
        );
    }

    #[test]
    fn we_can_permute_all_column_types() {
        let permutation = Permutation::try_new(vec![2, 0, 1]).unwrap();

        // Boolean
        let col: OwnedColumn<TestScalar> = OwnedColumn::Boolean(vec![true, false, true]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Boolean(vec![true, true, false])
        );

        // TinyInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::TinyInt(vec![3, 1, 2])
        );

        // Uint8
        let col: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![10, 20, 30]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Uint8(vec![30, 10, 20])
        );

        // SmallInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![100, 200, 300]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::SmallInt(vec![300, 100, 200])
        );

        // Int
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![1000, 2000, 3000]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Int(vec![3000, 1000, 2000])
        );

        // BigInt
        let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![10000, 20000, 30000]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::BigInt(vec![30000, 10000, 20000])
        );

        // Int128
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![100_000, 200_000, 300_000]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Int128(vec![300_000, 100_000, 200_000])
        );

        // VarChar
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::VarChar(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::VarChar(vec!["c".to_string(), "a".to_string(), "b".to_string()])
        );

        // VarBinary
        let col: OwnedColumn<TestScalar> =
            OwnedColumn::VarBinary(vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::VarBinary(vec![vec![5, 6], vec![1, 2], vec![3, 4]])
        );

        // Scalar
        let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ]);
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Scalar(vec![
                TestScalar::from(3),
                TestScalar::from(1),
                TestScalar::from(2)
            ])
        );

        // Decimal75
        let col: OwnedColumn<TestScalar> = OwnedColumn::Decimal75(
            Precision::new(10).unwrap(),
            2,
            vec![
                TestScalar::from(10),
                TestScalar::from(20),
                TestScalar::from(30),
            ],
        );
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Decimal75(
                Precision::new(10).unwrap(),
                2,
                vec![
                    TestScalar::from(30),
                    TestScalar::from(10),
                    TestScalar::from(20)
                ]
            )
        );

        // TimestampTZ
        let col: OwnedColumn<TestScalar> = OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            vec![100, 200, 300],
        );
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![300, 100, 200]
            )
        );
    }

    #[test]
    fn we_can_convert_scalars_to_numeric_types() {
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        // Uint8
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::Uint8).unwrap();
        assert_eq!(col, OwnedColumn::Uint8(vec![1, 2, 3]));

        // TinyInt
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::TinyInt).unwrap();
        assert_eq!(col, OwnedColumn::TinyInt(vec![1, 2, 3]));

        // SmallInt
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::SmallInt).unwrap();
        assert_eq!(col, OwnedColumn::SmallInt(vec![1, 2, 3]));

        // Int
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::Int).unwrap();
        assert_eq!(col, OwnedColumn::Int(vec![1, 2, 3]));

        // BigInt
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::BigInt).unwrap();
        assert_eq!(col, OwnedColumn::BigInt(vec![1, 2, 3]));

        // Int128
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::Int128).unwrap();
        assert_eq!(col, OwnedColumn::Int128(vec![1, 2, 3]));

        // Scalar
        let col = OwnedColumn::try_from_scalars(&scalars, ColumnType::Scalar).unwrap();
        assert_eq!(col, OwnedColumn::Scalar(scalars.clone()));

        // TimestampTZ
        let col = OwnedColumn::try_from_scalars(
            &scalars,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        )
        .unwrap();
        assert_eq!(
            col,
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![1, 2, 3])
        );
    }

    #[test]
    fn we_can_use_iterator_methods() {
        // u8_iter
        let col: OwnedColumn<TestScalar> = OwnedColumn::Uint8(vec![1, 2, 3]);
        let values: Vec<_> = col.u8_iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);

        // i8_iter
        let col: OwnedColumn<TestScalar> = OwnedColumn::TinyInt(vec![1, 2, 3]);
        let values: Vec<_> = col.i8_iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);

        // i16_iter
        let col: OwnedColumn<TestScalar> = OwnedColumn::SmallInt(vec![10, 20, 30]);
        let values: Vec<_> = col.i16_iter().copied().collect();
        assert_eq!(values, vec![10, 20, 30]);

        // i32_iter
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int(vec![100, 200, 300]);
        let values: Vec<_> = col.i32_iter().copied().collect();
        assert_eq!(values, vec![100, 200, 300]);

        // i64_iter
        let col: OwnedColumn<TestScalar> = OwnedColumn::BigInt(vec![1000, 2000, 3000]);
        let values: Vec<_> = col.i64_iter().copied().collect();
        assert_eq!(values, vec![1000, 2000, 3000]);

        // i128_iter
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1000, 2000, 3000]);
        let values: Vec<_> = col.i128_iter().copied().collect();
        assert_eq!(values, vec![1000, 2000, 3000]);

        // scalar_iter
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let col: OwnedColumn<TestScalar> = OwnedColumn::Scalar(scalars.clone());
        let values: Vec<_> = col.scalar_iter().copied().collect();
        assert_eq!(values, scalars);
    }

    #[test]
    fn we_can_coerce_scalar_to_uint8() {
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let col = OwnedColumn::Scalar(scalars);

        let coerced = col.try_coerce_scalar_to_numeric(ColumnType::Uint8).unwrap();
        assert_eq!(coerced, OwnedColumn::Uint8(vec![1, 2, 3]));
    }

    #[test]
    fn we_cannot_convert_scalars_to_numeric_with_overflow() {
        let large_scalars = vec![TestScalar::from(i128::MAX)];

        // Uint8 overflow
        let res = OwnedColumn::try_from_scalars(&large_scalars, ColumnType::Uint8);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // TinyInt overflow
        let res = OwnedColumn::try_from_scalars(&large_scalars, ColumnType::TinyInt);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // SmallInt overflow
        let res = OwnedColumn::try_from_scalars(&large_scalars, ColumnType::SmallInt);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // Int overflow
        let res = OwnedColumn::try_from_scalars(&large_scalars, ColumnType::Int);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // BigInt overflow
        let res = OwnedColumn::try_from_scalars(&large_scalars, ColumnType::BigInt);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // TimestampTZ overflow
        let res = OwnedColumn::try_from_scalars(
            &large_scalars,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        );
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));
    }
}
