use super::committable_column::CommittableColumn;
use thiserror::Error;

/// Inner value for [`Bounds::Sharp`] and [`Bounds::Bounded`].
///
/// Creating a separate type for this provides two benefits.
/// 1. reduced repeated code between the two variants
/// 2. privatization of the min/max for these variants, preventing invalid states
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BoundsInner<T>
where
    T: Ord,
{
    /// The minimum value of the data.
    min: T,
    /// The maximum value of the data.
    max: T,
}

impl<T> BoundsInner<T>
where
    T: Ord,
{
    /// Immutable accessor for the minimum value.
    pub fn min(&self) -> &T {
        &self.min
    }

    /// Immutable accessor for the maximum value.
    pub fn max(&self) -> &T {
        &self.max
    }

    /// Combine two [`Bounds`]s as if their source collections are being unioned.
    pub fn union(self, other: BoundsInner<T>) -> Self {
        BoundsInner {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Returns true if the value is within these bounds.
    ///
    /// This doesn't necessarily mean that the source collection contains this value.
    /// However, a `false` result implies that the source collection cannot contain this value.
    pub fn surrounds(&self, value: &T) -> bool {
        &self.min <= value && value <= &self.max
    }
}

/// Minimum and maximum values of a collection of data, with some other variants for edge cases.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum Bounds<T>
where
    T: Ord,
{
    /// The source collection is empty so has no bounds.
    #[default]
    Empty,
    /// After some operation (like [`Bounds::difference`]), the bounds cannot be determined exactly.
    ///
    /// Instead, this variant underestimates the minimum and overestimates the maximum.
    Bounded(BoundsInner<T>),
    /// The exact bounds of the values of the source collection.
    Sharp(BoundsInner<T>),
}

impl<T> Bounds<T>
where
    T: Ord,
{
    /// Combine two [`Bounds`]s as if their source collections are being unioned.
    fn union(self, other: Bounds<T>) -> Self {
        match (self, other) {
            (Bounds::Sharp(bounds_a), Bounds::Sharp(bounds_b)) => {
                Bounds::Sharp(bounds_a.union(bounds_b))
            }
            (Bounds::Bounded(bounds_a), Bounds::Bounded(bounds_b))
            | (Bounds::Bounded(bounds_a), Bounds::Sharp(bounds_b))
            | (Bounds::Sharp(bounds_a), Bounds::Bounded(bounds_b)) => {
                Bounds::Bounded(bounds_a.union(bounds_b))
            }
            (bounds, Bounds::Empty) | (Bounds::Empty, bounds) => bounds,
        }
    }

    /// Combine two [`Bounds`]s as if their source collections are being differenced.
    ///
    /// This should be interpreted as the set difference of the two collections.
    /// The result would be the rows in self that are not also rows in other.
    ///
    /// It can't be determined *which* values are being removed from self's source collection.
    /// So, in most cases, the resulting [`Bounds`] is [`Bounds::Bounded`].
    /// Exceptions to this are cases where it can be determined that *no* values are removed.
    fn difference(self, other: Bounds<T>) -> Self {
        match (self, other) {
            (Bounds::Empty, _) => Bounds::Empty,
            (bounds, Bounds::Empty) => bounds,
            (Bounds::Sharp(bounds_a), Bounds::Sharp(bounds_b))
            | (Bounds::Sharp(bounds_a), Bounds::Bounded(bounds_b))
                if bounds_a.max() < bounds_b.min() || bounds_b.max() < bounds_a.min() =>
            {
                // source collections must be disjoint, so no rows are removed
                Bounds::Sharp(bounds_a)
            }
            (Bounds::Bounded(bounds), _) | (Bounds::Sharp(bounds), _) => Bounds::Bounded(bounds),
        }
    }

    /// Returns true if the value is within these bounds.
    ///
    /// This doesn't necessarily mean that the source collection contains this value.
    /// However, a `false` result implies that the source collection cannot contain this value.
    pub fn surrounds(&self, value: &T) -> bool {
        match self {
            Bounds::Empty => false,
            Bounds::Bounded(inner) | Bounds::Sharp(inner) => inner.surrounds(value),
        }
    }
}

impl<'a, T> FromIterator<&'a T> for Bounds<T>
where
    T: Ord + Copy + 'a,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        let bounds_borrowed = iter
            .into_iter()
            .fold(Bounds::<&T>::Empty, |bounds, element| match bounds {
                Bounds::Sharp(BoundsInner { min, max }) => Bounds::Sharp(BoundsInner {
                    min: min.min(element),
                    max: max.max(element),
                }),
                Bounds::Empty => Bounds::Sharp(BoundsInner {
                    min: element,
                    max: element,
                }),
                Bounds::Bounded(_) => {
                    panic!("bounds should never be bounded in this function");
                }
            });

        // Copy only on the final bounds values
        match bounds_borrowed {
            Bounds::Sharp(BoundsInner { min, max }) => Bounds::Sharp(BoundsInner {
                min: *min,
                max: *max,
            }),
            Bounds::Empty => Bounds::Empty,
            Bounds::Bounded(_) => {
                panic!("bounds should never be bounded in this function")
            }
        }
    }
}

/// Columns with different [`ColumnBounds`] variants cannot operate with each other.
#[derive(Debug, Error)]
#[error("column with bounds {0:?} cannot operate with column with bounds {1:?}")]
pub struct ColumnBoundsMismatch(ColumnBounds, ColumnBounds);

/// Column metadata storing the bounds for column types that have order.
///
/// Other Ord column variants do exist (like Scalar/Boolean).
/// However, bounding these is useless unless we are performing indexing on these columns.
/// This functionality only be considered after we support them in the user-facing sql.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColumnBounds {
    /// Column does not have order.
    NoOrder,
    /// The bounds of a BigInt column.
    BigInt(Bounds<i64>),
    /// The bounds of an Int128 column.
    Int128(Bounds<i128>),
}

impl ColumnBounds {
    /// Construct a [`ColumnBounds`] from a column by reference.
    ///
    /// If the column variant has order, only the minimum and maximum value will be copied.
    pub fn from_column(column: &CommittableColumn) -> ColumnBounds {
        match column {
            CommittableColumn::BigInt(ints) => ColumnBounds::BigInt(Bounds::from_iter(*ints)),
            CommittableColumn::Int128(ints) => ColumnBounds::Int128(Bounds::from_iter(*ints)),
            _ => ColumnBounds::NoOrder,
        }
    }

    /// Combine two [`ColumnBounds`] as if their source collections are being unioned.
    ///
    /// Can error if the two values do not share the same [`ColumnBounds`] variant.
    pub fn try_union(self, other: Self) -> Result<Self, ColumnBoundsMismatch> {
        match (self, other) {
            (ColumnBounds::NoOrder, ColumnBounds::NoOrder) => Ok(ColumnBounds::NoOrder),
            (ColumnBounds::BigInt(bounds_a), ColumnBounds::BigInt(bounds_b)) => {
                Ok(ColumnBounds::BigInt(bounds_a.union(bounds_b)))
            }
            (ColumnBounds::Int128(bounds_a), ColumnBounds::Int128(bounds_b)) => {
                Ok(ColumnBounds::Int128(bounds_a.union(bounds_b)))
            }
            (bounds_a, bounds_b) => Err(ColumnBoundsMismatch(bounds_a, bounds_b)),
        }
    }

    /// Combine two [`ColumnBounds`] as if their source collections are being differenced.
    ///
    /// This should be interpreted as the set difference of the two collections.
    /// The result would be the rows in self that are not also rows in other.
    pub fn try_difference(self, other: Self) -> Result<Self, ColumnBoundsMismatch> {
        match (self, other) {
            (ColumnBounds::NoOrder, ColumnBounds::NoOrder) => Ok(self),
            (ColumnBounds::BigInt(bounds_a), ColumnBounds::BigInt(bounds_b)) => {
                Ok(ColumnBounds::BigInt(bounds_a.difference(bounds_b)))
            }
            (ColumnBounds::Int128(bounds_a), ColumnBounds::Int128(bounds_b)) => {
                Ok(ColumnBounds::Int128(bounds_a.difference(bounds_b)))
            }

            (_, _) => Err(ColumnBoundsMismatch(self, other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{database::OwnedColumn, scalar::ArkScalar};

    #[test]
    fn we_can_construct_bounds_from_iterator() {
        // empty case
        let empty_bounds = Bounds::<i32>::from_iter([]);
        assert_eq!(empty_bounds, Bounds::Empty);

        // nonempty case
        let ints = [1, 2, 3, 1, 0, -1];
        let bounds = Bounds::from_iter(&ints);
        assert_eq!(bounds, Bounds::Sharp(BoundsInner { min: -1, max: 3 }));
    }

    #[test]
    fn we_can_determine_if_bounds_surround_value() {
        // empty case
        assert!(!Bounds::Empty.surrounds(&0));

        let sharp = Bounds::Sharp(BoundsInner { min: 2, max: 4 });
        assert!(!sharp.surrounds(&1));
        assert!(sharp.surrounds(&2));
        assert!(sharp.surrounds(&3));
        assert!(sharp.surrounds(&4));
        assert!(!sharp.surrounds(&5));

        let bounded = Bounds::Bounded(BoundsInner { min: 2, max: 4 });
        assert!(!bounded.surrounds(&1));
        assert!(bounded.surrounds(&2));
        assert!(bounded.surrounds(&3));
        assert!(bounded.surrounds(&4));
        assert!(!bounded.surrounds(&5));
    }

    #[test]
    fn we_can_union_sharp_bounds() {
        let bounds_a = Bounds::Sharp(BoundsInner { min: 3, max: 6 });

        let bounds_b = Bounds::Sharp(BoundsInner { min: 1, max: 2 });
        assert_eq!(
            bounds_a.union(bounds_b),
            Bounds::Sharp(BoundsInner { min: 1, max: 6 })
        );

        let bounds_b = Bounds::Sharp(BoundsInner { min: 1, max: 4 });
        assert_eq!(
            bounds_a.union(bounds_b),
            Bounds::Sharp(BoundsInner { min: 1, max: 6 })
        );

        let bounds_b = Bounds::Sharp(BoundsInner { min: 1, max: 7 });
        assert_eq!(
            bounds_a.union(bounds_b),
            Bounds::Sharp(BoundsInner { min: 1, max: 7 })
        );

        let bounds_b = Bounds::Sharp(BoundsInner { min: 4, max: 5 });
        assert_eq!(
            bounds_a.union(bounds_b),
            Bounds::Sharp(BoundsInner { min: 3, max: 6 })
        );

        let bounds_b = Bounds::Sharp(BoundsInner { min: 4, max: 7 });
        assert_eq!(
            bounds_a.union(bounds_b),
            Bounds::Sharp(BoundsInner { min: 3, max: 7 })
        );

        let bounds_b = Bounds::Sharp(BoundsInner { min: 7, max: 8 });
        assert_eq!(
            bounds_a.union(bounds_b),
            Bounds::Sharp(BoundsInner { min: 3, max: 8 })
        );
    }

    #[test]
    fn we_can_union_sharp_and_empty_bounds() {
        let sharp = Bounds::Sharp(BoundsInner { min: 3, max: 6 });
        let empty = Bounds::Empty;

        assert_eq!(sharp.union(empty), sharp);
        assert_eq!(empty.union(sharp), sharp);
        assert_eq!(empty.union(empty), empty);
    }

    #[test]
    fn union_of_bounded_bounds_is_bounded() {
        let sharp = Bounds::Sharp(BoundsInner { min: 3, max: 6 });
        let bounded = Bounds::Bounded(BoundsInner { min: 7, max: 10 });
        let union = Bounds::Bounded(BoundsInner { min: 3, max: 10 });
        let empty = Bounds::Empty;

        assert_eq!(sharp.union(bounded), union);
        assert_eq!(bounded.union(sharp), union);

        assert_eq!(empty.union(bounded), bounded);
        assert_eq!(bounded.union(empty), bounded);

        assert_eq!(bounded.union(bounded), bounded);
    }

    #[test]
    fn we_can_take_difference_of_disjoint_bounds() {
        let bounds_a = Bounds::Sharp(BoundsInner { min: 3, max: 6 });
        let bounds_b = Bounds::Sharp(BoundsInner { min: -6, max: -3 });
        let bounded = Bounds::Bounded(BoundsInner { min: -6, max: -3 });

        assert_eq!(bounds_a.difference(bounds_b), bounds_a);
        assert_eq!(bounds_b.difference(bounds_a), bounds_b);

        assert_eq!(bounds_a.difference(bounded), bounds_a);

        let empty = Bounds::Empty;

        assert_eq!(bounds_a.difference(empty), bounds_a);
        assert_eq!(empty.difference(bounds_a), empty);

        assert_eq!(empty.difference(empty), empty);
    }

    #[test]
    fn difference_with_bounded_minuend_is_bounded() {
        let sharp = Bounds::Sharp(BoundsInner { min: -5, max: 5 });
        let bounded_a = Bounds::Bounded(BoundsInner { min: 6, max: 10 });
        let bounded_b = Bounds::Bounded(BoundsInner { min: 11, max: 15 });
        let empty = Bounds::Empty;

        assert_eq!(bounded_a.difference(sharp), bounded_a);

        assert_eq!(bounded_a.difference(bounded_b), bounded_a);
        assert_eq!(bounded_b.difference(bounded_a), bounded_b);

        assert_eq!(bounded_a.difference(empty), bounded_a);

        // Still empty since there are still no rows in empty that are also in bounded
        assert_eq!(empty.difference(bounded_a), empty);
    }

    #[test]
    fn difference_of_overlapping_bounds_is_bounded() {
        let bounds_a = BoundsInner { min: 3, max: 6 };
        let sharp_a = Bounds::Sharp(bounds_a);
        let bounded_a = Bounds::Bounded(bounds_a);

        let bounds_b = BoundsInner { min: 1, max: 4 };
        assert_eq!(sharp_a.difference(Bounds::Sharp(bounds_b)), bounded_a);
        assert_eq!(sharp_a.difference(Bounds::Bounded(bounds_b)), bounded_a);

        let bounds_b = BoundsInner { min: 1, max: 7 };
        assert_eq!(sharp_a.difference(Bounds::Sharp(bounds_b)), bounded_a);
        assert_eq!(sharp_a.difference(Bounds::Bounded(bounds_b)), bounded_a);

        let bounds_b = BoundsInner { min: 4, max: 5 };
        assert_eq!(sharp_a.difference(Bounds::Sharp(bounds_b)), bounded_a);
        assert_eq!(sharp_a.difference(Bounds::Bounded(bounds_b)), bounded_a);

        let bounds_b = BoundsInner { min: 4, max: 7 };
        assert_eq!(sharp_a.difference(Bounds::Sharp(bounds_b)), bounded_a);
        assert_eq!(sharp_a.difference(Bounds::Bounded(bounds_b)), bounded_a);
    }

    #[test]
    fn we_can_construct_column_bounds_from_column() {
        let varchar_column = OwnedColumn::<ArkScalar>::VarChar(
            ["Lorem", "ipsum", "dolor", "sit", "amet"]
                .map(String::from)
                .to_vec(),
        );
        let committable_varchar_column = CommittableColumn::from(&varchar_column);
        let varchar_column_bounds = ColumnBounds::from_column(&committable_varchar_column);
        assert_eq!(varchar_column_bounds, ColumnBounds::NoOrder);

        let bigint_column = OwnedColumn::<ArkScalar>::BigInt([1, 2, 3, 1, 0].to_vec());
        let committable_bigint_column = CommittableColumn::from(&bigint_column);
        let bigint_column_bounds = ColumnBounds::from_column(&committable_bigint_column);
        assert_eq!(
            bigint_column_bounds,
            ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 0, max: 3 }))
        );

        let int128_column = OwnedColumn::<ArkScalar>::Int128([1, 2, 3, 1, 0].to_vec());
        let committable_int128_column = CommittableColumn::from(&int128_column);
        let int128_column_bounds = ColumnBounds::from_column(&committable_int128_column);
        assert_eq!(
            int128_column_bounds,
            ColumnBounds::Int128(Bounds::Sharp(BoundsInner { min: 0, max: 3 }))
        );
    }

    #[test]
    fn we_can_union_column_bounds_with_matching_variant() {
        let no_order = ColumnBounds::NoOrder;
        assert_eq!(no_order.try_union(no_order).unwrap(), no_order);

        let bigint_a = ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 1, max: 3 }));
        let bigint_b = ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 4, max: 6 }));
        assert_eq!(
            bigint_a.try_union(bigint_b).unwrap(),
            ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 1, max: 6 }))
        );

        let int128_a = ColumnBounds::Int128(Bounds::Sharp(BoundsInner { min: 1, max: 3 }));
        let int128_b = ColumnBounds::Int128(Bounds::Bounded(BoundsInner { min: 4, max: 6 }));
        assert_eq!(
            int128_a.try_union(int128_b).unwrap(),
            ColumnBounds::Int128(Bounds::Bounded(BoundsInner { min: 1, max: 6 }))
        );
    }

    #[test]
    fn we_cannot_union_mismatched_column_bounds() {
        let no_order = ColumnBounds::NoOrder;
        let bigint = ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 1, max: 3 }));
        let int128 = ColumnBounds::Int128(Bounds::Sharp(BoundsInner { min: 4, max: 6 }));

        assert!(no_order.try_union(bigint).is_err());
        assert!(bigint.try_union(no_order).is_err());

        assert!(no_order.try_union(int128).is_err());
        assert!(int128.try_union(no_order).is_err());

        assert!(bigint.try_union(int128).is_err());
        assert!(int128.try_union(bigint).is_err());
    }

    #[test]
    fn we_can_difference_column_bounds_with_matching_variant() {
        let no_order = ColumnBounds::NoOrder;
        assert_eq!(no_order.try_difference(no_order).unwrap(), no_order);

        let bigint_a = ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 1, max: 3 }));
        let bigint_b = ColumnBounds::BigInt(Bounds::Empty);
        assert_eq!(bigint_a.try_difference(bigint_b).unwrap(), bigint_a);

        let int128_a = ColumnBounds::Int128(Bounds::Sharp(BoundsInner { min: 1, max: 4 }));
        let int128_b = ColumnBounds::Int128(Bounds::Sharp(BoundsInner { min: 3, max: 6 }));
        assert_eq!(
            int128_a.try_difference(int128_b).unwrap(),
            ColumnBounds::Int128(Bounds::Bounded(BoundsInner { min: 1, max: 4 }))
        );
    }

    #[test]
    fn we_cannot_difference_mismatched_column_bounds() {
        let no_order = ColumnBounds::NoOrder;
        let bigint = ColumnBounds::BigInt(Bounds::Sharp(BoundsInner { min: 1, max: 3 }));
        let int128 = ColumnBounds::Int128(Bounds::Sharp(BoundsInner { min: 4, max: 6 }));

        assert!(no_order.try_difference(bigint).is_err());
        assert!(bigint.try_difference(no_order).is_err());

        assert!(no_order.try_difference(int128).is_err());
        assert!(int128.try_difference(no_order).is_err());

        assert!(bigint.try_difference(int128).is_err());
        assert!(int128.try_difference(bigint).is_err());
    }
}
