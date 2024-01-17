use super::{committable_column::CommittableColumn, ColumnBounds};
use crate::base::database::ColumnType;
use std::fmt::Debug;
use thiserror::Error;

/// During column operation, metadata indicates that the operand columns cannot be the same.
#[derive(Debug, Error)]
#[error("column with type {0} cannot operate with column with type {1}")]
pub struct ColumnCommitmentMetadataMismatch(ColumnType, ColumnType);

const EXPECT_BOUNDS_MATCH_MESSAGE: &str = "we've already checked the column types match, which is a stronger requirement (mapping of type variants to bounds variants is surjective)";

/// Anonymous metadata associated with a column commitment.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColumnCommitmentMetadata {
    column_type: ColumnType,
    bounds: ColumnBounds,
}

impl ColumnCommitmentMetadata {
    /// Immutable reference to this column's type.
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }

    /// Immutable reference to this column's bounds.
    pub fn bounds(&self) -> &ColumnBounds {
        &self.bounds
    }

    /// Contruct a [`ColumnCommitmentMetadata`] by analyzing a column.
    pub fn from_column(column: &CommittableColumn) -> ColumnCommitmentMetadata {
        ColumnCommitmentMetadata {
            column_type: column.column_type(),
            bounds: ColumnBounds::from_column(column),
        }
    }

    /// Combine two [`ColumnCommitmentMetadata`] as if their source collections are being unioned.
    ///
    /// Can error if the two metadatas are mismatched.
    pub fn try_union(
        self,
        other: ColumnCommitmentMetadata,
    ) -> Result<ColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch> {
        if self.column_type != other.column_type {
            return Err(ColumnCommitmentMetadataMismatch(
                self.column_type,
                other.column_type,
            ));
        }

        let bounds = self
            .bounds
            .try_union(other.bounds)
            .expect(EXPECT_BOUNDS_MATCH_MESSAGE);

        Ok(ColumnCommitmentMetadata {
            bounds,
            column_type: self.column_type,
        })
    }

    /// Combine two [`ColumnBounds`] as if their source collections are being differenced.
    ///
    /// This should be interpreted as the set difference of the two collections.
    /// The result would be the rows in self that are not also rows in other.
    pub fn try_difference(
        self,
        other: ColumnCommitmentMetadata,
    ) -> Result<ColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch> {
        if self.column_type != other.column_type {
            return Err(ColumnCommitmentMetadataMismatch(
                self.column_type,
                other.column_type,
            ));
        }

        let bounds = self
            .bounds
            .try_difference(other.bounds)
            .expect(EXPECT_BOUNDS_MATCH_MESSAGE);

        Ok(ColumnCommitmentMetadata {
            bounds,
            column_type: self.column_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        commitment::column_bounds::Bounds, database::OwnedColumn, scalar::ArkScalar,
    };

    #[test]
    fn we_can_construct_metadata_from_column() {
        let varchar_column = OwnedColumn::VarChar(
            ["Lorem", "ipsum", "dolor", "sit", "amet"]
                .map(String::from)
                .to_vec(),
        );
        let committable_varchar_column = CommittableColumn::from(&varchar_column);
        let varchar_metadata = ColumnCommitmentMetadata::from_column(&committable_varchar_column);
        assert_eq!(varchar_metadata.column_type(), &ColumnType::VarChar);
        assert_eq!(varchar_metadata.bounds(), &ColumnBounds::NoOrder);

        let bigint_column = OwnedColumn::BigInt([1, 2, 3, 1, 0].to_vec());
        let committable_bigint_column = CommittableColumn::from(&bigint_column);
        let bigint_metadata = ColumnCommitmentMetadata::from_column(&committable_bigint_column);
        assert_eq!(bigint_metadata.column_type(), &ColumnType::BigInt);
        if let ColumnBounds::BigInt(Bounds::Sharp(bounds)) = bigint_metadata.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("Bounds constructed from nonempty BigInt column should be ColumnBounds::BigInt(Bounds::Sharp(_))");
        }

        let int128_column = OwnedColumn::Int128([].to_vec());
        let committable_int128_column = CommittableColumn::from(&int128_column);
        let int128_metadata = ColumnCommitmentMetadata::from_column(&committable_int128_column);
        assert_eq!(int128_metadata.column_type(), &ColumnType::Int128);
        assert_eq!(
            int128_metadata.bounds(),
            &ColumnBounds::Int128(Bounds::Empty)
        );

        let scalar_column = OwnedColumn::Scalar([1, 2, 3, 4, 5].map(ArkScalar::from).to_vec());
        let committable_scalar_column = CommittableColumn::from(&scalar_column);
        let scalar_metadata = ColumnCommitmentMetadata::from_column(&committable_scalar_column);
        assert_eq!(scalar_metadata.column_type(), &ColumnType::Scalar);
        assert_eq!(scalar_metadata.bounds(), &ColumnBounds::NoOrder);
    }

    #[test]
    fn we_can_union_matching_metadata() {
        // NoOrder cases
        let varchar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::VarChar,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            varchar_metadata.try_union(varchar_metadata).unwrap(),
            varchar_metadata
        );

        let scalar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Scalar,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            scalar_metadata.try_union(scalar_metadata).unwrap(),
            scalar_metadata
        );

        // Ordered case
        let ints = [1, 2, 3, 1, 0];
        let bigint_column_a = CommittableColumn::BigInt(&ints[..2]);
        let bigint_metadata_a = ColumnCommitmentMetadata::from_column(&bigint_column_a);
        let bigint_column_b = CommittableColumn::BigInt(&ints[2..]);
        let bigint_metadata_b = ColumnCommitmentMetadata::from_column(&bigint_column_b);
        let bigint_column_c = CommittableColumn::BigInt(&ints);
        let bigint_metadata_c = ColumnCommitmentMetadata::from_column(&bigint_column_c);
        assert_eq!(
            bigint_metadata_a.try_union(bigint_metadata_b).unwrap(),
            bigint_metadata_c
        );
    }

    #[test]
    fn we_can_difference_matching_metadata() {
        // NoOrder cases
        let varchar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::VarChar,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            varchar_metadata.try_union(varchar_metadata).unwrap(),
            varchar_metadata
        );

        let scalar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Scalar,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            scalar_metadata.try_union(scalar_metadata).unwrap(),
            scalar_metadata
        );

        // Ordered case
        let ints = [1, 2, 3, 1, 0];
        let bigint_column_a = CommittableColumn::BigInt(&ints[..2]);
        let bigint_metadata_a = ColumnCommitmentMetadata::from_column(&bigint_column_a);
        let bigint_column_b = CommittableColumn::BigInt(&ints);
        let bigint_metadata_b = ColumnCommitmentMetadata::from_column(&bigint_column_b);

        let b_difference_a = bigint_metadata_b.try_difference(bigint_metadata_a).unwrap();
        assert_eq!(b_difference_a.column_type, ColumnType::BigInt);
        if let ColumnBounds::BigInt(Bounds::Bounded(bounds)) = b_difference_a.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        let bigint_column_empty = CommittableColumn::BigInt(&[]);
        let bigint_metadata_empty = ColumnCommitmentMetadata::from_column(&bigint_column_empty);

        assert_eq!(
            bigint_metadata_b
                .try_difference(bigint_metadata_empty)
                .unwrap(),
            bigint_metadata_b
        );
        assert_eq!(
            bigint_metadata_empty
                .try_difference(bigint_metadata_b)
                .unwrap(),
            bigint_metadata_empty
        );
    }

    #[test]
    fn we_cannot_perform_arithmetic_on_mismatched_metadata() {
        let varchar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::VarChar,
            bounds: ColumnBounds::NoOrder,
        };
        let scalar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Scalar,
            bounds: ColumnBounds::NoOrder,
        };
        let bigint_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::BigInt,
            bounds: ColumnBounds::BigInt(Bounds::Empty),
        };
        let int128_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Int128,
            bounds: ColumnBounds::Int128(Bounds::Empty),
        };

        assert!(varchar_metadata.try_union(scalar_metadata).is_err());
        assert!(scalar_metadata.try_union(varchar_metadata).is_err());

        assert!(varchar_metadata.try_union(bigint_metadata).is_err());
        assert!(bigint_metadata.try_union(varchar_metadata).is_err());

        assert!(varchar_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(varchar_metadata).is_err());

        assert!(scalar_metadata.try_union(bigint_metadata).is_err());
        assert!(bigint_metadata.try_union(scalar_metadata).is_err());

        assert!(scalar_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(scalar_metadata).is_err());

        assert!(bigint_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(bigint_metadata).is_err());

        assert!(varchar_metadata.try_difference(scalar_metadata).is_err());
        assert!(scalar_metadata.try_difference(varchar_metadata).is_err());

        assert!(varchar_metadata.try_difference(bigint_metadata).is_err());
        assert!(bigint_metadata.try_difference(varchar_metadata).is_err());

        assert!(varchar_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(varchar_metadata).is_err());

        assert!(scalar_metadata.try_difference(bigint_metadata).is_err());
        assert!(bigint_metadata.try_difference(scalar_metadata).is_err());

        assert!(scalar_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(scalar_metadata).is_err());

        assert!(bigint_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(bigint_metadata).is_err());
    }
}
