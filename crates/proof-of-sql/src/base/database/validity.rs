//! Validity mask utilities for nullable column support.
//!
//! This module provides utilities for working with validity masks (null bitmaps)
//! in column data. A validity mask is a boolean slice where `true` indicates
//! a valid (non-null) value and `false` indicates a null value.
//!
//! ## Canonical Null Invariant
//!
//! When a value is null (validity[i] == false), the corresponding value slot
//! must contain a canonical default value:
//! - Numeric types: 0
//! - String types: empty string
//! - Binary types: empty slice
//!
//! This invariant is critical for proof soundness - it prevents provers from
//! hiding arbitrary values under null entries.

use alloc::vec::Vec;

/// Combines two validity masks using AND logic.
///
/// If both masks are None (all valid), returns None.
/// If one mask is None, returns the other.
/// If both masks are present, returns element-wise AND.
///
/// # Arguments
/// * `lhs` - Left validity mask (None means all valid)
/// * `rhs` - Right validity mask (None means all valid)
///
/// # Returns
/// Combined validity mask (None if all valid)
#[must_use]
pub fn combine_validity(lhs: Option<&[bool]>, rhs: Option<&[bool]>) -> Option<Vec<bool>> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some(v), None) | (None, Some(v)) => Some(v.to_vec()),
        (Some(l), Some(r)) => {
            debug_assert_eq!(l.len(), r.len(), "Validity masks must have same length");
            Some(l.iter().zip(r.iter()).map(|(&a, &b)| a && b).collect())
        }
    }
}

/// Combines owned validity masks using AND logic.
///
/// More efficient version when we already own the vectors.
#[must_use]
pub fn combine_validity_owned(lhs: Option<Vec<bool>>, rhs: Option<Vec<bool>>) -> Option<Vec<bool>> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some(v), None) | (None, Some(v)) => Some(v),
        (Some(l), Some(r)) => {
            debug_assert_eq!(l.len(), r.len(), "Validity masks must have same length");
            Some(l.into_iter().zip(r).map(|(a, b)| a && b).collect())
        }
    }
}

/// Creates an all-valid mask of the given length.
#[must_use]
pub fn all_valid(len: usize) -> Vec<bool> {
    vec![true; len]
}

/// Creates an all-null mask of the given length.
#[must_use]
pub fn all_null(len: usize) -> Vec<bool> {
    vec![false; len]
}

/// Checks if any value in the mask is null (false).
#[must_use]
pub fn has_nulls(validity: Option<&[bool]>) -> bool {
    validity.is_some_and(|v| v.iter().any(|&b| !b))
}

/// Counts the number of null values in the mask.
#[must_use]
pub fn null_count(validity: Option<&[bool]>) -> usize {
    validity.map_or(0, |v| v.iter().filter(|&&b| !b).count())
}

/// Counts the number of valid (non-null) values in the mask.
#[must_use]
pub fn valid_count(validity: Option<&[bool]>, total_len: usize) -> usize {
    validity.map_or(total_len, |v| v.iter().filter(|&&b| b).count())
}

/// Enforces canonical null values for a numeric slice.
///
/// Sets all values at null positions to zero.
///
/// # Arguments
/// * `values` - Mutable slice of values to canonicalize
/// * `validity` - Validity mask (None means all valid, no changes needed)
pub fn canonicalize_nulls_numeric<T: Default + Copy>(values: &mut [T], validity: Option<&[bool]>) {
    if let Some(v) = validity {
        debug_assert_eq!(
            values.len(),
            v.len(),
            "Values and validity must have same length"
        );
        for (val, &is_valid) in values.iter_mut().zip(v.iter()) {
            if !is_valid {
                *val = T::default();
            }
        }
    }
}

/// Creates a new vector with canonical null values for numeric types.
///
/// # Arguments
/// * `values` - Source values
/// * `validity` - Validity mask (None means all valid)
///
/// # Returns
/// New vector with nulls set to default value
#[must_use]
pub fn with_canonical_nulls_numeric<T: Default + Copy>(
    values: &[T],
    validity: Option<&[bool]>,
) -> Vec<T> {
    match validity {
        None => values.to_vec(),
        Some(v) => {
            debug_assert_eq!(
                values.len(),
                v.len(),
                "Values and validity must have same length"
            );
            values
                .iter()
                .zip(v.iter())
                .map(|(&val, &is_valid)| if is_valid { val } else { T::default() })
                .collect()
        }
    }
}

/// Slices a validity mask.
#[must_use]
pub fn slice_validity(validity: Option<&[bool]>, start: usize, end: usize) -> Option<Vec<bool>> {
    validity.map(|v| v[start..end].to_vec())
}

/// Filters a validity mask by a selection vector.
#[must_use]
pub fn filter_validity(validity: Option<&[bool]>, selection: &[bool]) -> Option<Vec<bool>> {
    validity.map(|v| {
        v.iter()
            .zip(selection.iter())
            .filter_map(|(&val, &sel)| if sel { Some(val) } else { None })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_validity_both_none() {
        assert!(combine_validity(None, None).is_none());
    }

    #[test]
    fn test_combine_validity_one_none() {
        let v = vec![true, false, true];
        assert_eq!(combine_validity(Some(&v), None), Some(v.clone()));
        assert_eq!(combine_validity(None, Some(&v)), Some(v));
    }

    #[test]
    fn test_combine_validity_both_some() {
        let l = vec![true, true, false, false];
        let r = vec![true, false, true, false];
        let expected = vec![true, false, false, false];
        assert_eq!(combine_validity(Some(&l), Some(&r)), Some(expected));
    }

    #[test]
    fn test_has_nulls() {
        assert!(!has_nulls(None));
        assert!(!has_nulls(Some(&[true, true, true])));
        assert!(has_nulls(Some(&[true, false, true])));
    }

    #[test]
    fn test_null_count() {
        assert_eq!(null_count(None), 0);
        assert_eq!(null_count(Some(&[true, true, true])), 0);
        assert_eq!(null_count(Some(&[true, false, true])), 1);
        assert_eq!(null_count(Some(&[false, false, false])), 3);
    }

    #[test]
    fn test_canonicalize_nulls_numeric() {
        let mut values = vec![10i64, 20, 30, 40];
        let validity = vec![true, false, true, false];
        canonicalize_nulls_numeric(&mut values, Some(&validity));
        assert_eq!(values, vec![10, 0, 30, 0]);
    }

    #[test]
    fn test_with_canonical_nulls_numeric() {
        let values = vec![10i64, 20, 30, 40];
        let validity = vec![true, false, true, false];
        let result = with_canonical_nulls_numeric(&values, Some(&validity));
        assert_eq!(result, vec![10, 0, 30, 0]);
    }

    #[test]
    fn test_slice_validity() {
        let v = vec![true, false, true, false, true];
        assert_eq!(
            slice_validity(Some(&v), 1, 4),
            Some(vec![false, true, false])
        );
        assert!(slice_validity(None, 1, 4).is_none());
    }
}
