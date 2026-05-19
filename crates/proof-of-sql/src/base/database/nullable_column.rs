use super::{
    Column, ColumnOperationError, ColumnOperationResult, NullableOwnedColumn, OwnedColumn,
};
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

    fn binary_presence(
        &self,
        rhs: &Self,
        alloc: &'a Bump,
    ) -> ColumnOperationResult<Option<&'a [bool]>> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }

        Ok(match (self.presence(), rhs.presence()) {
            (None, None) => None,
            (Some(lhs), None) => Some(alloc.alloc_slice_copy(lhs)),
            (None, Some(rhs)) => Some(alloc.alloc_slice_copy(rhs)),
            (Some(lhs), Some(rhs)) => {
                Some(alloc.alloc_slice_fill_iter(
                    lhs.iter().zip(rhs.iter()).map(|(lhs, rhs)| *lhs && *rhs),
                ))
            }
        })
    }

    fn apply_binary_operation(
        &self,
        rhs: &Self,
        alloc: &'a Bump,
        operation: impl FnOnce(
            &OwnedColumn<S>,
            &OwnedColumn<S>,
        ) -> ColumnOperationResult<OwnedColumn<S>>,
    ) -> ColumnOperationResult<Self> {
        let lhs_values = OwnedColumn::from(&self.values);
        let rhs_values = OwnedColumn::from(&rhs.values);
        let values = alloc.alloc(operation(&lhs_values, &rhs_values)?);
        let presence = self.binary_presence(rhs, alloc)?;
        Self::try_new(Column::from_owned_column(values, alloc), presence)
    }

    /// Adds two columns element-wise and propagates nullable presence data.
    pub fn element_wise_add(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, alloc, OwnedColumn::element_wise_add)
    }

    /// Subtracts two columns element-wise and propagates nullable presence data.
    pub fn element_wise_sub(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, alloc, OwnedColumn::element_wise_sub)
    }

    /// Multiplies two columns element-wise and propagates nullable presence data.
    pub fn element_wise_mul(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, alloc, OwnedColumn::element_wise_mul)
    }

    /// Compares two columns for equality and propagates nullable presence data.
    pub fn element_wise_eq(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, alloc, OwnedColumn::element_wise_eq)
    }

    /// Compares two columns with less-than and propagates nullable presence data.
    pub fn element_wise_lt(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, alloc, OwnedColumn::element_wise_lt)
    }

    /// Compares two columns with greater-than and propagates nullable presence data.
    pub fn element_wise_gt(&self, rhs: &Self, alloc: &'a Bump) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, alloc, OwnedColumn::element_wise_gt)
    }

    /// Returns a non-nullable boolean column matching SQL `IS NULL`.
    #[must_use]
    pub fn is_null(&self, alloc: &'a Bump) -> Column<'a, S> {
        Column::Boolean(match self.presence {
            Some(presence) => {
                alloc.alloc_slice_fill_iter(presence.iter().map(|is_present| !*is_present))
            }
            None => alloc.alloc_slice_fill_copy(self.len(), false),
        })
    }

    /// Returns a non-nullable boolean column matching SQL `IS NOT NULL`.
    #[must_use]
    pub fn is_not_null(&self, alloc: &'a Bump) -> Column<'a, S> {
        Column::Boolean(match self.presence {
            Some(presence) => alloc.alloc_slice_copy(presence),
            None => alloc.alloc_slice_fill_copy(self.len(), true),
        })
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

    #[test]
    fn nullable_column_null_checks_reflect_presence() {
        let alloc = Bump::new();
        let column = NullableColumn::<TestScalar>::try_new(
            Column::BigInt(&[10, 20, 30]),
            Some(&[true, false, true]),
        )
        .unwrap();

        assert_eq!(
            column.is_null(&alloc),
            Column::Boolean(&[false, true, false])
        );
        assert_eq!(
            column.is_not_null(&alloc),
            Column::Boolean(&[true, false, true])
        );
    }

    #[test]
    fn nonnullable_column_null_checks_are_constant() {
        let alloc = Bump::new();
        let column = NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[10, 20, 30]));

        assert_eq!(
            column.is_null(&alloc),
            Column::Boolean(&[false, false, false])
        );
        assert_eq!(
            column.is_not_null(&alloc),
            Column::Boolean(&[true, true, true])
        );
    }

    #[test]
    fn nullable_column_propagates_presence_from_nullable_lhs() {
        let alloc = Bump::new();
        let lhs = NullableColumn::<TestScalar>::try_new(
            Column::BigInt(&[10, 20, 30]),
            Some(&[true, false, true]),
        )
        .unwrap();
        let rhs = NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[1, 2, 3]));

        let result = lhs.element_wise_add(&rhs, &alloc).unwrap();

        assert_eq!(
            result,
            NullableColumn::try_new(Column::BigInt(&[11, 22, 33]), Some(&[true, false, true]))
                .unwrap()
        );
    }

    #[test]
    fn nullable_column_ands_presence_for_two_nullable_columns() {
        let alloc = Bump::new();
        let lhs = NullableColumn::<TestScalar>::try_new(
            Column::BigInt(&[10, 20, 30]),
            Some(&[true, false, true]),
        )
        .unwrap();
        let rhs = NullableColumn::<TestScalar>::try_new(
            Column::BigInt(&[1, 2, 3]),
            Some(&[false, true, true]),
        )
        .unwrap();

        let result = lhs.element_wise_add(&rhs, &alloc).unwrap();

        assert_eq!(
            result,
            NullableColumn::try_new(Column::BigInt(&[11, 22, 33]), Some(&[false, false, true]))
                .unwrap()
        );
    }

    #[test]
    fn nullable_column_keeps_nonnullable_result_when_inputs_are_nonnullable() {
        let alloc = Bump::new();
        let lhs = NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[10, 20, 30]));
        let rhs = NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[1, 2, 3]));

        let result = lhs.element_wise_add(&rhs, &alloc).unwrap();

        assert_eq!(
            result,
            NullableColumn::new_nonnullable(Column::BigInt(&[11, 22, 33]))
        );
    }

    #[test]
    fn nullable_column_propagates_presence_for_comparison_results() {
        let alloc = Bump::new();
        let lhs = NullableColumn::<TestScalar>::try_new(
            Column::BigInt(&[10, 20, 30]),
            Some(&[true, false, true]),
        )
        .unwrap();
        let rhs = NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[10, 2, 30]));

        let result = lhs.element_wise_eq(&rhs, &alloc).unwrap();

        assert_eq!(
            result,
            NullableColumn::try_new(
                Column::Boolean(&[true, false, true]),
                Some(&[true, false, true])
            )
            .unwrap()
        );
    }
}
