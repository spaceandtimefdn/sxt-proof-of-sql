//! Nullable Arrow array conversion utilities.
//!
//! This module provides functions to convert Arrow arrays with null values
//! into Proof of SQL nullable column types, preserving validity information.
//!
//! ## Key Features
//!
//! - Extracts validity bitmaps from Arrow arrays
//! - Enforces canonical null values (0 for numeric, empty for strings)
//! - Creates `NullableOwnedColumn` from Arrow arrays
//!
//! ## Usage
//!
//! ```ignore
//! use arrow::array::Int64Array;
//! let array = Int64Array::from(vec![Some(1), None, Some(3)]);
//! let nullable_col = nullable_bigint_from_arrow(&array)?;
//! ```

use crate::base::{
    database::{NullableOwnedColumn, OwnedColumn},
    scalar::Scalar,
};
use alloc::vec::Vec;
use arrow::array::{Array, Int64Array};
use snafu::Snafu;

/// Errors that can occur during nullable Arrow conversion.
#[derive(Debug, Snafu, PartialEq)]
pub enum NullableArrowConversionError {
    /// The array type is not supported for nullable conversion.
    #[snafu(display("unsupported array type for nullable conversion"))]
    UnsupportedType,
}

/// Extracts the validity mask from an Arrow array.
///
/// Returns `None` if the array has no nulls (all valid).
/// Returns `Some(Vec<bool>)` where `true` = valid, `false` = null.
#[must_use]
pub fn extract_validity(array: &dyn Array) -> Option<Vec<bool>> {
    if array.null_count() == 0 {
        return None;
    }

    let validity: Vec<bool> = (0..array.len()).map(|i| array.is_valid(i)).collect();
    Some(validity)
}

/// Converts an Arrow Int64Array to a NullableOwnedColumn<BigInt>.
///
/// - Extracts the validity bitmap
/// - Enforces canonical null values (0 for null positions)
/// - Returns a NullableOwnedColumn with both data and validity
///
/// # Arguments
/// * `array` - The Arrow Int64Array to convert
///
/// # Returns
/// A `NullableOwnedColumn` containing BigInt values with validity mask.
#[must_use]
pub fn nullable_bigint_from_arrow<S: Scalar>(array: &Int64Array) -> NullableOwnedColumn<S> {
    let validity = extract_validity(array);

    // Extract values, using 0 for null positions (will be canonicalized anyway)
    let values: Vec<i64> = (0..array.len())
        .map(|i| {
            if array.is_valid(i) {
                array.value(i)
            } else {
                0 // Canonical null value
            }
        })
        .collect();

    NullableOwnedColumn::new(OwnedColumn::BigInt(values), validity)
}

/// Converts an Arrow Int64Array slice to a NullableOwnedColumn<BigInt>.
///
/// # Arguments
/// * `array` - The Arrow Int64Array to convert
/// * `start` - Start index (inclusive)
/// * `end` - End index (exclusive)
#[must_use]
pub fn nullable_bigint_from_arrow_slice<S: Scalar>(
    array: &Int64Array,
    start: usize,
    end: usize,
) -> NullableOwnedColumn<S> {
    let len = end - start;

    let validity = if array.null_count() == 0 {
        None
    } else {
        let v: Vec<bool> = (start..end).map(|i| array.is_valid(i)).collect();
        // Only return Some if there are actual nulls in the slice
        if v.iter().all(|&b| b) {
            None
        } else {
            Some(v)
        }
    };

    let values: Vec<i64> = (start..end)
        .map(|i| {
            if array.is_valid(i) {
                array.value(i)
            } else {
                0 // Canonical null value
            }
        })
        .collect();

    NullableOwnedColumn::new(OwnedColumn::BigInt(values), validity)
}

/// Checks if an Arrow array has any null values.
#[must_use]
pub fn has_nulls(array: &dyn Array) -> bool {
    array.null_count() > 0
}

/// Computes the validity mask for a range of an Arrow array.
#[must_use]
pub fn validity_for_range(array: &dyn Array, start: usize, end: usize) -> Option<Vec<bool>> {
    if array.null_count() == 0 {
        return None;
    }

    let validity: Vec<bool> = (start..end).map(|i| array.is_valid(i)).collect();

    // If all valid in range, return None
    if validity.iter().all(|&b| b) {
        None
    } else {
        Some(validity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use arrow::array::Int64Array;

    #[test]
    fn test_extract_validity_no_nulls() {
        let array = Int64Array::from(vec![1i64, 2, 3, 4, 5]);
        assert!(extract_validity(&array).is_none());
    }

    #[test]
    fn test_extract_validity_with_nulls() {
        let array = Int64Array::from(vec![Some(1i64), None, Some(3), None, Some(5)]);
        let validity = extract_validity(&array).unwrap();
        assert_eq!(validity, vec![true, false, true, false, true]);
    }

    #[test]
    fn test_nullable_bigint_from_arrow_no_nulls() {
        let array = Int64Array::from(vec![10i64, 20, 30]);
        let nullable: NullableOwnedColumn<TestScalar> = nullable_bigint_from_arrow(&array);

        assert!(!nullable.has_nulls());
        assert!(!nullable.is_nullable());

        if let OwnedColumn::BigInt(vals) = nullable.column() {
            assert_eq!(vals, &[10, 20, 30]);
        } else {
            panic!("Expected BigInt");
        }
    }

    #[test]
    fn test_nullable_bigint_from_arrow_with_nulls() {
        let array = Int64Array::from(vec![Some(10i64), None, Some(30), None]);
        let nullable: NullableOwnedColumn<TestScalar> = nullable_bigint_from_arrow(&array);

        assert!(nullable.has_nulls());
        assert!(nullable.is_nullable());
        assert_eq!(nullable.null_count(), 2);

        // Check values - nulls should be canonical (0)
        if let OwnedColumn::BigInt(vals) = nullable.column() {
            assert_eq!(vals, &[10, 0, 30, 0]);
        } else {
            panic!("Expected BigInt");
        }

        // Check validity
        assert_eq!(
            nullable.validity(),
            Some(vec![true, false, true, false].as_slice())
        );
    }

    #[test]
    fn test_nullable_bigint_from_arrow_all_nulls() {
        let array = Int64Array::from(vec![None, None, None]);
        let nullable: NullableOwnedColumn<TestScalar> = nullable_bigint_from_arrow(&array);

        assert_eq!(nullable.null_count(), 3);

        if let OwnedColumn::BigInt(vals) = nullable.column() {
            assert_eq!(vals, &[0, 0, 0]); // All canonical
        } else {
            panic!("Expected BigInt");
        }
    }

    #[test]
    fn test_nullable_bigint_from_arrow_slice() {
        let array = Int64Array::from(vec![Some(1i64), None, Some(3), None, Some(5)]);
        let nullable: NullableOwnedColumn<TestScalar> =
            nullable_bigint_from_arrow_slice(&array, 1, 4);

        // Slice is [None, Some(3), None]
        assert_eq!(nullable.len(), 3);
        assert_eq!(nullable.null_count(), 2);

        if let OwnedColumn::BigInt(vals) = nullable.column() {
            assert_eq!(vals, &[0, 3, 0]);
        } else {
            panic!("Expected BigInt");
        }
    }

    #[test]
    fn test_validity_for_range_no_nulls_in_range() {
        let array = Int64Array::from(vec![None, Some(2i64), Some(3), None]);
        // Range [1, 3) has no nulls
        let validity = validity_for_range(&array, 1, 3);
        assert!(validity.is_none());
    }

    #[test]
    fn test_validity_for_range_with_nulls() {
        let array = Int64Array::from(vec![None, Some(2i64), None, Some(4)]);
        let validity = validity_for_range(&array, 0, 4);
        assert_eq!(validity, Some(vec![false, true, false, true]));
    }
}
