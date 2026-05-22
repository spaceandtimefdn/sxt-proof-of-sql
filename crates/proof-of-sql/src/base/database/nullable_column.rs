use super::{
    and_validity_masks, canonicalize_nulls, ensure_canonical_column_nulls, ensure_canonical_nulls,
    filter_valid_owned_values, validate_mask_length, validity_column, Column, ColumnOperationError,
    ColumnType, OwnedColumn, ValidityError,
};
use crate::base::{math::permutation::Permutation, scalar::Scalar};
use alloc::vec::Vec;
use snafu::Snafu;

/// Errors for nullable column construction and operations.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum NullableColumnError {
    /// Validity mask or canonical-null invariant failed.
    #[snafu(transparent)]
    Validity {
        /// The underlying validity error.
        source: ValidityError,
    },
    /// Backing column operation failed.
    #[snafu(transparent)]
    ColumnOperation {
        /// The underlying column operation error.
        source: ColumnOperationError,
    },
}

/// Result type for nullable column operations.
pub type NullableColumnResult<T> = core::result::Result<T, NullableColumnError>;

/// An owned nullable column represented as values plus a validity mask.
///
/// Invalid rows must contain the canonical null sentinel for their physical
/// column type. This makes nullable columns deterministic for commitments while
/// the validity mask carries SQL null semantics.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct NullableOwnedColumn<S: Scalar> {
    values: OwnedColumn<S>,
    validity: Vec<bool>,
}

impl<S: Scalar> NullableOwnedColumn<S> {
    /// Creates a nullable column from already-canonicalized values.
    pub fn try_new(values: OwnedColumn<S>, validity: Vec<bool>) -> NullableColumnResult<Self> {
        ensure_canonical_nulls(&values, &validity)?;
        Ok(Self { values, validity })
    }

    /// Creates a nullable column and canonicalizes invalid rows in-place.
    pub fn try_new_canonicalized(
        mut values: OwnedColumn<S>,
        validity: Vec<bool>,
    ) -> NullableColumnResult<Self> {
        canonicalize_nulls(&mut values, &validity)?;
        Ok(Self { values, validity })
    }

    /// Returns the backing values column.
    #[must_use]
    pub fn values(&self) -> &OwnedColumn<S> {
        &self.values
    }

    /// Returns the validity mask.
    #[must_use]
    pub fn validity(&self) -> &[bool] {
        &self.validity
    }

    /// Consumes the nullable column into its values and validity mask.
    #[must_use]
    pub fn into_parts(self) -> (OwnedColumn<S>, Vec<bool>) {
        (self.values, self.validity)
    }

    /// Returns the number of rows in the nullable column.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the nullable column has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the physical type of the nullable column values.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.values.column_type()
    }

    /// Returns the number of non-null rows.
    #[must_use]
    pub fn valid_len(&self) -> usize {
        self.validity.iter().filter(|is_valid| **is_valid).count()
    }

    /// Returns the validity mask as a boolean `OwnedColumn`.
    #[must_use]
    pub fn validity_column(&self) -> OwnedColumn<S> {
        validity_column(&self.validity)
    }

    /// Returns only the values whose rows are non-null.
    pub fn valid_values(&self) -> NullableColumnResult<OwnedColumn<S>> {
        Ok(filter_valid_owned_values(&self.values, &self.validity)?)
    }

    /// Returns the nullable column with its rows permuted.
    pub fn try_permute(
        &self,
        permutation: &Permutation,
    ) -> Result<Self, crate::base::math::permutation::PermutationError> {
        let values = self.values.try_permute(permutation)?;
        let validity = permutation.try_apply(&self.validity)?;
        Ok(Self { values, validity })
    }

    /// Returns the sliced nullable column.
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self {
            values: self.values.slice(start, end),
            validity: self.validity[start..end].to_vec(),
        }
    }

    /// Adds a nullable column to a non-nullable column.
    ///
    /// The output is null exactly where `self` is null.
    pub fn try_element_wise_add_nonnullable(
        &self,
        rhs: &OwnedColumn<S>,
    ) -> NullableColumnResult<Self> {
        validate_mask_length(rhs.len(), &self.validity)?;
        let mut values = self.values.element_wise_add(rhs)?;
        canonicalize_nulls(&mut values, &self.validity)?;
        Ok(Self {
            values,
            validity: self.validity.clone(),
        })
    }

    /// Adds two nullable columns.
    ///
    /// The output is null unless both input rows are non-null.
    pub fn try_element_wise_add_nullable(&self, rhs: &Self) -> NullableColumnResult<Self> {
        let validity = and_validity_masks(&self.validity, &rhs.validity)?;
        let mut values = self.values.element_wise_add(&rhs.values)?;
        canonicalize_nulls(&mut values, &validity)?;
        Ok(Self { values, validity })
    }
}

/// A borrowed nullable column represented as values plus a validity mask.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct NullableColumn<'a, S: Scalar> {
    values: Column<'a, S>,
    validity: &'a [bool],
}

impl<'a, S: Scalar> NullableColumn<'a, S> {
    /// Creates a borrowed nullable column from canonicalized values.
    pub fn try_new(values: Column<'a, S>, validity: &'a [bool]) -> NullableColumnResult<Self> {
        ensure_canonical_column_nulls(&values, validity)?;
        Ok(Self { values, validity })
    }

    /// Returns the backing values column.
    #[must_use]
    pub fn values(&self) -> Column<'a, S> {
        self.values
    }

    /// Returns the validity mask.
    #[must_use]
    pub fn validity(&self) -> &'a [bool] {
        self.validity
    }

    /// Returns the number of rows in the nullable column.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the nullable column has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the physical type of the nullable column values.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.values.column_type()
    }
}
