use super::{Column, OwnedColumn};
use crate::base::scalar::Scalar;
use alloc::{string::String, vec::Vec};
use snafu::Snafu;

/// Errors for validity-mask based nullable column handling.
#[derive(Snafu, Debug, PartialEq, Eq, Clone)]
pub enum ValidityError {
    /// The value column and validity mask have different lengths.
    #[snafu(display(
        "validity mask length mismatch: value length {value_len}, validity length {validity_len}"
    ))]
    LengthMismatch {
        /// Number of values in the backing column.
        value_len: usize,
        /// Number of entries in the validity mask.
        validity_len: usize,
    },
    /// A null row contains a non-canonical value.
    #[snafu(display("non-canonical null value at row {index}"))]
    NonCanonicalNull {
        /// Row index containing a non-canonical null sentinel.
        index: usize,
    },
}

/// Result type for validity-mask operations.
pub type ValidityResult<T> = core::result::Result<T, ValidityError>;

/// Checks that a values column and validity mask have equal row counts.
pub fn validate_mask_length(value_len: usize, validity: &[bool]) -> ValidityResult<()> {
    if value_len == validity.len() {
        Ok(())
    } else {
        Err(ValidityError::LengthMismatch {
            value_len,
            validity_len: validity.len(),
        })
    }
}

/// Computes the SQL validity mask for a binary operation.
///
/// A binary expression is non-null exactly when both inputs are non-null.
pub fn and_validity_masks(lhs: &[bool], rhs: &[bool]) -> ValidityResult<Vec<bool>> {
    validate_mask_length(lhs.len(), rhs)?;
    Ok(lhs
        .iter()
        .zip(rhs)
        .map(|(left, right)| *left && *right)
        .collect())
}

/// Returns `true` when all rows in the validity mask are non-null.
#[must_use]
pub fn all_rows_valid(validity: &[bool]) -> bool {
    validity.iter().all(|is_valid| *is_valid)
}

/// Replaces every invalid row with the canonical null sentinel for the column type.
pub fn canonicalize_nulls<S: Scalar>(
    values: &mut OwnedColumn<S>,
    validity: &[bool],
) -> ValidityResult<()> {
    validate_mask_length(values.len(), validity)?;
    match values {
        OwnedColumn::Boolean(values) => canonicalize_slice(values, validity, false),
        OwnedColumn::Uint8(values) => canonicalize_slice(values, validity, 0),
        OwnedColumn::TinyInt(values) => canonicalize_slice(values, validity, 0),
        OwnedColumn::SmallInt(values) => canonicalize_slice(values, validity, 0),
        OwnedColumn::Int(values) => canonicalize_slice(values, validity, 0),
        OwnedColumn::BigInt(values) | OwnedColumn::TimestampTZ(_, _, values) => {
            canonicalize_slice(values, validity, 0);
        }
        OwnedColumn::Int128(values) => canonicalize_slice(values, validity, 0),
        OwnedColumn::VarChar(values) => {
            canonicalize_slice(values, validity, String::new());
        }
        OwnedColumn::VarBinary(values) => canonicalize_slice(values, validity, Vec::new()),
        OwnedColumn::Decimal75(_, _, values) | OwnedColumn::Scalar(values) => {
            canonicalize_slice(values, validity, S::ZERO);
        }
    }
    Ok(())
}

/// Checks that every invalid row already contains the canonical null sentinel.
pub fn ensure_canonical_nulls<S: Scalar>(
    values: &OwnedColumn<S>,
    validity: &[bool],
) -> ValidityResult<()> {
    validate_mask_length(values.len(), validity)?;
    match values {
        OwnedColumn::Boolean(values) => ensure_canonical_slice(values, validity, &false),
        OwnedColumn::Uint8(values) => ensure_canonical_slice(values, validity, &0),
        OwnedColumn::TinyInt(values) => ensure_canonical_slice(values, validity, &0),
        OwnedColumn::SmallInt(values) => ensure_canonical_slice(values, validity, &0),
        OwnedColumn::Int(values) => ensure_canonical_slice(values, validity, &0),
        OwnedColumn::BigInt(values) | OwnedColumn::TimestampTZ(_, _, values) => {
            ensure_canonical_slice(values, validity, &0)
        }
        OwnedColumn::Int128(values) => ensure_canonical_slice(values, validity, &0),
        OwnedColumn::VarChar(values) => ensure_canonical_slice(values, validity, &String::new()),
        OwnedColumn::VarBinary(values) => ensure_canonical_slice(values, validity, &Vec::new()),
        OwnedColumn::Decimal75(_, _, values) | OwnedColumn::Scalar(values) => {
            ensure_canonical_slice(values, validity, &S::ZERO)
        }
    }
}

/// Checks that a borrowed column uses canonical null sentinels for invalid rows.
pub fn ensure_canonical_column_nulls<S: Scalar>(
    values: &Column<'_, S>,
    validity: &[bool],
) -> ValidityResult<()> {
    validate_mask_length(values.len(), validity)?;
    match values {
        Column::Boolean(values) => ensure_canonical_slice(values, validity, &false),
        Column::Uint8(values) => ensure_canonical_slice(values, validity, &0),
        Column::TinyInt(values) => ensure_canonical_slice(values, validity, &0),
        Column::SmallInt(values) => ensure_canonical_slice(values, validity, &0),
        Column::Int(values) => ensure_canonical_slice(values, validity, &0),
        Column::BigInt(values) | Column::TimestampTZ(_, _, values) => {
            ensure_canonical_slice(values, validity, &0)
        }
        Column::Int128(values) => ensure_canonical_slice(values, validity, &0),
        Column::VarChar((values, _)) => ensure_canonical_slice(values, validity, &""),
        Column::VarBinary((values, _)) => ensure_canonical_slice(values, validity, &&[][..]),
        Column::Decimal75(_, _, values) | Column::Scalar(values) => {
            ensure_canonical_slice(values, validity, &S::ZERO)
        }
    }
}

/// Filters an owned column down to rows that are marked valid.
pub fn filter_valid_owned_values<S: Scalar>(
    values: &OwnedColumn<S>,
    validity: &[bool],
) -> ValidityResult<OwnedColumn<S>> {
    validate_mask_length(values.len(), validity)?;
    Ok(match values {
        OwnedColumn::Boolean(values) => OwnedColumn::Boolean(filter_valid_slice(values, validity)),
        OwnedColumn::Uint8(values) => OwnedColumn::Uint8(filter_valid_slice(values, validity)),
        OwnedColumn::TinyInt(values) => OwnedColumn::TinyInt(filter_valid_slice(values, validity)),
        OwnedColumn::SmallInt(values) => {
            OwnedColumn::SmallInt(filter_valid_slice(values, validity))
        }
        OwnedColumn::Int(values) => OwnedColumn::Int(filter_valid_slice(values, validity)),
        OwnedColumn::BigInt(values) => OwnedColumn::BigInt(filter_valid_slice(values, validity)),
        OwnedColumn::Int128(values) => OwnedColumn::Int128(filter_valid_slice(values, validity)),
        OwnedColumn::VarChar(values) => OwnedColumn::VarChar(filter_valid_slice(values, validity)),
        OwnedColumn::VarBinary(values) => {
            OwnedColumn::VarBinary(filter_valid_slice(values, validity))
        }
        OwnedColumn::Decimal75(precision, scale, values) => {
            OwnedColumn::Decimal75(*precision, *scale, filter_valid_slice(values, validity))
        }
        OwnedColumn::TimestampTZ(time_unit, timezone, values) => {
            OwnedColumn::TimestampTZ(*time_unit, *timezone, filter_valid_slice(values, validity))
        }
        OwnedColumn::Scalar(values) => OwnedColumn::Scalar(filter_valid_slice(values, validity)),
    })
}

/// Builds a boolean column from a validity mask.
#[must_use]
pub fn validity_column<S: Scalar>(validity: &[bool]) -> OwnedColumn<S> {
    OwnedColumn::Boolean(validity.to_vec())
}

fn canonicalize_slice<T: Clone>(values: &mut [T], validity: &[bool], canonical_null: T) {
    for (value, is_valid) in values.iter_mut().zip(validity) {
        if !*is_valid {
            *value = canonical_null.clone();
        }
    }
}

fn ensure_canonical_slice<T: PartialEq>(
    values: &[T],
    validity: &[bool],
    canonical_null: &T,
) -> ValidityResult<()> {
    values
        .iter()
        .zip(validity)
        .enumerate()
        .find(|(_, (value, is_valid))| !**is_valid && *value != canonical_null)
        .map_or(Ok(()), |(index, _)| {
            Err(ValidityError::NonCanonicalNull { index })
        })
}

fn filter_valid_slice<T: Clone>(values: &[T], validity: &[bool]) -> Vec<T> {
    values
        .iter()
        .zip(validity)
        .filter_map(|(value, is_valid)| is_valid.then(|| value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::vec;

    #[test]
    fn we_can_canonicalize_invalid_rows() {
        let mut column = OwnedColumn::<TestScalar>::BigInt(vec![10, 999, 20]);

        canonicalize_nulls(&mut column, &[true, false, true]).unwrap();

        assert_eq!(column, OwnedColumn::BigInt(vec![10, 0, 20]));
    }

    #[test]
    fn we_reject_noncanonical_null_rows() {
        let column = OwnedColumn::<TestScalar>::VarChar(vec![
            "valid".to_string(),
            "not-null-sentinel".to_string(),
        ]);

        assert_eq!(
            ensure_canonical_nulls(&column, &[true, false]),
            Err(ValidityError::NonCanonicalNull { index: 1 })
        );
    }

    #[test]
    fn we_can_filter_valid_rows() {
        let column = OwnedColumn::<TestScalar>::Int(vec![4, 8, 15, 16, 23]);

        let filtered =
            filter_valid_owned_values(&column, &[true, false, true, false, true]).unwrap();

        assert_eq!(filtered, OwnedColumn::Int(vec![4, 15, 23]));
    }

    #[test]
    fn we_reject_validity_length_mismatches() {
        assert_eq!(
            validate_mask_length(3, &[true, false]),
            Err(ValidityError::LengthMismatch {
                value_len: 3,
                validity_len: 2
            })
        );
    }
}
