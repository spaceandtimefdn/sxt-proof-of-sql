use super::OwnedColumn;
use crate::base::scalar::Scalar;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use sqlparser::ast::Ident;

/// Errors returned by nullable column proof-of-concept helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullableColumnError {
    /// Two aligned nullable-column buffers must have the same length.
    LengthMismatch {
        /// Length of the first buffer.
        left: usize,
        /// Length of the second buffer.
        right: usize,
    },
    /// Invalid rows must store the canonical null value.
    NonCanonicalNullValue {
        /// Row index with the invalid value.
        index: usize,
        /// Non-canonical value found at the null row.
        value: i64,
    },
    /// Adding two non-null bigint values overflowed.
    IntegerOverflow {
        /// Left operand value.
        left: i64,
        /// Right operand value.
        right: i64,
    },
    /// The Arrow array type is not supported by this proof-of-concept wrapper.
    UnsupportedArrowType {
        /// The unsupported Arrow datatype name.
        datatype: String,
    },
}

impl fmt::Display for NullableColumnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NullableColumnError::LengthMismatch { left, right } => write!(
                f,
                "nullable column buffer length {left} does not match aligned buffer length {right}"
            ),
            NullableColumnError::NonCanonicalNullValue { index, value } => write!(
                f,
                "nullable bigint row {index} stores non-canonical null value {value}"
            ),
            NullableColumnError::IntegerOverflow { left, right } => {
                write!(f, "overflow in nullable bigint addition {left} + {right}")
            }
            NullableColumnError::UnsupportedArrowType { datatype } => {
                write!(f, "unsupported nullable bigint Arrow datatype {datatype}")
            }
        }
    }
}

/// Result type for nullable column proof-of-concept helpers.
pub type NullableColumnResult<T> = Result<T, NullableColumnError>;

/// Proof-of-concept nullable `BIGINT` column backed by a value vector and a validity mask.
///
/// This does not add a public SQL-facing nullable type yet. It provides the narrow
/// data model needed for a sound POC: null rows are canonicalized to zero, and the
/// validity mask can be committed and referenced by proof expressions separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullableBigIntColumn {
    values: Vec<i64>,
    validity: Vec<bool>,
}

impl NullableBigIntColumn {
    /// Create a nullable bigint column from already-canonical physical columns.
    pub fn try_new(values: Vec<i64>, validity: Vec<bool>) -> NullableColumnResult<Self> {
        if values.len() != validity.len() {
            return Err(NullableColumnError::LengthMismatch {
                left: values.len(),
                right: validity.len(),
            });
        }

        if let Some((index, value)) = values
            .iter()
            .zip(validity.iter())
            .enumerate()
            .find_map(|(index, (value, valid))| (!valid && *value != 0).then_some((index, *value)))
        {
            return Err(NullableColumnError::NonCanonicalNullValue { index, value });
        }

        Ok(Self { values, validity })
    }

    /// Create a nullable bigint column from logical optional values.
    #[must_use]
    pub fn from_options(values: impl IntoIterator<Item = Option<i64>>) -> Self {
        let (values, validity): (Vec<_>, Vec<_>) = values
            .into_iter()
            .map(|value| match value {
                Some(value) => (value, true),
                None => (0_i64, false),
            })
            .unzip();

        Self { values, validity }
    }

    /// Return the canonical physical bigint values.
    #[must_use]
    pub fn values(&self) -> &[i64] {
        &self.values
    }

    /// Return the validity mask where `true` means the row is not null.
    #[must_use]
    pub fn validity(&self) -> &[bool] {
        &self.validity
    }

    /// Return the number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Return true if the nullable column is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Add a non-nullable bigint column, propagating nulls by preserving the validity mask.
    pub fn try_add_bigint(&self, rhs: impl IntoIterator<Item = i64>) -> NullableColumnResult<Self> {
        let rhs: Vec<_> = rhs.into_iter().collect();
        if self.len() != rhs.len() {
            return Err(NullableColumnError::LengthMismatch {
                left: self.len(),
                right: rhs.len(),
            });
        }

        let values = self
            .values
            .iter()
            .zip(rhs.iter())
            .zip(self.validity.iter())
            .map(|((lhs, rhs), valid)| {
                if *valid {
                    lhs.checked_add(*rhs)
                        .ok_or(NullableColumnError::IntegerOverflow {
                            left: *lhs,
                            right: *rhs,
                        })
                } else {
                    Ok(0_i64)
                }
            })
            .collect::<NullableColumnResult<Vec<_>>>()?;

        Self::try_new(values, self.validity.clone())
    }

    /// Return the physical value column used for commitments and proof expressions.
    #[must_use]
    pub fn value_owned_column<S: Scalar>(&self, name: impl Into<Ident>) -> (Ident, OwnedColumn<S>) {
        (name.into(), OwnedColumn::BigInt(self.values.clone()))
    }

    /// Return the physical validity column used for commitments and proof expressions.
    #[must_use]
    pub fn validity_owned_column<S: Scalar>(
        &self,
        name: impl Into<Ident>,
    ) -> (Ident, OwnedColumn<S>) {
        (name.into(), OwnedColumn::Boolean(self.validity.clone()))
    }
}

#[cfg(feature = "arrow")]
impl TryFrom<&arrow::array::ArrayRef> for NullableBigIntColumn {
    type Error = NullableColumnError;

    fn try_from(value: &arrow::array::ArrayRef) -> NullableColumnResult<Self> {
        use arrow::array::{Array, Int64Array};

        let array = value.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
            NullableColumnError::UnsupportedArrowType {
                datatype: value.data_type().to_string(),
            }
        })?;

        let values = (0..array.len())
            .map(|index| {
                if array.is_null(index) {
                    0_i64
                } else {
                    array.value(index)
                }
            })
            .collect();
        let validity = (0..array.len())
            .map(|index| !array.is_null(index))
            .collect();

        Self::try_new(values, validity)
    }
}

#[cfg(test)]
mod tests {
    use super::{NullableBigIntColumn, NullableColumnError};

    #[test]
    fn nullable_bigint_from_options_canonicalizes_nulls() {
        let column = NullableBigIntColumn::from_options([Some(4_i64), None, Some(-3)]);

        assert_eq!(column.values(), &[4, 0, -3]);
        assert_eq!(column.validity(), &[true, false, true]);
    }

    #[test]
    fn nullable_bigint_rejects_non_canonical_null_values() {
        let err = NullableBigIntColumn::try_new(vec![4, 99], vec![true, false]).unwrap_err();

        assert_eq!(
            err,
            NullableColumnError::NonCanonicalNullValue {
                index: 1,
                value: 99
            }
        );
    }

    #[test]
    fn nullable_bigint_adds_non_nullable_bigint_and_propagates_nulls() {
        let lhs = NullableBigIntColumn::from_options([Some(5_i64), None, Some(9)]);
        let result = lhs.try_add_bigint([7_i64, 12, 1]).unwrap();

        assert_eq!(result.values(), &[12, 0, 10]);
        assert_eq!(result.validity(), &[true, false, true]);
    }

    #[test]
    fn nullable_bigint_addition_checks_overflow_for_valid_rows() {
        let lhs = NullableBigIntColumn::from_options([Some(i64::MAX), None]);
        let err = lhs.try_add_bigint([1_i64, i64::MAX]).unwrap_err();

        assert_eq!(
            err,
            NullableColumnError::IntegerOverflow {
                left: i64::MAX,
                right: 1
            }
        );
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn nullable_bigint_from_arrow_int64_canonicalizes_nulls() {
        use alloc::sync::Arc;
        use arrow::array::{ArrayRef, Int64Array};

        let array: ArrayRef = Arc::new(Int64Array::from(vec![Some(4_i64), None, Some(-3)]));
        let column = NullableBigIntColumn::try_from(&array).unwrap();

        assert_eq!(column.values(), &[4, 0, -3]);
        assert_eq!(column.validity(), &[true, false, true]);
    }
}
