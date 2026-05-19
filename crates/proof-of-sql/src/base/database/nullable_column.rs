use super::{Column, ColumnOperationError, ColumnOperationResult, NullableOwnedColumn};
use crate::base::scalar::Scalar;
use bumpalo::Bump;

/// A borrowed column plus optional row-presence data.
///
/// `None` means the column is non-nullable. `Some(presence)` means the column is
/// nullable, with `true` for present values and `false` for SQL `NULL`.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct NullableColumn<'a, S: Scalar> {
    values: Column<'a, S>,
    presence: Option<&'a [bool]>,
}

impl<'a, S: Scalar> NullableColumn<'a, S> {
    /// Creates a nullable column and checks that the presence slice matches the value length.
    pub fn try_new(
        values: Column<'a, S>,
        presence: Option<&'a [bool]>,
    ) -> ColumnOperationResult<Self> {
        if let Some(presence) = presence {
            if presence.len() != values.len() {
                return Err(ColumnOperationError::DifferentColumnLength {
                    len_a: values.len(),
                    len_b: presence.len(),
                });
            }
        }
        Ok(Self { values, presence })
    }

    /// Wraps a non-nullable column.
    #[must_use]
    pub const fn new_nonnullable(values: Column<'a, S>) -> Self {
        Self {
            values,
            presence: None,
        }
    }

    /// Converts an owned nullable column into a borrowed nullable column.
    pub fn from_owned_column(
        owned_column: &'a NullableOwnedColumn<S>,
        alloc: &'a Bump,
    ) -> ColumnOperationResult<Self> {
        Self::try_new(
            Column::from_owned_column(owned_column.values(), alloc),
            owned_column.presence(),
        )
    }

    /// Returns the backing non-nullable values.
    #[must_use]
    pub const fn values(&self) -> Column<'a, S> {
        self.values
    }

    /// Returns the optional row-presence slice.
    #[must_use]
    pub const fn presence(&self) -> Option<&'a [bool]> {
        self.presence
    }

    /// Returns whether this column carries nullable presence data.
    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        self.presence.is_some()
    }

    /// Returns the length of the value and presence columns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the column has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<'a, S: Scalar> From<Column<'a, S>> for NullableColumn<'a, S> {
    fn from(values: Column<'a, S>) -> Self {
        Self::new_nonnullable(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{database::OwnedColumn, scalar::test_scalar::TestScalar};
    use alloc::vec;

    #[test]
    fn we_can_borrow_a_nullable_owned_column() {
        let alloc = Bump::new();
        let owned = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![10, 20, 30]),
            Some(vec![true, false, true]),
        )
        .unwrap();

        let column = NullableColumn::from_owned_column(&owned, &alloc).unwrap();

        assert_eq!(column.values(), Column::BigInt(&[10, 20, 30]));
        assert_eq!(column.presence(), Some([true, false, true].as_slice()));
        assert!(column.is_nullable());
        assert_eq!(column.len(), 3);
    }

    #[test]
    fn we_reject_borrowed_presence_with_wrong_length() {
        let result =
            NullableColumn::<TestScalar>::try_new(Column::BigInt(&[10, 20]), Some(&[true]));

        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
    }
}
