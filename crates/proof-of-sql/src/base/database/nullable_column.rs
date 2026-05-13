use super::{Column, ColumnOperationError, ColumnOperationResult, ColumnType, OwnedColumn};
use crate::base::{
    database::OwnedColumnResult,
    math::permutation::{Permutation, PermutationError},
    scalar::Scalar,
};
use alloc::{string::String, vec::Vec};
use bumpalo::Bump;
use snafu::Snafu;
use sqlparser::ast::Ident;

/// Errors from operations related to nullable columns.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum NullableColumnError {
    /// The value column and presence column have different lengths.
    #[snafu(display(
        "Value and presence columns have different lengths: {values_len} != {presence_len}"
    ))]
    PresenceLengthMismatch {
        /// The length of the values column.
        values_len: usize,
        /// The length of the presence column.
        presence_len: usize,
    },
}

/// Result type for operations related to nullable columns.
pub type NullableColumnResult<T> = core::result::Result<T, NullableColumnError>;

/// An owned column with optional nullability metadata.
///
/// `presence == None` means the column is non-nullable. `presence == Some(mask)` means each row is
/// present when the corresponding mask entry is true, and null otherwise. The value column remains
/// the physical representation used by existing column operations and commitments.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct NullableOwnedColumn<S: Scalar> {
    values: OwnedColumn<S>,
    presence: Option<Vec<bool>>,
}

impl<S: Scalar> NullableOwnedColumn<S> {
    /// Create a nullable owned column from a physical value column and optional presence mask.
    pub fn try_new(
        values: OwnedColumn<S>,
        presence: Option<Vec<bool>>,
    ) -> NullableColumnResult<Self> {
        check_presence_len(values.len(), presence.as_deref())?;
        Ok(Self { values, presence })
    }

    /// Create a non-nullable wrapper around an owned column.
    #[must_use]
    pub fn non_nullable(values: OwnedColumn<S>) -> Self {
        Self {
            values,
            presence: None,
        }
    }

    /// Convert logical optional scalars to a nullable owned column.
    ///
    /// Null entries are represented by `S::ZERO` before conversion to the target physical type. If
    /// all entries are present, the returned column is non-nullable.
    pub fn try_from_option_scalars(
        option_scalars: &[Option<S>],
        column_type: ColumnType,
    ) -> OwnedColumnResult<Self> {
        let mut saw_null = false;
        let scalars = option_scalars
            .iter()
            .map(|scalar| {
                scalar.unwrap_or_else(|| {
                    saw_null = true;
                    S::ZERO
                })
            })
            .collect::<Vec<_>>();
        let values = OwnedColumn::try_from_scalars(&scalars, column_type)?;
        Ok(if saw_null {
            Self {
                values,
                presence: Some(option_scalars.iter().map(Option::is_some).collect()),
            }
        } else {
            Self::non_nullable(values)
        })
    }

    /// Return the physical value column.
    #[must_use]
    pub fn values(&self) -> &OwnedColumn<S> {
        &self.values
    }

    /// Return the optional presence mask.
    #[must_use]
    pub fn presence(&self) -> Option<&[bool]> {
        self.presence.as_deref()
    }

    /// Split this nullable column into the physical value column and optional presence mask.
    #[must_use]
    pub fn into_inner(self) -> (OwnedColumn<S>, Option<Vec<bool>>) {
        (self.values, self.presence)
    }

    /// Return the physical value column with a name suitable for an [`OwnedTable`].
    #[must_use]
    pub fn value_owned_column(&self, name: impl Into<Ident>) -> (Ident, OwnedColumn<S>) {
        (name.into(), self.values.clone())
    }

    /// Return the physical presence column with a name suitable for an [`OwnedTable`].
    ///
    /// Returns `None` when this column is non-nullable.
    #[must_use]
    pub fn presence_owned_column(&self, name: impl Into<Ident>) -> Option<(Ident, OwnedColumn<S>)> {
        self.presence
            .as_ref()
            .map(|presence| (name.into(), OwnedColumn::Boolean(presence.clone())))
    }

    /// Return true when this column has a presence mask.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.presence.is_some()
    }

    /// Return the physical column type.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.values.column_type()
    }

    /// Return the number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Return true if the column contains no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Return the column with its entries permuted.
    pub fn try_permute(&self, permutation: &Permutation) -> Result<Self, PermutationError> {
        Ok(Self {
            values: self.values.try_permute(permutation)?,
            presence: self
                .presence
                .as_ref()
                .map(|presence| permutation.try_apply(presence))
                .transpose()?,
        })
    }

    /// Return the sliced column.
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self {
            values: self.values.slice(start, end),
            presence: self
                .presence
                .as_ref()
                .map(|presence| presence[start..end].to_vec()),
        }
    }

    /// Borrow this nullable owned column.
    pub fn as_column<'a>(&'a self, alloc: &'a Bump) -> NullableColumn<'a, S> {
        NullableColumn {
            values: Column::from_owned_column(&self.values, alloc),
            presence: self.presence(),
        }
    }

    /// Element-wise NOT operation on the value column. Presence is propagated unchanged.
    pub fn element_wise_not(&self) -> ColumnOperationResult<Self> {
        let values =
            replace_null_rows_with(&self.values, self.presence(), NullRowReplacement::Zero)
                .element_wise_not()?;
        Ok(Self {
            values: canonicalize_null_rows(values, self.presence()),
            presence: self.presence.clone(),
        })
    }

    /// Element-wise AND operation. Presence is the conjunction of operand presences.
    pub fn element_wise_and(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_and, NullRowReplacement::Zero)
    }

    /// Element-wise OR operation. Presence is the conjunction of operand presences.
    pub fn element_wise_or(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_or, NullRowReplacement::Zero)
    }

    /// Element-wise equality check. Presence is the conjunction of operand presences.
    pub fn element_wise_eq(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_eq, NullRowReplacement::Zero)
    }

    /// Element-wise less-than check. Presence is the conjunction of operand presences.
    pub fn element_wise_lt(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_lt, NullRowReplacement::Zero)
    }

    /// Element-wise greater-than check. Presence is the conjunction of operand presences.
    pub fn element_wise_gt(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_gt, NullRowReplacement::Zero)
    }

    /// Element-wise addition. Presence is the conjunction of operand presences.
    pub fn element_wise_add(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_add, NullRowReplacement::Zero)
    }

    /// Element-wise subtraction. Presence is the conjunction of operand presences.
    pub fn element_wise_sub(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_sub, NullRowReplacement::Zero)
    }

    /// Element-wise multiplication. Presence is the conjunction of operand presences.
    pub fn element_wise_mul(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_mul, NullRowReplacement::Zero)
    }

    /// Element-wise division. Presence is the conjunction of operand presences.
    pub fn element_wise_div(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        self.try_binary_op(rhs, OwnedColumn::element_wise_div, NullRowReplacement::One)
    }

    fn try_binary_op(
        &self,
        rhs: &Self,
        op: impl Fn(&OwnedColumn<S>, &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>>,
        rhs_null_replacement: NullRowReplacement,
    ) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        let presence = combine_presence(self.presence(), rhs.presence());
        let lhs_values =
            replace_null_rows_with(&self.values, presence.as_deref(), NullRowReplacement::Zero);
        let rhs_values =
            replace_null_rows_with(&rhs.values, presence.as_deref(), rhs_null_replacement);
        let values = op(&lhs_values, &rhs_values)?;
        Ok(Self {
            values: canonicalize_null_rows(values, presence.as_deref()),
            presence,
        })
    }
}

impl<'a, S: Scalar> TryFrom<NullableColumn<'a, S>> for NullableOwnedColumn<S> {
    type Error = NullableColumnError;

    fn try_from(value: NullableColumn<'a, S>) -> NullableColumnResult<Self> {
        Self::try_new(
            OwnedColumn::from(&value.values),
            value.presence.map(<[bool]>::to_vec),
        )
    }
}

/// A borrowed column with optional nullability metadata.
///
/// `presence == None` means the column is non-nullable. `presence == Some(mask)` means each row is
/// present when the corresponding mask entry is true, and null otherwise.
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct NullableColumn<'a, S: Scalar> {
    values: Column<'a, S>,
    presence: Option<&'a [bool]>,
}

impl<'a, S: Scalar> NullableColumn<'a, S> {
    /// Create a nullable borrowed column from a physical value column and optional presence mask.
    pub fn try_new(
        values: Column<'a, S>,
        presence: Option<&'a [bool]>,
    ) -> NullableColumnResult<Self> {
        check_presence_len(values.len(), presence)?;
        Ok(Self { values, presence })
    }

    /// Create a non-nullable wrapper around a borrowed column.
    #[must_use]
    pub fn non_nullable(values: Column<'a, S>) -> Self {
        Self {
            values,
            presence: None,
        }
    }

    /// Return the physical value column.
    #[must_use]
    pub fn values(&self) -> Column<'a, S> {
        self.values
    }

    /// Return the optional presence mask.
    #[must_use]
    pub fn presence(&self) -> Option<&'a [bool]> {
        self.presence
    }

    /// Return true when this column has a presence mask.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.presence.is_some()
    }

    /// Return the physical column type.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.values.column_type()
    }

    /// Return the number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Return true if the column contains no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

fn check_presence_len(values_len: usize, presence: Option<&[bool]>) -> NullableColumnResult<()> {
    if let Some(presence) = presence {
        if presence.len() != values_len {
            return Err(NullableColumnError::PresenceLengthMismatch {
                values_len,
                presence_len: presence.len(),
            });
        }
    }
    Ok(())
}

fn combine_presence(lhs: Option<&[bool]>, rhs: Option<&[bool]>) -> Option<Vec<bool>> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some(lhs), None) => Some(lhs.to_vec()),
        (None, Some(rhs)) => Some(rhs.to_vec()),
        (Some(lhs), Some(rhs)) => Some(
            lhs.iter()
                .zip(rhs.iter())
                .map(|(left, right)| *left && *right)
                .collect(),
        ),
    }
}

#[derive(Clone, Copy)]
enum NullRowReplacement {
    Zero,
    One,
}

fn replace_null_rows_with<S: Scalar>(
    column: &OwnedColumn<S>,
    presence: Option<&[bool]>,
    replacement: NullRowReplacement,
) -> OwnedColumn<S> {
    let Some(presence) = presence else {
        return column.clone();
    };
    let mut column = column.clone();
    replace_absent_rows(&mut column, presence, replacement);
    column
}

fn canonicalize_null_rows<S: Scalar>(
    mut column: OwnedColumn<S>,
    presence: Option<&[bool]>,
) -> OwnedColumn<S> {
    if let Some(presence) = presence {
        replace_absent_rows(&mut column, presence, NullRowReplacement::Zero);
    }
    column
}

fn replace_absent_rows<S: Scalar>(
    column: &mut OwnedColumn<S>,
    presence: &[bool],
    replacement: NullRowReplacement,
) {
    match column {
        OwnedColumn::Boolean(values) => {
            replace_absent_values(
                values,
                presence,
                matches!(replacement, NullRowReplacement::One),
            );
        }
        OwnedColumn::Uint8(values) => replace_absent_values(
            values,
            presence,
            match replacement {
                NullRowReplacement::Zero => 0,
                NullRowReplacement::One => 1,
            },
        ),
        OwnedColumn::TinyInt(values) => replace_absent_values(
            values,
            presence,
            match replacement {
                NullRowReplacement::Zero => 0,
                NullRowReplacement::One => 1,
            },
        ),
        OwnedColumn::SmallInt(values) => replace_absent_values(
            values,
            presence,
            match replacement {
                NullRowReplacement::Zero => 0,
                NullRowReplacement::One => 1,
            },
        ),
        OwnedColumn::Int(values) => replace_absent_values(
            values,
            presence,
            match replacement {
                NullRowReplacement::Zero => 0,
                NullRowReplacement::One => 1,
            },
        ),
        OwnedColumn::BigInt(values) | OwnedColumn::TimestampTZ(_, _, values) => {
            replace_absent_values(
                values,
                presence,
                match replacement {
                    NullRowReplacement::Zero => 0,
                    NullRowReplacement::One => 1,
                },
            );
        }
        OwnedColumn::Int128(values) => replace_absent_values(
            values,
            presence,
            match replacement {
                NullRowReplacement::Zero => 0,
                NullRowReplacement::One => 1,
            },
        ),
        OwnedColumn::Decimal75(_, _, values) | OwnedColumn::Scalar(values) => {
            replace_absent_values(
                values,
                presence,
                match replacement {
                    NullRowReplacement::Zero => S::ZERO,
                    NullRowReplacement::One => S::ONE,
                },
            );
        }
        OwnedColumn::VarChar(values) => replace_absent_values(values, presence, String::new()),
        OwnedColumn::VarBinary(values) => replace_absent_values(values, presence, Vec::new()),
    }
}

fn replace_absent_values<T: Clone>(values: &mut [T], presence: &[bool], replacement: T) {
    for (value, present) in values.iter_mut().zip(presence.iter()) {
        if !present {
            *value = replacement.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{math::permutation::Permutation, scalar::test_scalar::TestScalar};
    use alloc::vec;

    #[test]
    fn nullable_owned_column_rejects_mismatched_presence_length() {
        let err = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::BigInt(vec![1, 2, 3]),
            Some(vec![true, false]),
        )
        .unwrap_err();

        assert_eq!(
            err,
            NullableColumnError::PresenceLengthMismatch {
                values_len: 3,
                presence_len: 2
            }
        );
    }

    #[test]
    fn nullable_owned_column_converts_option_scalars() {
        let option_scalars = [Some(1), None, Some(3)]
            .iter()
            .map(|value| value.map(TestScalar::from))
            .collect::<Vec<_>>();

        let column =
            NullableOwnedColumn::try_from_option_scalars(&option_scalars, ColumnType::BigInt)
                .unwrap();

        assert_eq!(column.values(), &OwnedColumn::BigInt(vec![1, 0, 3]));
        assert_eq!(column.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_owned_column_without_nulls_omits_presence_mask() {
        let option_scalars = [Some(1), Some(2), Some(3)]
            .iter()
            .map(|value| value.map(TestScalar::from))
            .collect::<Vec<_>>();

        let column =
            NullableOwnedColumn::try_from_option_scalars(&option_scalars, ColumnType::BigInt)
                .unwrap();

        assert_eq!(column.values(), &OwnedColumn::BigInt(vec![1, 2, 3]));
        assert_eq!(column.presence(), None);
    }

    #[test]
    fn nullable_binary_operations_propagate_one_presence_mask() {
        let lhs = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::BigInt(vec![5, 0, 9]),
            Some(vec![true, false, true]),
        )
        .unwrap();
        let rhs = NullableOwnedColumn::non_nullable(OwnedColumn::BigInt(vec![7, 12, 1]));

        let result = lhs.element_wise_add(&rhs).unwrap();

        assert_eq!(result.values(), &OwnedColumn::BigInt(vec![12, 0, 10]));
        assert_eq!(result.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_binary_operations_conjoin_two_presence_masks() {
        let lhs = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::BigInt(vec![5, 0, 9, 2]),
            Some(vec![true, false, true, true]),
        )
        .unwrap();
        let rhs = NullableOwnedColumn::try_new(
            OwnedColumn::BigInt(vec![7, 12, 0, 3]),
            Some(vec![true, true, false, true]),
        )
        .unwrap();

        let result = lhs.element_wise_add(&rhs).unwrap();

        assert_eq!(result.values(), &OwnedColumn::BigInt(vec![12, 0, 0, 5]));
        assert_eq!(
            result.presence(),
            Some([true, false, false, true].as_slice())
        );
    }

    #[test]
    fn nullable_division_ignores_division_by_zero_on_null_rows() {
        let lhs =
            NullableOwnedColumn::<TestScalar>::non_nullable(OwnedColumn::BigInt(vec![10, 20, 30]));
        let rhs = NullableOwnedColumn::try_new(
            OwnedColumn::BigInt(vec![2, 0, 5]),
            Some(vec![true, false, true]),
        )
        .unwrap();

        let result = lhs.element_wise_div(&rhs).unwrap();

        assert_eq!(result.values(), &OwnedColumn::BigInt(vec![5, 0, 6]));
        assert_eq!(result.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_comparisons_return_nullable_boolean_columns() {
        let lhs = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::BigInt(vec![5, 0, 9]),
            Some(vec![true, false, true]),
        )
        .unwrap();
        let rhs = NullableOwnedColumn::non_nullable(OwnedColumn::BigInt(vec![5, 0, 1]));

        let result = lhs.element_wise_gt(&rhs).unwrap();

        assert_eq!(
            result.values(),
            &OwnedColumn::Boolean(vec![false, false, true])
        );
        assert_eq!(result.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_not_preserves_presence_mask() {
        let column = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::Boolean(vec![true, false, true]),
            Some(vec![true, false, true]),
        )
        .unwrap();

        let result = column.element_wise_not().unwrap();

        assert_eq!(
            result.values(),
            &OwnedColumn::Boolean(vec![false, false, false])
        );
        assert_eq!(result.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn nullable_slice_and_permute_keep_values_and_presence_aligned() {
        let column = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::BigInt(vec![5, 0, 9, 2]),
            Some(vec![true, false, true, true]),
        )
        .unwrap();

        let sliced = column.slice(1, 4);
        assert_eq!(sliced.values(), &OwnedColumn::BigInt(vec![0, 9, 2]));
        assert_eq!(sliced.presence(), Some([false, true, true].as_slice()));

        let permutation = Permutation::try_new(vec![2, 0, 1]).unwrap();
        let permuted = sliced.try_permute(&permutation).unwrap();
        assert_eq!(permuted.values(), &OwnedColumn::BigInt(vec![2, 0, 9]));
        assert_eq!(permuted.presence(), Some([true, false, true].as_slice()));
    }

    #[test]
    fn borrowed_and_owned_nullable_columns_round_trip() {
        let alloc = Bump::new();
        let owned = NullableOwnedColumn::<TestScalar>::try_new(
            OwnedColumn::BigInt(vec![5, 0, 9]),
            Some(vec![true, false, true]),
        )
        .unwrap();

        let borrowed = owned.as_column(&alloc);
        assert_eq!(borrowed.values(), Column::BigInt(&[5, 0, 9]));
        assert_eq!(borrowed.presence(), Some([true, false, true].as_slice()));

        let round_trip = NullableOwnedColumn::try_from(borrowed).unwrap();
        assert_eq!(round_trip, owned);
    }
}
