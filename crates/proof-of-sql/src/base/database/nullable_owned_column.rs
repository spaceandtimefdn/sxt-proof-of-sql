use super::{ColumnOperationError, ColumnOperationResult, OwnedColumn};
use crate::base::scalar::Scalar;
use alloc::{vec, vec::Vec};
use serde::{Deserialize, Serialize};

/// An owned column plus optional row-presence data.
///
/// `None` means the column is non-nullable. `Some(presence)` means the column is
/// nullable, with `true` for present values and `false` for SQL `NULL`.
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub struct NullableOwnedColumn<S: Scalar> {
    values: OwnedColumn<S>,
    presence: Option<Vec<bool>>,
}

impl<S: Scalar> NullableOwnedColumn<S> {
    /// Creates a nullable owned column and checks that presence matches value length.
    pub fn try_new(
        values: OwnedColumn<S>,
        presence: Option<Vec<bool>>,
    ) -> ColumnOperationResult<Self> {
        if let Some(presence) = &presence {
            if presence.len() != values.len() {
                return Err(ColumnOperationError::DifferentColumnLength {
                    len_a: values.len(),
                    len_b: presence.len(),
                });
            }
        }
        Ok(Self { values, presence })
    }

    /// Wraps a non-nullable owned column.
    #[must_use]
    pub const fn new_nonnullable(values: OwnedColumn<S>) -> Self {
        Self {
            values,
            presence: None,
        }
    }

    /// Returns the backing non-nullable values.
    #[must_use]
    pub const fn values(&self) -> &OwnedColumn<S> {
        &self.values
    }

    /// Returns the optional row-presence slice.
    #[must_use]
    pub fn presence(&self) -> Option<&[bool]> {
        self.presence.as_deref()
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

    /// Returns the backing values and optional row-presence vector.
    #[must_use]
    pub fn into_parts(self) -> (OwnedColumn<S>, Option<Vec<bool>>) {
        (self.values, self.presence)
    }

    fn binary_presence(&self, rhs: &Self) -> ColumnOperationResult<Option<Vec<bool>>> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }

        Ok(match (self.presence(), rhs.presence()) {
            (None, None) => None,
            (Some(lhs), None) => Some(lhs.to_vec()),
            (None, Some(rhs)) => Some(rhs.to_vec()),
            (Some(lhs), Some(rhs)) => Some(
                lhs.iter()
                    .zip(rhs.iter())
                    .map(|(lhs, rhs)| *lhs && *rhs)
                    .collect(),
            ),
        })
    }

    fn apply_binary_operation(
        &self,
        rhs: &Self,
        operation: impl FnOnce(
            &OwnedColumn<S>,
            &OwnedColumn<S>,
        ) -> ColumnOperationResult<OwnedColumn<S>>,
    ) -> ColumnOperationResult<Self> {
        let values = operation(&self.values, &rhs.values)?;
        let presence = self.binary_presence(rhs)?;
        Self::try_new(values, presence)
    }

    /// Adds two columns element-wise and propagates nullable presence data.
    pub fn element_wise_add(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, OwnedColumn::element_wise_add)
    }

    /// Subtracts two columns element-wise and propagates nullable presence data.
    pub fn element_wise_sub(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, OwnedColumn::element_wise_sub)
    }

    /// Multiplies two columns element-wise and propagates nullable presence data.
    pub fn element_wise_mul(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, OwnedColumn::element_wise_mul)
    }

    /// Compares two columns for equality and propagates nullable presence data.
    pub fn element_wise_eq(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, OwnedColumn::element_wise_eq)
    }

    /// Compares two columns with less-than and propagates nullable presence data.
    pub fn element_wise_lt(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, OwnedColumn::element_wise_lt)
    }

    /// Compares two columns with greater-than and propagates nullable presence data.
    pub fn element_wise_gt(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.apply_binary_operation(rhs, OwnedColumn::element_wise_gt)
    }

    /// Returns a non-nullable boolean column matching SQL `IS NULL`.
    #[must_use]
    pub fn is_null(&self) -> OwnedColumn<S> {
        OwnedColumn::Boolean(match self.presence() {
            Some(presence) => presence.iter().map(|is_present| !*is_present).collect(),
            None => vec![false; self.len()],
        })
    }

    /// Returns a non-nullable boolean column matching SQL `IS NOT NULL`.
    #[must_use]
    pub fn is_not_null(&self) -> OwnedColumn<S> {
        OwnedColumn::Boolean(match self.presence() {
            Some(presence) => presence.to_vec(),
            None => vec![true; self.len()],
        })
    }
}

impl<S: Scalar> From<OwnedColumn<S>> for NullableOwnedColumn<S> {
    fn from(values: OwnedColumn<S>) -> Self {
        Self::new_nonnullable(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use alloc::vec;

    #[test]
    fn we_reject_presence_with_wrong_length() {
        let result = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![10, 20]),
            Some(vec![true]),
        );

        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
    }

    #[test]
    fn we_propagate_presence_from_nullable_lhs() {
        let lhs = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![10, 20, 30]),
            Some(vec![true, false, true]),
        )
        .unwrap();
        let rhs =
            NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]));

        let result = lhs.element_wise_add(&rhs).unwrap();

        assert_eq!(
            result,
            NullableOwnedColumn::try_new(
                OwnedColumn::<TestScalar>::BigInt(vec![11, 22, 33]),
                Some(vec![true, false, true])
            )
            .unwrap()
        );
    }

    #[test]
    fn we_propagate_presence_from_nullable_rhs() {
        let lhs = NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![
            10, 20, 30,
        ]));
        let rhs = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]),
            Some(vec![true, false, true]),
        )
        .unwrap();

        let result = lhs.element_wise_add(&rhs).unwrap();

        assert_eq!(
            result,
            NullableOwnedColumn::try_new(
                OwnedColumn::<TestScalar>::BigInt(vec![11, 22, 33]),
                Some(vec![true, false, true])
            )
            .unwrap()
        );
    }

    #[test]
    fn we_and_presence_for_two_nullable_columns() {
        let lhs = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![10, 20, 30]),
            Some(vec![true, false, true]),
        )
        .unwrap();
        let rhs = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]),
            Some(vec![false, true, true]),
        )
        .unwrap();

        let result = lhs.element_wise_add(&rhs).unwrap();

        assert_eq!(
            result,
            NullableOwnedColumn::try_new(
                OwnedColumn::<TestScalar>::BigInt(vec![11, 22, 33]),
                Some(vec![false, false, true])
            )
            .unwrap()
        );
    }

    #[test]
    fn we_keep_nonnullable_result_when_inputs_are_nonnullable() {
        let lhs = NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![
            10, 20, 30,
        ]));
        let rhs =
            NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]));

        let result = lhs.element_wise_add(&rhs).unwrap();

        assert_eq!(
            result,
            NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![
                11, 22, 33
            ]))
        );
    }

    #[test]
    fn we_propagate_presence_for_comparison_results() {
        let lhs = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![10, 20, 30]),
            Some(vec![true, false, true]),
        )
        .unwrap();
        let rhs = NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![
            10, 2, 30,
        ]));

        let result = lhs.element_wise_eq(&rhs).unwrap();

        assert_eq!(
            result,
            NullableOwnedColumn::try_new(
                OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]),
                Some(vec![true, false, true])
            )
            .unwrap()
        );
    }

    #[test]
    fn nullable_owned_column_null_checks_reflect_presence() {
        let column = NullableOwnedColumn::try_new(
            OwnedColumn::<TestScalar>::BigInt(vec![10, 20, 30]),
            Some(vec![true, false, true]),
        )
        .unwrap();

        assert_eq!(
            column.is_null(),
            OwnedColumn::<TestScalar>::Boolean(vec![false, true, false])
        );
        assert_eq!(
            column.is_not_null(),
            OwnedColumn::<TestScalar>::Boolean(vec![true, false, true])
        );
    }

    #[test]
    fn nonnullable_owned_column_null_checks_are_constant() {
        let column = NullableOwnedColumn::new_nonnullable(OwnedColumn::<TestScalar>::BigInt(vec![
            10, 20, 30,
        ]));

        assert_eq!(
            column.is_null(),
            OwnedColumn::<TestScalar>::Boolean(vec![false, false, false])
        );
        assert_eq!(
            column.is_not_null(),
            OwnedColumn::<TestScalar>::Boolean(vec![true, true, true])
        );
    }
}
