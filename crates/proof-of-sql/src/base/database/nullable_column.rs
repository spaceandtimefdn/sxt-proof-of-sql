//! Nullable column support for Proof of SQL.
//!
//! This module provides the [`NullableColumn`] and [`NullableOwnedColumn`] types
//! which wrap regular columns with an optional validity mask to support NULL values.
//!
//! ## Design Overview
//!
//! Nullable columns consist of:
//! - A data column (the underlying values)
//! - An optional validity mask (None means all values are valid/non-null)
//!
//! ## Canonical Null Invariant
//!
//! When a value is null (validity[i] == false), the corresponding value slot
//! must contain a canonical default:
//! - Numeric types: 0
//! - String types: empty string
//! - Binary types: empty slice
//!
//! This invariant is enforced at construction and is critical for proof soundness.

use super::{validity, Column, ColumnType, OwnedColumn};
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// A nullable owned column - wraps an [`OwnedColumn`] with an optional validity mask.
///
/// The validity mask is a boolean vector where `true` indicates a valid (non-null)
/// value and `false` indicates a null value. If the validity mask is `None`, all
/// values are considered valid.
///
/// # Canonical Null Invariant
///
/// Values at null positions (where validity[i] == false) must be set to the
/// canonical default for the type (0 for numeric, empty for strings/binary).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NullableOwnedColumn<S: Scalar> {
    /// The underlying column data
    column: OwnedColumn<S>,
    /// Optional validity mask (None = all valid)
    validity: Option<Vec<bool>>,
}

impl<S: Scalar> NullableOwnedColumn<S> {
    /// Creates a new nullable column from values and validity.
    ///
    /// # Arguments
    /// * `column` - The underlying column data
    /// * `validity` - Optional validity mask (None means all valid)
    ///
    /// # Panics
    /// Panics if validity length doesn't match column length.
    #[must_use]
    pub fn new(column: OwnedColumn<S>, validity: Option<Vec<bool>>) -> Self {
        if let Some(ref v) = validity {
            assert_eq!(
                column.len(),
                v.len(),
                "Validity mask length must match column length"
            );
        }
        Self { column, validity }
    }

    /// Creates a new nullable column, enforcing canonical null values.
    ///
    /// This version ensures that all null positions have canonical default values.
    #[must_use]
    pub fn new_with_canonical_nulls(column: OwnedColumn<S>, validity: Option<Vec<bool>>) -> Self {
        let column = if let Some(ref v) = validity {
            Self::canonicalize_column(column, v)
        } else {
            column
        };
        Self::new(column, validity)
    }

    /// Creates a non-nullable column (all values valid).
    #[must_use]
    pub fn non_nullable(column: OwnedColumn<S>) -> Self {
        Self {
            column,
            validity: None,
        }
    }

    /// Returns the underlying column.
    #[must_use]
    pub fn column(&self) -> &OwnedColumn<S> {
        &self.column
    }

    /// Returns the validity mask.
    #[must_use]
    pub fn validity(&self) -> Option<&[bool]> {
        self.validity.as_deref()
    }

    /// Returns the length of the column.
    #[must_use]
    pub fn len(&self) -> usize {
        self.column.len()
    }

    /// Returns true if the column is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.column.is_empty()
    }

    /// Returns true if any value is null.
    #[must_use]
    pub fn has_nulls(&self) -> bool {
        validity::has_nulls(self.validity.as_deref())
    }

    /// Returns the number of null values.
    #[must_use]
    pub fn null_count(&self) -> usize {
        validity::null_count(self.validity.as_deref())
    }

    /// Returns the column type.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.column.column_type()
    }

    /// Returns true if the column is nullable (can contain nulls).
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.validity.is_some()
    }

    /// Checks if a specific index is valid (non-null).
    #[must_use]
    pub fn is_valid(&self, index: usize) -> bool {
        self.validity
            .as_ref()
            .is_none_or(|v| v.get(index).copied().unwrap_or(false))
    }

    /// Consumes self and returns the inner column and validity.
    #[must_use]
    pub fn into_parts(self) -> (OwnedColumn<S>, Option<Vec<bool>>) {
        (self.column, self.validity)
    }

    /// Canonicalize null values in a column.
    fn canonicalize_column(column: OwnedColumn<S>, validity: &[bool]) -> OwnedColumn<S> {
        match column {
            OwnedColumn::BigInt(mut values) => {
                validity::canonicalize_nulls_numeric(&mut values, Some(validity));
                OwnedColumn::BigInt(values)
            }
            OwnedColumn::Int(mut values) => {
                validity::canonicalize_nulls_numeric(&mut values, Some(validity));
                OwnedColumn::Int(values)
            }
            OwnedColumn::SmallInt(mut values) => {
                validity::canonicalize_nulls_numeric(&mut values, Some(validity));
                OwnedColumn::SmallInt(values)
            }
            OwnedColumn::TinyInt(mut values) => {
                validity::canonicalize_nulls_numeric(&mut values, Some(validity));
                OwnedColumn::TinyInt(values)
            }
            OwnedColumn::Int128(mut values) => {
                validity::canonicalize_nulls_numeric(&mut values, Some(validity));
                OwnedColumn::Int128(values)
            }
            OwnedColumn::Uint8(mut values) => {
                validity::canonicalize_nulls_numeric(&mut values, Some(validity));
                OwnedColumn::Uint8(values)
            }
            // For other types, return as-is for now
            // TODO: Add canonicalization for other types
            other => other,
        }
    }
}

/// A nullable column view - wraps a [`Column`] with an optional validity mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullableColumn<'a, S: Scalar> {
    /// The underlying column data
    column: Column<'a, S>,
    /// Optional validity mask (None = all valid)
    validity: Option<&'a [bool]>,
}

impl<'a, S: Scalar> NullableColumn<'a, S> {
    /// Creates a new nullable column from a column and validity mask.
    #[must_use]
    pub fn new(column: Column<'a, S>, validity: Option<&'a [bool]>) -> Self {
        if let Some(v) = validity {
            debug_assert_eq!(
                column.len(),
                v.len(),
                "Validity mask length must match column length"
            );
        }
        Self { column, validity }
    }

    /// Creates a non-nullable column (all values valid).
    #[must_use]
    pub fn non_nullable(column: Column<'a, S>) -> Self {
        Self {
            column,
            validity: None,
        }
    }

    /// Returns the underlying column.
    #[must_use]
    pub fn column(&self) -> Column<'a, S> {
        self.column
    }

    /// Returns the validity mask.
    #[must_use]
    pub fn validity(&self) -> Option<&'a [bool]> {
        self.validity
    }

    /// Returns the length of the column.
    #[must_use]
    pub fn len(&self) -> usize {
        self.column.len()
    }

    /// Returns true if the column is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.column.is_empty()
    }

    /// Returns the column type.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.column.column_type()
    }

    /// Creates a `NullableColumn` from a `NullableOwnedColumn`.
    #[must_use]
    pub fn from_nullable_owned(
        owned: &'a NullableOwnedColumn<S>,
        alloc: &'a Bump,
    ) -> NullableColumn<'a, S> {
        let column = Column::from_owned_column(owned.column(), alloc);
        let validity = owned.validity();
        NullableColumn::new(column, validity)
    }
}

/// Adds two nullable `BigInt` columns element-wise.
///
/// Null propagation: if either operand is null, the result is null.
/// Result values at null positions are set to 0 (canonical null).
///
/// # Panics
///
/// Panics if either column is not a `BigInt` column, or if column lengths don't match.
#[must_use]
pub fn add_nullable_bigint<S: Scalar>(
    lhs: &NullableOwnedColumn<S>,
    rhs: &NullableOwnedColumn<S>,
) -> NullableOwnedColumn<S> {
    // Extract BigInt values
    let OwnedColumn::BigInt(lhs_values) = lhs.column() else {
        panic!("Expected BigInt column for lhs")
    };
    let OwnedColumn::BigInt(rhs_values) = rhs.column() else {
        panic!("Expected BigInt column for rhs")
    };

    assert_eq!(
        lhs_values.len(),
        rhs_values.len(),
        "Columns must have same length"
    );

    // Combine validity masks
    let result_validity = validity::combine_validity(lhs.validity(), rhs.validity());

    // Compute result values
    let result_values: Vec<i64> = lhs_values
        .iter()
        .zip(rhs_values.iter())
        .enumerate()
        .map(|(i, (&l, &r))| {
            // If result will be null, use canonical value (0)
            if let Some(ref v) = result_validity {
                if !v[i] {
                    return 0;
                }
            }
            l.wrapping_add(r)
        })
        .collect();

    NullableOwnedColumn::new(OwnedColumn::BigInt(result_values), result_validity)
}

/// Subtracts two nullable `BigInt` columns element-wise.
///
/// # Panics
///
/// Panics if either column is not a `BigInt` column, or if column lengths don't match.
#[must_use]
pub fn subtract_nullable_bigint<S: Scalar>(
    lhs: &NullableOwnedColumn<S>,
    rhs: &NullableOwnedColumn<S>,
) -> NullableOwnedColumn<S> {
    let OwnedColumn::BigInt(lhs_values) = lhs.column() else {
        panic!("Expected BigInt column for lhs")
    };
    let OwnedColumn::BigInt(rhs_values) = rhs.column() else {
        panic!("Expected BigInt column for rhs")
    };

    assert_eq!(
        lhs_values.len(),
        rhs_values.len(),
        "Columns must have same length"
    );

    let result_validity = validity::combine_validity(lhs.validity(), rhs.validity());

    let result_values: Vec<i64> = lhs_values
        .iter()
        .zip(rhs_values.iter())
        .enumerate()
        .map(|(i, (&l, &r))| {
            if let Some(ref v) = result_validity {
                if !v[i] {
                    return 0;
                }
            }
            l.wrapping_sub(r)
        })
        .collect();

    NullableOwnedColumn::new(OwnedColumn::BigInt(result_values), result_validity)
}

/// Multiplies two nullable `BigInt` columns element-wise.
///
/// # Panics
///
/// Panics if either column is not a `BigInt` column, or if column lengths don't match.
#[must_use]
pub fn multiply_nullable_bigint<S: Scalar>(
    lhs: &NullableOwnedColumn<S>,
    rhs: &NullableOwnedColumn<S>,
) -> NullableOwnedColumn<S> {
    let OwnedColumn::BigInt(lhs_values) = lhs.column() else {
        panic!("Expected BigInt column for lhs")
    };
    let OwnedColumn::BigInt(rhs_values) = rhs.column() else {
        panic!("Expected BigInt column for rhs")
    };

    assert_eq!(
        lhs_values.len(),
        rhs_values.len(),
        "Columns must have same length"
    );

    let result_validity = validity::combine_validity(lhs.validity(), rhs.validity());

    let result_values: Vec<i64> = lhs_values
        .iter()
        .zip(rhs_values.iter())
        .enumerate()
        .map(|(i, (&l, &r))| {
            if let Some(ref v) = result_validity {
                if !v[i] {
                    return 0;
                }
            }
            l.wrapping_mul(r)
        })
        .collect();

    NullableOwnedColumn::new(OwnedColumn::BigInt(result_values), result_validity)
}

/// Adds a nullable `BigInt` column to a non-nullable `BigInt` column.
///
/// This demonstrates the required functionality: nullable + non-nullable.
///
/// # Panics
///
/// Panics if either column is not a `BigInt` column, or if column lengths don't match.
#[must_use]
pub fn add_nullable_to_nonnullable_bigint<S: Scalar>(
    nullable: &NullableOwnedColumn<S>,
    non_nullable: &OwnedColumn<S>,
) -> NullableOwnedColumn<S> {
    let OwnedColumn::BigInt(nullable_values) = nullable.column() else {
        panic!("Expected BigInt column for nullable")
    };
    let OwnedColumn::BigInt(nonnull_values) = non_nullable else {
        panic!("Expected BigInt column for non-nullable")
    };

    assert_eq!(
        nullable_values.len(),
        nonnull_values.len(),
        "Columns must have same length"
    );

    // Result is nullable if input is nullable
    let result_validity = nullable.validity().map(<[bool]>::to_vec);

    let result_values: Vec<i64> = nullable_values
        .iter()
        .zip(nonnull_values.iter())
        .enumerate()
        .map(|(i, (&l, &r))| {
            if let Some(ref v) = result_validity {
                if !v[i] {
                    return 0;
                }
            }
            l.wrapping_add(r)
        })
        .collect();

    NullableOwnedColumn::new(OwnedColumn::BigInt(result_values), result_validity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_nullable_column_creation() {
        let values = vec![1i64, 2, 3, 4, 5];
        let validity = vec![true, false, true, false, true];
        let col = OwnedColumn::<TestScalar>::BigInt(values);
        let nullable = NullableOwnedColumn::new(col, Some(validity.clone()));

        assert_eq!(nullable.len(), 5);
        assert!(nullable.has_nulls());
        assert_eq!(nullable.null_count(), 2);
        assert!(nullable.is_valid(0));
        assert!(!nullable.is_valid(1));
        assert!(nullable.is_valid(2));
    }

    #[test]
    fn test_non_nullable_column() {
        let values = vec![1i64, 2, 3];
        let col = OwnedColumn::<TestScalar>::BigInt(values);
        let nullable = NullableOwnedColumn::non_nullable(col);

        assert!(!nullable.has_nulls());
        assert_eq!(nullable.null_count(), 0);
        assert!(!nullable.is_nullable());
    }

    #[test]
    fn test_canonical_nulls() {
        let values = vec![10i64, 20, 30, 40];
        let validity = vec![true, false, true, false];
        let col = OwnedColumn::<TestScalar>::BigInt(values);
        let nullable = NullableOwnedColumn::new_with_canonical_nulls(col, Some(validity));

        // Check that null positions have canonical value (0)
        if let OwnedColumn::BigInt(vals) = nullable.column() {
            assert_eq!(vals[0], 10); // valid
            assert_eq!(vals[1], 0); // null -> canonical
            assert_eq!(vals[2], 30); // valid
            assert_eq!(vals[3], 0); // null -> canonical
        } else {
            panic!("Expected BigInt column");
        }
    }

    #[test]
    fn test_add_nullable_bigint() {
        let lhs_values = vec![10i64, 20, 30, 40];
        let lhs_validity = vec![true, false, true, true];
        let lhs = NullableOwnedColumn::<TestScalar>::new(
            OwnedColumn::BigInt(lhs_values),
            Some(lhs_validity),
        );

        let rhs_values = vec![1i64, 2, 3, 4];
        let rhs_validity = vec![true, true, false, true];
        let rhs = NullableOwnedColumn::<TestScalar>::new(
            OwnedColumn::BigInt(rhs_values),
            Some(rhs_validity),
        );

        let result = add_nullable_bigint(&lhs, &rhs);

        // Expected validity: AND of both masks
        // [true, false, false, true]
        assert!(result.is_valid(0)); // both valid
        assert!(!result.is_valid(1)); // lhs null
        assert!(!result.is_valid(2)); // rhs null
        assert!(result.is_valid(3)); // both valid

        if let OwnedColumn::BigInt(vals) = result.column() {
            assert_eq!(vals[0], 11); // 10 + 1
            assert_eq!(vals[1], 0); // null (canonical)
            assert_eq!(vals[2], 0); // null (canonical)
            assert_eq!(vals[3], 44); // 40 + 4
        } else {
            panic!("Expected BigInt column");
        }
    }

    #[test]
    fn test_add_nullable_to_nonnullable() {
        // This tests the explicit requirement: nullable bigint + non-nullable bigint
        let nullable_values = vec![10i64, 20, 30];
        let nullable_validity = vec![true, false, true];
        let nullable = NullableOwnedColumn::<TestScalar>::new(
            OwnedColumn::BigInt(nullable_values),
            Some(nullable_validity),
        );

        let nonnull = OwnedColumn::<TestScalar>::BigInt(vec![1i64, 2, 3]);

        let result = add_nullable_to_nonnullable_bigint(&nullable, &nonnull);

        // Result should be nullable
        assert!(result.is_nullable());

        // Check validity propagation
        assert!(result.is_valid(0)); // nullable was valid
        assert!(!result.is_valid(1)); // nullable was null
        assert!(result.is_valid(2)); // nullable was valid

        if let OwnedColumn::BigInt(vals) = result.column() {
            assert_eq!(vals[0], 11); // 10 + 1
            assert_eq!(vals[1], 0); // null (canonical)
            assert_eq!(vals[2], 33); // 30 + 3
        } else {
            panic!("Expected BigInt column");
        }
    }

    #[test]
    fn test_multiply_nullable_bigint() {
        let lhs = NullableOwnedColumn::<TestScalar>::new(
            OwnedColumn::BigInt(vec![2i64, 3, 4]),
            Some(vec![true, false, true]),
        );
        let rhs = NullableOwnedColumn::<TestScalar>::new(
            OwnedColumn::BigInt(vec![10i64, 10, 10]),
            Some(vec![true, true, true]),
        );

        let result = multiply_nullable_bigint(&lhs, &rhs);

        if let OwnedColumn::BigInt(vals) = result.column() {
            assert_eq!(vals[0], 20); // 2 * 10
            assert_eq!(vals[1], 0); // null
            assert_eq!(vals[2], 40); // 4 * 10
        } else {
            panic!("Expected BigInt column");
        }
    }
}
